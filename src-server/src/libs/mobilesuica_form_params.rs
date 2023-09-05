use encoding_rs::SHIFT_JIS;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

// SJISにしてURIエンコードする
fn encode_uri_from_sjis(text: &str) -> String {
    SHIFT_JIS
        .encode(text)
        .0
        .to_vec()
        .iter()
        .fold(String::new(), |encoded_str, byte| match byte {
            32 => format!("{}+", encoded_str),
            42 | 45 | 46 | 95 => format!("{}{}", encoded_str, *byte as char),
            48..=57 | 65..=90 | 97..=122 => format!("{}{}", encoded_str, *byte as char),
            _ => format!("{}%{:2X}", encoded_str, byte),
        })
}

#[allow(non_snake_case)]
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
    pub fn new(html: &str) -> Self {
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
            baseVariable: get_input_value("baseVariable"),
            baseVarLogoutBtn: String::from("off"),
            LOGIN: String::from("ログイン"),
            MailAddress: String::new(),
            Password: String::new(),
            WebCaptcha1__editor: String::new(),
            WebCaptcha1__editor_clientState: String::new(),
            WebCaptcha1_clientState: String::from("[[[[null]],[],[]],[{},[]],null]"),
        }
    }

    pub fn set_captcha(&mut self, captcha: &str) -> &mut Self {
        self.WebCaptcha1__editor = String::from(captcha);
        self.WebCaptcha1__editor_clientState =
            format!("|0|01{0}||[[[[]],[],[]],[{1},[]],\"01{0}\"]", captcha, "{}");

        self
    }

    pub fn set_mail_address(&mut self, mail_address: &str) -> &mut Self {
        self.MailAddress = String::from(mail_address);

        self
    }

    pub fn set_password(&mut self, password: &str) -> &mut Self {
        self.Password = String::from(password);

        self
    }

    // 各パラメータをSJISにしてURIエンコードする
    pub fn serialize_into_sjis(&self) -> String {
        [
            ("__EVENTARGUMENT", &self.__EVENTARGUMENT),
            ("__EVENTTARGET", &self.__EVENTTARGET),
            ("__VIEWSTATE", &self.__VIEWSTATE),
            ("__VIEWSTATEENCRYPTED", &self.__VIEWSTATEENCRYPTED),
            ("__VIEWSTATEGENERATOR", &self.__VIEWSTATEGENERATOR),
            ("baseVariable", &self.baseVariable),
            ("baseVarLogoutBtn", &self.baseVarLogoutBtn),
            ("LOGIN", &self.LOGIN),
            ("MailAddress", &self.MailAddress),
            ("Password", &self.Password),
            ("WebCaptcha1__editor", &self.WebCaptcha1__editor),
            (
                "WebCaptcha1__editor_clientState",
                &self.WebCaptcha1__editor_clientState,
            ),
            ("WebCaptcha1_clientState", &self.WebCaptcha1_clientState),
        ]
        .iter()
        .map(|(key, value)| {
            format!(
                "{}={}",
                encode_uri_from_sjis(key),
                encode_uri_from_sjis(value)
            )
        })
        .collect::<Vec<String>>()
        .join("&")
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_encode_uri_from_sjis() {
        let text = "あいうえお";

        let encoded = encode_uri_from_sjis(text);

        assert_eq!(encoded, "%82%A0%82%A2%82%A4%82%A6%82%A8");

        let text2 = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

        let encoded2 = encode_uri_from_sjis(text2);

        assert_eq!(encoded2, "ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    }

    #[test]
    fn test_mobilesuica_form_params_serialize() {
        let form_params = MobilesuicaFormParams {
            __EVENTARGUMENT: String::from("__EVENTARGUMENT"),
            __EVENTTARGET: String::from("__EVENTTARGET"),
            __VIEWSTATE: String::from("__VIEWSTATE"),
            __VIEWSTATEENCRYPTED: String::from("__VIEWSTATEENCRYPTED"),
            __VIEWSTATEGENERATOR: String::from("__VIEWSTATEGENERATOR"),
            baseVariable: String::from("baseVariable"),
            baseVarLogoutBtn: String::from("off"),
            LOGIN: String::from("ログイン"),
            MailAddress: String::from("MailAddress"),
            Password: String::from("Password"),
            WebCaptcha1__editor: String::from("WebCaptcha1__editor"),
            WebCaptcha1__editor_clientState: String::from("WebCaptcha1__editor_clientState"),
            WebCaptcha1_clientState: String::from("[[[[null]],[],[]],[{},[]],null]"),
        };

        let serialized = form_params.serialize_into_sjis();

        assert_eq!(serialized, "\
            __EVENTARGUMENT=__EVENTARGUMENT\
            &__EVENTTARGET=__EVENTTARGET\
            &__VIEWSTATE=__VIEWSTATE\
            &__VIEWSTATEENCRYPTED=__VIEWSTATEENCRYPTED\
            &__VIEWSTATEGENERATOR=__VIEWSTATEGENERATOR\
            &baseVariable=baseVariable\
            &baseVarLogoutBtn=off\
            &LOGIN=%83%8D%83O%83C%83%93\
            &MailAddress=MailAddress\
            &Password=Password\
            &WebCaptcha1__editor=WebCaptcha1__editor\
            &WebCaptcha1__editor_clientState=WebCaptcha1__editor_clientState\
            &WebCaptcha1_clientState=%5B%5B%5B%5Bnull%5D%5D%2C%5B%5D%2C%5B%5D%5D%2C%5B%7B%7D%2C%5B%5D%5D%2Cnull%5D\
        ");
    }

    #[test]
    fn test_mobilesuica_form_params_set_captcha() {
        let mut form_params = MobilesuicaFormParams::default();

        form_params.set_captcha("captcha");

        assert_eq!(form_params.WebCaptcha1__editor, "captcha");
        assert_eq!(
            form_params.WebCaptcha1__editor_clientState,
            "|0|01captcha||[[[[]],[],[]],[{},[]],\"01captcha\"]"
        );
    }

    #[test]
    fn test_mobilesuica_form_params_set_mail_address() {
        let mut form_params = MobilesuicaFormParams::default();

        form_params.set_mail_address("mail_address");

        assert_eq!(form_params.MailAddress, "mail_address");
    }

    #[test]
    fn test_mobilesuica_form_params_set_password() {
        let mut form_params = MobilesuicaFormParams::default();

        form_params.set_password("password");

        assert_eq!(form_params.Password, "password");
    }

    #[test]
    fn test_mobilesuica_form_params_new() {
        let form_params = MobilesuicaFormParams::new(include_str!("../../test/login.html"));

        assert_eq!(form_params.__EVENTARGUMENT, "");
        assert_eq!(form_params.__EVENTTARGET, "");
        assert_eq!(form_params.__VIEWSTATE, "UVC5XJZwUa57v2fAfXWXy67apj73Ua01sHcynBlD7/8pEoMbC+vpKphizN0ZUa4NQ7z3naiVgssqNh4hZndGVHkAkKAJkK3+5x3SAY79V2WO++eW019o644Vj6bOAximDB7kLjWx9kD8D+gXKQVtpKMwyl3cF/MDa9lQ2Zpe9G+wYHeCdnmw4OH1S9QlWazd3Kb0f7NweDEy7yMaZkP263kMp1uPWwUuljNrnKFQWF4eKWKM4LZKsAEpoOm1lYNr1w4ERsqmAORm7UZPU0BJaYiGeMCxrcnJ/MRtYsjYECYLsUx0JLvfOwHLmu3SN5gwKR+IqLvBo3OtZBS8NyQ1RQTCwkByEvjNWReP0daS86QodpoY");
        assert_eq!(form_params.__VIEWSTATEENCRYPTED, "");
        assert_eq!(form_params.__VIEWSTATEGENERATOR, "BB3126B1");
        assert_eq!(form_params.baseVariable, "vuZRB69odJx4bQoQqFjSZBQY-BGYbwiwK~4oOE3Fa7SbHsMZOzL6J1t6UPfqXCsTs8UJHSvvK243G2Fe8d~AasXNQ7lAcxzVHxRMrDAOIIem8olWN4QS~WjkMygJdPB-z");
    }
}
