extern crate self as auth;

mod handlers;
mod urls;

pub const PROVIDER_ID: &str = "email";
pub const DEFAULT_REDIRECT_ROUTE: &str = "/";

// TODO: logout

#[ft_sdk::handle_http]
fn handle(in_: ft_sdk::In, conn: ft_sdk::Connection) -> ft_sdk::http::Result {
    use auth::handlers;
    use auth::urls::Route;

    let mut conn = conn;

    let mut resp: http::Response<bytes::Bytes> = match Into::<Route>::into(in_.req.uri().path()) {
        Route::CreateAccount => handlers::create_account::handle(in_.clone(), &mut conn)?,
        Route::Login => handlers::login::handle(in_.clone(), &mut conn)?,
        Route::Logout => todo!(),
        Route::EmailConfirmationSent => todo!(),
        Route::ConfirmEmail => todo!(),
        Route::ResendConfirmationEmail => todo!(),

        Route::Onboarding => todo!(),

        Route::ForgotPasswordSuccess => todo!(),
        Route::ForgotPassword => todo!(),
        Route::SetPassword => todo!(),
        Route::SetPasswordSuccess => todo!(),

        Route::GithubLogin => todo!(),
        Route::GithubCallback => todo!(),

        Route::Invalid => todo!(),
    }
    .into();

    let cookies = in_.set_cookies.borrow();
    for cookie in cookies.iter() {
        resp.headers_mut().insert(
            http::header::SET_COOKIE,
            cookie.to_string_strict().parse().unwrap(),
        );
    }

    Ok(ft_sdk::http::Output::Http(resp))
}
