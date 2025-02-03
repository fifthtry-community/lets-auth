extern crate self as email_auth;

mod handlers;
mod urls;

use ft_sdk::Error;
pub(crate) use handlers::utils;
use http::Request;
use serde_json::Value;

pub const PROVIDER_ID: &str = "email";
pub const SUBSCRIPTION_PROVIDER_ID: &str = "subscription";
pub const EMAIL_CONF_CODE_KEY: &str = "email_confirmation_code";
pub const PASSWORD_RESET_CODE_KEY: &str = "password_reset_code";
pub const PASSWORD_RESET_CODE_SENT_AT: &str = "password_reset_code_sent_at";
pub const EMAIL_CONF_SENT_AT: &str = "email_conf_sent_at";

#[derive(serde::Deserialize, Debug)]
pub struct Config {
    #[serde(rename = "sign-in-url")]
    pub signin_url: String,
    #[serde(rename = "sign-up-url")]
    pub signup_url: String,
}

impl ft_sdk::FromRequest for Config {
    fn from_request(req: &Request<Value>) -> Result<Self, Error> {
        let scheme = ft_sdk::Scheme::from_request(req)?;
        let host = ft_sdk::Host::from_request(req)?;
        let app_url: ft_sdk::AppUrl = ft_sdk::AppUrl::from_request(req)?;

        let url = app_url.join(&scheme, &host, "config")?;
        ft_sdk::println!("url: {url}");

        let req = http::Request::builder()
            .uri(url)
            .body(bytes::Bytes::new())?;

        let res = ft_sdk::http::send(req).unwrap();

        serde_json::from_slice(res.body()).map_err(|e| e.into())
    }
}
