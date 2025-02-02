extern crate self as email_auth;

mod handlers;
mod urls;

pub(crate) use handlers::utils;

pub const PROVIDER_ID: &str = "email";
pub const SUBSCRIPTION_PROVIDER_ID: &str = "subscription";
pub const EMAIL_CONF_CODE_KEY: &str = "email_confirmation_code";
pub const PASSWORD_RESET_CODE_KEY: &str = "password_reset_code";
pub const PASSWORD_RESET_CODE_SENT_AT: &str = "password_reset_code_sent_at";
pub const EMAIL_CONF_SENT_AT: &str = "email_conf_sent_at";

#[derive(serde::Deserialize, Debug)]
pub struct Config {
    pub signin_url: String,
}

pub fn config(
    ft_sdk::Host(host): &ft_sdk::Host,
    ft_sdk::AppUrl(mountpoint): &ft_sdk::AppUrl,
) -> Result<Config, ft_sdk::Error> {
    let url = format!("https://{host}{}config/", mountpoint.as_ref().unwrap());
    ft_sdk::println!("url: {url}");

    let req = http::Request::builder()
        .uri(url)
        .body(bytes::Bytes::new())?;

    let res = ft_sdk::http::send(req).unwrap();

    serde_json::from_slice(res.body()).map_err(|e| e.into())
}
