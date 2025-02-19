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
