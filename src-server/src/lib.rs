use std::{collections::HashMap, sync::Arc};

use bytes::Bytes;

use reqwest::cookie;
use scraper::{Html, Selector};

use serde::{Deserialize, Serialize};
use url::Url;

pub type MobilesuicaCookies = HashMap<String, String>;
pub const BASE_URL: &str = "https://www.mobilesuica.com/";

#[allow(non_snake_case, dead_code)]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MobilesuicaFormParams {
    __EVENTARGUMENT: String,
    __EVENTTARGET: String,
    __VIEWSTATE: String,
    __VIEWSTATEENCRYPTED: String,
    __VIEWSTATEGENERATOR: String,
    baseVariable: String,
    baseVarLogoutBtn: String,
    LOGIN: String,
    MailAddress: String,
    Password: String,
    WebCaptcha1__editor: String,
    WebCaptcha1__editor_clientState: String,
    WebCaptcha1_clientState: String,
}

impl MobilesuicaFormParams {
    fn new(html: &str) -> Self {
        let document = Html::parse_document(html);

        let get_input_value = |name: &str| {
            let selector = Selector::parse(&format!("input[name='{}']", name)).unwrap();

            let element = &document.select(&selector).next();

            match element {
                Some(element) => element.value().attr("value").unwrap_or(""),
                None => "",
            }
            .to_string()
        };

        MobilesuicaFormParams {
            __EVENTARGUMENT: get_input_value("__EVENTARGUMENT"),
            __EVENTTARGET: get_input_value("__EVENTTARGET"),
            __VIEWSTATE: get_input_value("__VIEWSTATE"),
            __VIEWSTATEENCRYPTED: get_input_value("__VIEWSTATEENCRYPTED"),
            __VIEWSTATEGENERATOR: get_input_value("__VIEWSTATEGENERATOR"),
            baseVariable: String::new(),
            baseVarLogoutBtn: String::from("off"),
            LOGIN: String::from("ログイン"),
            MailAddress: String::new(),
            Password: String::new(),
            WebCaptcha1__editor: String::new(),
            WebCaptcha1__editor_clientState: String::new(),
            WebCaptcha1_clientState: String::from("[[[[null]],[],[]],[{},[]],null]"),
        }
    }

    fn set_captcha(&mut self, captcha: &str) {
        self.WebCaptcha1__editor = String::from(captcha);
    }

    fn set_mail_address(&mut self, mail_address: &str) {
        self.MailAddress = String::from(mail_address);
    }

    fn set_password(&mut self, password: &str) {
        self.Password = String::from(password);
    }
}

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

fn get_cookies(response: &reqwest::Response) -> MobilesuicaCookies {
    response
        .cookies()
        .map(|cookie| (cookie.name().to_string(), cookie.value().to_string()))
        .collect::<HashMap<_, _>>()
}

pub async fn fetch_mobilesuica(
    client: &reqwest::Client,
) -> Result<(MobilesuicaFormParams, MobilesuicaCookies, String), reqwest::Error> {
    let response = client.get(BASE_URL).send().await?;
    let cookies = get_cookies(&response);

    let html = response.text_with_charset("utf-8").await?;
    let mobilesuica_form_params = MobilesuicaFormParams::new(&html);

    let captcha_url = get_captcha_imageurl(&html);

    Ok((mobilesuica_form_params, cookies, captcha_url))
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
