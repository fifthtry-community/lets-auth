pub fn route(r: http::Request<bytes::Bytes>) -> http::Response<bytes::Bytes> {
    use auth::handlers;
    use auth::layout::Auth;
    use auth::urls::Route;
    use ft_sdk::Layout;

    match Into::<Route>::into(r.uri().path()) {
        Route::CreateAccount => Auth::action::<handlers::CreateAccount>(r),
        Route::Login => Auth::action::<handlers::Login>(r),
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
}
