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
// TODO: make this configurable as well. We need DKIM support among other things before we can do
// this
pub const EMAIL_SENDER: &str = "support@fifthtry.com";

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    email_sender_name: String,
    email_reply_to: String,
}

impl Config {
    pub fn from_email(&self) -> ft_sdk::EmailAddress {
        ft_sdk::EmailAddress {
            name: Some(self.email_sender_name.clone()),
            email: EMAIL_SENDER.to_string(),
        }
    }

    pub fn reply_to(&self) -> ft_sdk::EmailAddress {
        ft_sdk::EmailAddress {
            name: Some(self.email_sender_name.clone()),
            email: self.email_reply_to.clone(),
        }
    }
}

/// Generate https url prefix to reach handlers of this crate
/// path: `/confirm-email`
/// output: https://examplehost.com/-/auth/backend/confirm-email/ (given that the app is
/// mounted on /-/auth/ and the wasm module is named backend)
pub fn wasm_handler_link(
    path: &str,
    ft_sdk::Host(host): &ft_sdk::Host,
    ft_sdk::AppUrl(app_url): ft_sdk::AppUrl,
) -> String {
    let path = path.trim_start_matches('/');
    let path = path.trim_end_matches('/');

    format!(
        "https://{host}{app_url}/backend/{path}/",
        app_url = app_url.unwrap_or_default().trim_end_matches('/'),
    )
}

/// Same as `ft_sdk::Scheme`
/// - https only when host: 127.0.0.1
struct HTTPSScheme(pub ft_sdk::Scheme);

impl ft_sdk::FromRequest for HTTPSScheme {
    fn from_request(req: &http::Request<serde_json::Value>) -> Result<Self, ft_sdk::Error> {
        let host = ft_sdk::Host::from_request(req)?;

        if host.without_port() == "127.0.0.1" {
            Ok(HTTPSScheme(ft_sdk::Scheme::Http))
        } else {
            Ok(HTTPSScheme(ft_sdk::Scheme::Https))
        }
    }
}

impl std::ops::Deref for HTTPSScheme {
    type Target = ft_sdk::Scheme;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
