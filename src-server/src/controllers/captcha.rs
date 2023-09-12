use std::collections::HashMap;

use axum::extract::State;
use axum::{body::Body, response::Response};
use mobilesuica_sheet_app_server::HtmlDocument;
use mobilesuica_sheet_app_server::{
    HttpClient::{get_client, get_cookies, MobilesuicaCookies, BASE_URL},
    MobilesuicaFormParams,
};
use reqwest::StatusCode;

use crate::store::AppState;

#[derive(Debug, PartialEq)]
enum CaptchaError {
    FetchFailed,
    DownloadFailed,
}

fn get_captcha_error_message(error: CaptchaError) -> String {
    match error {
        CaptchaError::FetchFailed => "キャプチャ画像の取得に失敗しました。",
        CaptchaError::DownloadFailed => "キャプチャ画像のダウンロードに失敗しました。",
    }
    .to_string()
}

fn get_captcha_imageurl(html: &str) -> String {
    let document = HtmlDocument::new(html);

    match document.query_selector(".igc_TrendyCaptchaImage") {
        Some(element) => element.value().attr("src").unwrap_or(""),
        None => "",
    }
    .to_string()
}

fn get_action_url(html: &str) -> String {
    let document = HtmlDocument::new(html);

    match document.get_element_by_id("form1") {
        Some(element) => element.value().attr("action").unwrap_or(""),
        None => "",
    }
    .to_string()
}

async fn fetch_mobilesuica(
    client: &reqwest::Client,
) -> Result<(MobilesuicaFormParams, MobilesuicaCookies, String, String), reqwest::Error> {
    let response = client.get(BASE_URL).send().await?;
    let cookies = get_cookies(&response);

    let html = response.text_with_charset("utf-8").await?;
    let mobilesuica_form_params = MobilesuicaFormParams::new(&html);

    let captcha_url = get_captcha_imageurl(&html);
    let action_url = get_action_url(&html);

    Ok((mobilesuica_form_params, cookies, captcha_url, action_url))
}

async fn download_captcha(
    client: &reqwest::Client,
    captcha_url: &str,
) -> Result<Vec<u8>, reqwest::Error> {
    let url = format!("{}{}", BASE_URL, captcha_url);

    let response = client.get(url).send().await?;

    let captcha_image = response.bytes().await?;

    Ok(captcha_image.to_vec())
}

pub async fn handler(State(state): State<AppState>) -> Response {
    let cookies_default: MobilesuicaCookies = HashMap::new();

    let client = match get_client(cookies_default).await {
        Ok(client) => client,
        Err(_) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Internal Server Error"))
                .unwrap();
        }
    };

    let result: Result<Vec<u8>, CaptchaError> = (|| async {
        let (mobilesuica_form_params, cookies, captcha_url, action_url) =
            fetch_mobilesuica(&client)
                .await
                .map_err(|_| CaptchaError::FetchFailed)?;

        let captcha_image = download_captcha(&client, &captcha_url)
            .await
            .map_err(|_| CaptchaError::DownloadFailed)?;

        {
            let mut session = state.session.lock().unwrap();

            session.set("action_url", action_url);
            session.set("mobilesuica_form_params", mobilesuica_form_params);
            session.set("cookies", cookies);
        }

        Ok(captcha_image)
    })()
    .await;

    match result {
        Ok(captcha_image) => Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "image/gif")
            .body(Body::from(captcha_image)),

        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(get_captcha_error_message(e))),
    }
    .unwrap()
}
