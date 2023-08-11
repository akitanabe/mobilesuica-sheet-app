use axum::{body::Body, response::Response};
use mobilesuica_sheet_app_server::{download_captcha, fetch_captcha_url, get_client};
use reqwest::StatusCode;

pub async fn handler() -> Response {
    let client = get_client().await.unwrap();
    let captcha_url = fetch_captcha_url(&client).await.unwrap();

    let captcha_image = download_captcha(&client, &captcha_url).await.unwrap();

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "image/gif")
        .body(Body::from(captcha_image.to_vec()))
        .unwrap()
}
