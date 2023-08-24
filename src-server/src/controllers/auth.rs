use axum::extract::State;
use axum::response::Response;
use axum::Form;
use mobilesuica_sheet_app_server::HttpClient::{get_client, MobilesuicaCookies};
use serde::Deserialize;

use crate::AppState;
use mobilesuica_sheet_app_server::MobilesuicaFormParams;

#[derive(Deserialize, Debug)]
pub struct Payload {
    email: String,
    password: String,
    captcha: String,
}

pub async fn login(
    client: &reqwest::Client,
    form_params: &MobilesuicaFormParams,
) -> Result<(), reqwest::Error> {
    client
        .post("http:://127.0.0.1/request")
        .form(form_params)
        .send()
        .await?;

    Ok(())
}

pub async fn handler(State(state): State<AppState>, payload: Form<Payload>) -> Response {
    let (cookies, mobilesuica_form_params) = {
        let session = state.session.lock().unwrap();

        let mut mobilesuica_form_params =
            session.get::<MobilesuicaFormParams>("mobilesuica_form_params");

        // 入力値セット
        mobilesuica_form_params
            .set_mail_address(&payload.email)
            .set_password(&payload.password)
            .set_captcha(&payload.captcha);

        let cookies = session.get::<MobilesuicaCookies>("cookies");

        (cookies, mobilesuica_form_params)
    };

    let client = get_client(cookies).await.unwrap();

    // let _result = login(&client, &mobilesuica_form_params).await;

    Response::new("Hello, World!".into())
}
