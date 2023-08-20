use axum::extract::State;
use axum::{body::Body, response::Response};
use mobilesuica_sheet_app_server::{
    download_captcha, fetch_mobilesuica, get_client, MobilesuicaCookies,
};
use reqwest::StatusCode;

use crate::AppState;

pub async fn handler(State(state): State<AppState>) -> Response {
    let mut session = state.session.lock().unwrap().clone();

    let client = get_client(session.get::<MobilesuicaCookies>("cookies"))
        .await
        .unwrap();

    let (mobilesuica_form_params, cookies, captcha_url) = fetch_mobilesuica(&client).await.unwrap();

    let captcha_image = download_captcha(&client, &captcha_url).await.unwrap();

    session.set("mobilesuica_form_params", mobilesuica_form_params);
    session.set("cookies", cookies);

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "image/gif")
        .body(Body::from(captcha_image.to_vec()))
        .unwrap()
}
