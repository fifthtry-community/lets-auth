extern crate self as auth;

mod handlers;
mod urls;

pub const PROVIDER_ID: &str = "email";
pub const DEFAULT_REDIRECT_ROUTE: &str = "/";
