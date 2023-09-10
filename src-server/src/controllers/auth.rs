use axum::Form;
use axum::{extract::State, Json};
use mobilesuica_sheet_app_server::HttpClient::{
    get_client, get_cookies, MobilesuicaCookies, BASE_URL,
};
use reqwest::header::{HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

use crate::AppState;
use mobilesuica_sheet_app_server::{HtmlDocument, MobilesuicaFormParams};

#[derive(Deserialize, Debug)]
pub struct Payload {
    email: String,
    password: String,
    captcha: String,
}

#[derive(Debug, PartialEq)]
enum AuthError {
    SessionNotFound,
    RequestFailed,
    LoginFailed,
}

fn get_auth_error_message(error: AuthError) -> String {
    match error {
        AuthError::SessionNotFound => {
            "ログイン情報が取得できません。キャプチャ画像を再取得してください。"
        }
        AuthError::LoginFailed => "ログインに失敗しました。",
        AuthError::RequestFailed => "ログインリクエストに失敗しました。",
    }
    .to_string()
}

async fn login(
    client: &reqwest::Client,
    action_url: String,
    form_params: &MobilesuicaFormParams,
) -> Result<(bool, MobilesuicaCookies), reqwest::Error> {
    let url = format!("{}{}", BASE_URL, action_url);

    let form_body = form_params.serialize_into_sjis();

    let response = client
        .post(url)
        .header(
            CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        )
        .body(form_body)
        .send()
        .await?;

    let cookies = get_cookies(&response);
    let html = response.text_with_charset("utf-8").await?;

    let title = get_title(&html);

    match title.as_str() {
        "JR東日本：モバイルSuica＞会員メニュー" => Ok((true, cookies)),
        _ => Ok((false, cookies)),
    }
}

fn get_title(html: &str) -> String {
    let document = HtmlDocument::new(html);

    match document.query_selector("title") {
        Some(element) => element.text().collect::<String>(),
        None => "".to_string(),
    }
}

fn get_session_items(
    state: &AppState,
) -> Result<(MobilesuicaCookies, MobilesuicaFormParams, String), AuthError> {
    let session = state.session.lock().unwrap();

    let mobilesuica_form_params = session
        .get::<MobilesuicaFormParams>("mobilesuica_form_params")
        .ok_or(AuthError::SessionNotFound)?;

    let cookies = session
        .get::<MobilesuicaCookies>("cookies")
        .ok_or(AuthError::SessionNotFound)?;

    let action_url = session
        .get::<String>("action_url")
        .ok_or(AuthError::SessionNotFound)?;

    Ok((cookies, mobilesuica_form_params, action_url))
}

fn create_auth_response(success: bool, message: String) -> AuthMobilesuica {
    AuthMobilesuica::new(success, AuthMobilesuicaResult { success, message })
}

#[derive(Serialize, Debug)]
pub struct AuthMobilesuica {
    ok: bool,
    result: AuthMobilesuicaResult,
}

#[derive(Serialize, Debug)]
struct AuthMobilesuicaResult {
    success: bool,
    message: String,
}

impl AuthMobilesuica {
    fn new(ok: bool, result: AuthMobilesuicaResult) -> Self {
        AuthMobilesuica { ok, result }
    }
}

pub async fn handler(
    State(state): State<AppState>,
    payload: Form<Payload>,
) -> Json<AuthMobilesuica> {
    let (cookies, mut mobilesuica_form_params, action_url) = match get_session_items(&state) {
        Ok(items) => items,
        Err(e) => {
            let message = get_auth_error_message(e);
            return Json(create_auth_response(false, message));
        }
    };

    // 入力値セット
    mobilesuica_form_params
        .set_mail_address(&payload.email)
        .set_password(&payload.password)
        .set_captcha(&payload.captcha);

    let result: Result<(bool, MobilesuicaCookies), AuthError> = {
        let client = get_client(cookies).await;

        if client.is_ok() {
            login(&client.unwrap(), action_url, &mobilesuica_form_params)
                .await
                .map_err(|_| AuthError::RequestFailed)
        } else {
            Err(AuthError::RequestFailed)
        }
    };

    let (success, auth_cookies) = match result {
        Ok((success, cookies)) => (success, cookies),
        Err(e) => {
            let message = get_auth_error_message(e);
            return Json(create_auth_response(false, message));
        }
    };

    if success {
        let mut session = state.session.lock().unwrap();
        session.set("auth_cookies", auth_cookies);
    }

    let message = match success {
        true => "ログイン成功".to_string(),
        false => get_auth_error_message(AuthError::LoginFailed),
    };

    let auth_mobilesuica = create_auth_response(true, message);

    Json(auth_mobilesuica)
}

#[cfg(test)]
mod test {

    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    use super::*;
    use mobilesuica_sheet_app_server::Session;

    #[test]
    fn test_get_title() {
        let html = r#"
        <html>
            <head>
                <title>test</title>
            </head>
            <body>
            </body>
        </html>
        "#;

        let title = get_title(html);

        assert_eq!(title, "test".to_string());
    }

    fn get_state(session: &Session) -> AppState {
        AppState {
            session: Arc::new(Mutex::new(session.clone())),
        }
    }

    #[test]
    fn test_get_session_items() {
        let session_id = Session::new();
        let mut session = Session::get_session(&session_id).unwrap();

        let mut cookies: MobilesuicaCookies = HashMap::new();

        cookies.insert("test".to_string(), "test_cookie".to_string());

        let mut mobilesuica_form_params = MobilesuicaFormParams::default();
        mobilesuica_form_params.set_mail_address("test@example.com");

        let action_url = "action_url".to_string();

        // cookieのみセット
        session.set("cookies", &cookies);

        {
            let state = get_state(&session);

            assert_eq!(
                get_session_items(&state).err().unwrap(),
                AuthError::SessionNotFound
            );
        }

        // MobileSuicaFormParamsを追加でセット
        session.set("mobilesuica_form_params", &mobilesuica_form_params);

        {
            let state = get_state(&session);

            assert_eq!(
                get_session_items(&state).err().unwrap(),
                AuthError::SessionNotFound
            );
        }

        // action_urlを追加でセット
        session.set("action_url", action_url);

        {
            let state = get_state(&session);

            match get_session_items(&state) {
                Ok(items) => {
                    assert_eq!(items.0, cookies);
                    assert_eq!(items.1, mobilesuica_form_params);
                    assert_eq!(items.2, "action_url");
                }
                Err(_) => assert!(false),
            }
        }
    }
}
