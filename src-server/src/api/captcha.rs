use std::sync::Arc;

use axum::{body::Body, response::Response};
use mobilesuica_sheet_app_server::library::session::Session;
use mobilesuica_sheet_app_server::{download_captcha, fetch_mobilesuica, get_client};
use reqwest::StatusCode;

pub async fn handler() -> Response {
    let client = get_client().await.unwrap();
    let (mobilesuica_form_params, cookies, captcha_url) = fetch_mobilesuica(&client).await.unwrap();

    let captcha_image = download_captcha(&client, &captcha_url).await.unwrap();

    let session_id = Session::new();

    let mut session = Session::get_session(session_id);

    session.set("mobilesuica_form_params", mobilesuica_form_params);
    session.set("cookies", cookies);

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "image/gif")
        .body(Body::from(captcha_image.to_vec()))
        .unwrap()
}
