use crate::handlers::create_account::CreateAccountPayload;
use ft_sdk::auth::provider as auth_provider;

struct CreateVerifiedAccount {
    email: String,
    #[cfg(feature = "username")]
    username: String,
    name: String,
    hashed_password: String,
    user_id: ft_sdk::UserId,
}

impl CreateVerifiedAccount {
    fn to_provider_data(&self) -> ft_sdk::auth::ProviderData {
        ft_sdk::auth::ProviderData {
            #[cfg(feature = "username")]
            identity: Some(self.username.to_string()),
            #[cfg(not(feature = "username"))]
            identity: self.email.to_string(),
            #[cfg(feature = "username")]
            username: Some(self.username.to_string()),
            #[cfg(not(feature = "username"))]
            username: None,
            name: Some(self.name.to_string()),
            emails: vec![self.email.clone()],
            verified_emails: vec![],
            profile_picture: None,
            custom: serde_json::json!({
                "hashed_password": self.hashed_password
            }),
        }
    }
}

fn validate(
    code: &str,
    payload: CreateAccountPayload,
    conn: &mut ft_sdk::Connection,
) -> Result<CreateVerifiedAccount, ft_sdk::Error> {
    let mut errors = std::collections::HashMap::new();

    payload.validate(conn, &mut errors)?;

    if !errors.is_empty() {
        return Err(ft_sdk::SpecialError::Multi(errors).into());
    }

    // PROVIDER_ID should be SUBSCRIPTION_ID as we are expecting a imported user
    let (user_id, data) = ft_sdk::auth::provider::user_data_by_custom_attribute(
        conn,
        crate::SUBSCRIPTION_PROVIDER,
        // WARN: this matches the key defined in `luma-import.py`
        "confirmation-code",
        code,
    )?;

    // Check if the email is already present in `data -> 'email' -> 'emails'` then
    // check if identity is already created which means user has already an account with the email.
    // If identity is not created this means email is stored because of subscription or other apps.
    if data.identity.is_some() {
        return Err(ft_sdk::single_error("email", "email already exists").into());
    }

    Ok(CreateVerifiedAccount {
        hashed_password: payload.hashed_password(),
        email: payload.email,
        name: payload.name,
        #[cfg(feature = "username")]
        username: payload.username,
        user_id,
    })
}

/// create-verified-account handler to create account for already verified emails
///
/// In regular account creation flow we ask for email, and send a confirmation
/// link to the mail. But sometimes we already have the person in our database,
/// say they are subscribed to our newsletter, we can ask them to "upgrade" their
/// subscription to account. This handler is for that.
///
/// The journey should start with an email we send to the user, with a link to
/// this handler, which includes a verification code. The link will render a page
/// which will fetch user's email from this provider (TODO: add link to handler) and pre-fill the
/// form
#[ft_sdk::form]
pub fn create_verified_account(
    mut conn: ft_sdk::Connection,
    ft_sdk::Query(code): ft_sdk::Query<"code">,
    ft_sdk::Form(payload): ft_sdk::Form<CreateAccountPayload>,
    ft_sdk::Cookie(sid): ft_sdk::Cookie<{ ft_sdk::auth::SESSION_KEY }>,
    host: ft_sdk::Host,
) -> ft_sdk::form::Result {
    let account_meta = validate(&code, payload, &mut conn)?;
    ft_sdk::println!("Account meta done for {}", account_meta.username);

    auth_provider::update_user(
        &mut conn,
        auth::PROVIDER_ID,
        &account_meta.user_id,
        account_meta.to_provider_data(),
        true,
    )?;

    let ft_sdk::auth::SessionID(sid) = auth_provider::login(
        &mut conn,
        &account_meta.user_id,
        sid.map(ft_sdk::auth::SessionID),
    )?;

    ft_sdk::println!("update User done for sid {sid}");

    Ok(ft_sdk::form::redirect("/")?.with_cookie(auth::session_cookie(sid.as_str(), host)?))
}
