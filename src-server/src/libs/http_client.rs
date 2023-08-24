use reqwest::cookie;
use std::{collections::HashMap, sync::Arc};
use url::Url;

pub type MobilesuicaCookies = HashMap<String, String>;
pub const BASE_URL: &str = "https://www.mobilesuica.com/";

pub async fn get_client(cookies: MobilesuicaCookies) -> Result<reqwest::Client, reqwest::Error> {
    let cookie_store = Arc::new(cookie::Jar::default());

    for (name, value) in cookies {
        cookie_store.add_cookie_str(
            &format!("{}={}", name, value),
            &Url::parse(BASE_URL).unwrap(),
        );
    }

    let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/115.0.0.0 Safari/537.36";
    let client = reqwest::Client::builder()
        .user_agent(ua)
        .cookie_provider(cookie_store)
        .build()?;

    Ok(client)
}

pub fn get_cookies(response: &reqwest::Response) -> MobilesuicaCookies {
    response
        .cookies()
        .map(|cookie| (cookie.name().to_string(), cookie.value().to_string()))
        .collect::<HashMap<_, _>>()
}
