extern crate self as auth;

use cookie::time::{Duration, OffsetDateTime};

mod handlers;
mod urls;

pub const PROVIDER_ID: &str = "email";
pub const DEFAULT_REDIRECT_ROUTE: &str = "/";

pub fn session_cookie(sid: &str, host: ft_sdk::Host) -> Result<http::HeaderValue, ft_sdk::Error> {
    // DO NOT CHANGE THINGS HERE, consult logout code in fastn.
    let mut cookie = cookie::Cookie::build(
        (ft_sdk::auth::SESSION_KEY,
        sid,
        ))
        .domain(host.without_port())
        .path("/")
        .max_age(Duration::seconds(34560000))
        .same_site(cookie::SameSite::Strict)
        .build();

    Ok(http::HeaderValue::from_str(cookie.to_string().as_str())?)
}