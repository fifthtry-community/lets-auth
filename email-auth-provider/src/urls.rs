// This code is not used anywhere.
// TODO: Remove this when the project achieves feature parity with the old fastn's auth impl
pub(crate) enum Route {
    Login,
    GithubLogin,
    GithubCallback,
    Logout,
    CreateAccount,
    EmailConfirmationSent,
    ConfirmEmail,
    ResendConfirmationEmail,
    Onboarding,
    ForgotPassword,
    ForgotPasswordSuccess,
    SetPassword,
    SetPasswordSuccess,
    Invalid,
}

impl From<&str> for Route {
    fn from(s: &str) -> Self {
        match s {
            "/login/" => Self::Login,
            "/github/" => Self::GithubLogin,
            "/github/callback/" => Self::GithubCallback,
            "/logout/" => Self::Logout,
            "/create-account/" => Self::CreateAccount,
            "/email-confirmation-sent/" => Self::EmailConfirmationSent,
            "/confirm-email/" => Self::ConfirmEmail,
            "/resend-confirmation-email/" => Self::ResendConfirmationEmail,
            "/onboarding/" => Self::Onboarding,
            "/forgot-password/" => Self::ForgotPassword,
            "/forgot-password-success/" => Self::ForgotPasswordSuccess,
            "/set-password/" => Self::SetPassword,
            "/set-password-success/" => Self::SetPasswordSuccess,
            _ => Self::Invalid,
        }
    }
}

impl std::fmt::Display for Route {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Login => write!(f, "/login/"),
            Self::GithubLogin => write!(f, "/github/"),
            Self::GithubCallback => write!(f, "/github/callback/"),
            Self::Logout => write!(f, "/logout/"),
            Self::CreateAccount => write!(f, "/create-account/"),
            Self::EmailConfirmationSent => write!(f, "/email-confirmation-sent/"),
            Self::ConfirmEmail => write!(f, "/confirm-email/"),
            Self::ResendConfirmationEmail => write!(f, "/resend-confirmation-email/"),
            Self::Onboarding => write!(f, "/onboarding/"),
            Self::ForgotPassword => write!(f, "/forgot-password/"),
            Self::ForgotPasswordSuccess => write!(f, "/forgot-password-success/"),
            Self::SetPassword => write!(f, "/set-password/"),
            Self::SetPasswordSuccess => write!(f, "/set-password-success/"),
            Self::Invalid => write!(f, "invalid route"),
        }
    }
}
