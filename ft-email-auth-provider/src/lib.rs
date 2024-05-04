extern crate self as auth;

mod handlers;
mod layout;
mod route;
mod urls;
mod utils;

pub const PROVIDER_ID: &str = "email";

// TODO: login
// TODO: create-account
// TODO: logout

#[no_mangle]
pub extern "C" fn main_ft() {
    let req = ft_sdk::http::current_request();
    let resp = route::route(req);
    ft_sdk::http::send_response(resp);
}
