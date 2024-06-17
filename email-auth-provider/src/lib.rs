extern crate self as email_auth;

mod handlers;
mod urls;

pub const PROVIDER_ID: &str = "email";
pub const SUBSCRIPTION_PROVIDER_ID: &str = "subscription";
pub const EMAIL_CONF_CODE_KEY: &str = "email_confirmation_code";
pub const EMAIL_CONF_SENT_AT: &str = "email_conf_sent_at";
