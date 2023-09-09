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
    let (cookies, mobilesuica_form_params, action_url) = {
        let session = state.session.lock().unwrap();

        let mut mobilesuica_form_params =
            session.get::<MobilesuicaFormParams>("mobilesuica_form_params");

        // 入力値セット
        mobilesuica_form_params
            .set_mail_address(&payload.email)
            .set_password(&payload.password)
            .set_captcha(&payload.captcha);

        let cookies = session.get::<MobilesuicaCookies>("cookies");
        let action_url = session.get::<String>("action_url");

        (cookies, mobilesuica_form_params, action_url)
    };

    let client = get_client(cookies).await.unwrap();

    let (success, auth_cookies) = login(&client, action_url, &mobilesuica_form_params)
        .await
        .unwrap();

    if success {
        let mut session = state.session.lock().unwrap();
        session.set("cookies", auth_cookies);
    }

    let message = match success {
        true => "ログイン成功",
        false => "ログイン失敗",
    };
    let auth_mobilesuica = AuthMobilesuica::new(
        success,
        AuthMobilesuicaResult {
            success,
            message: message.to_string(),
        },
    );

    Json(auth_mobilesuica)
}

#[cfg(test)]
mod test {

    use super::*;

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
}
