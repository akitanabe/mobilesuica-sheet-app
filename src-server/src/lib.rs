use bytes::Bytes;
use scraper::{Html, Selector};

const BASE_URL: &str = "https://www.mobilesuica.com/";

pub async fn get_client() -> Result<reqwest::Client, reqwest::Error> {
    let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/115.0.0.0 Safari/537.36";
    let client = reqwest::Client::builder()
        .user_agent(ua)
        .cookie_store(true)
        .build()?;

    Ok(client)
}

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

pub async fn fetch_captcha_url(client: &reqwest::Client) -> Result<String, reqwest::Error> {
    let response = client.get(BASE_URL).send().await?;

    let html = response.text_with_charset("utf-8").await?;

    let captcha_url = get_captcha_imageurl(&html);

    Ok(captcha_url)
}

pub async fn download_captcha(
    client: &reqwest::Client,
    captcha_url: &str,
) -> Result<Bytes, reqwest::Error> {
    let url = format!("{}{}", BASE_URL, captcha_url);

    let response = client.get(url).send().await?;

    let captcha_image = response.bytes().await?;

    Ok(captcha_image)
}
