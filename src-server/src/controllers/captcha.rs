use std::collections::HashMap;

use axum::extract::State;
use axum::{body::Body, response::Response};
use mobilesuica_sheet_app_server::{
    HttpClient::{get_client, get_cookies, MobilesuicaCookies, BASE_URL},
    MobilesuicaFormParams,
};
use reqwest::StatusCode;
use scraper::{Html, Selector};

use crate::store::AppState;

fn get_captcha_imageurl(html: &str) -> String {
    let document = Html::parse_document(html);

    let selector = Selector::parse(".igc_TrendyCaptchaImage").unwrap();

    let captcha_element = document.select(&selector).next();

    let captcha_url: &str = match captcha_element {
        Some(element) => element.value().attr("src").unwrap_or(""),
        None => "",
    };

    String::from(captcha_url)
}

fn get_action_url(html: &str) -> String {
    let document = Html::parse_document(html);

    let selector = Selector::parse("#form1").unwrap();

    let form_element = document.select(&selector).next();

    let action_url: &str = match form_element {
        Some(element) => element.value().attr("action").unwrap_or(""),
        None => "",
    };

    String::from(action_url)
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

    let client = get_client(cookies_default).await.unwrap();

    let (mobilesuica_form_params, cookies, captcha_url, action_url) =
        fetch_mobilesuica(&client).await.unwrap();

    let captcha_image = download_captcha(&client, &captcha_url).await.unwrap();

    {
        let mut session = state.session.lock().unwrap();

        session.set("action_url", action_url);
        session.set("mobilesuica_form_params", mobilesuica_form_params);
        session.set("cookies", cookies);
    }

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "image/gif")
        .body(Body::from(captcha_image))
        .unwrap()
}
