use ft_sdk::auth::provider as auth_provider;
use validator::ValidateEmail;

pub struct CreateAccount {
    email: String,
    #[cfg(feature = "username")]
    username: String,
    name: String,
    hashed_password: String,
    email_confirmation_code: String,
    user_id: Option<ft_sdk::UserId>,
}

impl CreateAccount {
    fn to_provider_data(&self) -> ft_sdk::auth::ProviderData {
        ft_sdk::auth::ProviderData {
            #[cfg(feature = "username")]
            identity: self.username.to_string(),
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
                "hashed_password": self.hashed_password,
                "email_confirmation_code": self.email_confirmation_code
            }),
        }
    }

    fn confirm_account_html(name: &str, link: &str) -> String {
        // TODO: until we figure out email templates, this has to do
        format!(
            r#"
            <html>
                <head>
                    <title>Confirm your account</title>
                </head>
                <body>
                    <h1>Hi {name},</h1>
                    <p>Click the link below to confirm your account</p>
                    <a href="{link}">Confirm your account</a>

                    In case you can't click the link, copy and paste the following link in your browser:
                    <br>
                    <a href="{link}">{link}</a>
                </body>
            </html>
            "#,
        )
    }

    fn confirm_account_text(name: &str, link: &str) -> String {
        format!(
            r#"
            Hi {name},

            Click the link below to confirm your account:

            {link}

            In case you can't click the link, copy and paste it in your browser.
            "#,
        )
    }

    fn generate_key(length: usize) -> String {
        ft_sdk::Rng::generate_key(length)
    }

    fn confirmation_link(&self, ft_sdk::Host(host): &ft_sdk::Host) -> String {
        format!(
            "https://{host}{confirm_email_route}?code={key}",
            key = self.email_confirmation_code,
            confirm_email_route = auth::urls::Route::ConfirmEmail,
        )
    }

    fn get_from_address_from_env() -> (String, String) {
        let email = ft_sdk::env::var("FASTN_SMTP_SENDER_EMAIL".to_string())
            .unwrap_or_else(|| "support@fifthtry.com".to_string());
        let name = ft_sdk::env::var("FASTN_SMTP_SENDER_NAME".to_string())
            .unwrap_or_else(|| "FifthTry Team".to_string());

        (name, email)
    }

    fn is_strong_password(password: &str, _email: &str, _name: &str) -> Option<String> {
        // TODO: better password validation
        if password.len() < 4 {
            return Some("password is too short".to_string());
        }

        None
    }

    fn validate_email(email: &str) -> bool {
        email.validate_email()
    }
}

fn validate(
    payload: CreateAccountPayload,
    conn: &mut ft_sdk::Connection,
) -> Result<CreateAccount, ft_sdk::Error> {
    let mut errors = std::collections::HashMap::new();

    if !CreateAccount::validate_email(&payload.email) {
        errors.insert("email".to_string(), "invalid email format".to_string());
    }

    if payload.password != payload.password2 {
        errors.insert(
            "password2".to_string(),
            "password and confirm password field do not match".to_string(),
        );
    }

    if let Some(message) =
        CreateAccount::is_strong_password(&payload.password, &payload.email, &payload.name)
    {
        errors.insert("password".to_string(), message);
    }

    if !payload.accept_terms {
        errors.insert(
            "accept_terms".to_string(),
            "you must accept the terms and conditions".to_string(),
        );
    }

    if !errors.is_empty() {
        return Err(ft_sdk::SpecialError::Multi(errors).into());
    }

    use diesel::prelude::*;
    use ft_sdk::auth::fastn_user;

    if fastn_user::table
        .select(diesel::dsl::count_star())
        .filter(fastn_user::identity.eq({
            #[cfg(feature = "username")]
            {
                &payload.username
            }
            #[cfg(not(feature = "username"))]
            {
                &payload.email
            }
        }))
        .get_result::<i64>(conn)?
        > 0
    {
        return Err(ft_sdk::single_error("username", "username already exists").into());
    }

    if diesel::sql_query(
        r#"
        SELECT
            COUNT(*) AS count
        FROM fastn_user
        WHERE
            EXISTS (
                SELECT 1
                FROM json_each(data -> 'email' -> 'verified_emails')
                WHERE value = $1
            )
    "#,
    )
    .bind::<diesel::sql_types::Text, _>(&payload.email)
    .get_result::<ft_sdk::auth::Counter>(conn)?
    .count
        > 0
    {
        return Err(ft_sdk::single_error("email", "email already exists").into());
    }

    #[derive(diesel::QueryableByName)]
    #[diesel(table_name = fastn_user)]
    struct Identity {
        identity: Option<String>,
        id: i64,
    }

    // Check if the email is already present in `data -> 'email' -> 'emails'` then
    // check if identity is already created which means user has already an account with the email.
    // If identity is not created this means email is stored because of subscription or other apps.
    let user_id = match diesel::sql_query(
        r#"
            SELECT
                id, identity
            FROM fastn_user
            WHERE
                EXISTS (
                    SELECT 1
                    FROM json_each ( data -> 'email' -> 'emails' )
                    WHERE value = $1
                )
        "#,
    )
    .bind::<diesel::sql_types::Text, _>(&payload.email)
    .get_result::<Identity>(conn)
    {
        Ok(identity) => {
            if identity.identity.is_some() {
                return Err(ft_sdk::single_error("email", "email already exists").into());
            }
            Some(identity.id)
        }
        Err(diesel::result::Error::NotFound) => None,
        Err(e) => return Err(e.into()),
    };

    let salt = argon2::password_hash::SaltString::generate(&mut ft_sdk::Rng {});

    let argon2 = argon2::Argon2::default();

    let hashed_password = argon2::password_hash::PasswordHasher::hash_password(
        &argon2,
        payload.password.as_bytes(),
        &salt,
    )
    .unwrap()
    .to_string();

    Ok(CreateAccount {
        email: payload.email,
        name: payload.name,
        hashed_password,
        #[cfg(feature = "username")]
        username: payload.username,
        email_confirmation_code: CreateAccount::generate_key(64),
        user_id: user_id.map(ft_sdk::UserId),
    })
}

#[derive(serde::Deserialize)]
struct CreateAccountPayload {
    email: String,
    #[cfg(feature = "username")]
    username: String,
    name: String,
    password: String,
    password2: String,
    accept_terms: bool,
}

/// Create account handler, this is available on /create_account/ route
///
/// If a user does not exist for the given username and email, we create it.
///
/// It can happen that a user exists for the given email, but has no identity
/// (identity is empty string). This can happen if you have imported email
/// subscribers to the user table.
///
/// When importing, make sure that only unverified email is added in the
/// email provider data, nothing else. E.g. `data -> 'email'` should only
/// contain `{ "emails": ["email@being-imported.com"] }`, all other
/// subscriber data, e.g. if there is double opt-in, or the `name` of user,
/// `tags` for the user should be stored in any other `data` key (`data -> 'subscriptions')
#[ft_sdk::form]
pub fn create_account(
    mut conn: ft_sdk::Connection,
    ft_sdk::Form(payload): ft_sdk::Form<CreateAccountPayload>,
    ft_sdk::Cookie(sid): ft_sdk::Cookie<{ ft_sdk::auth::SESSION_KEY }>,
    host: ft_sdk::Host,
) -> ft_sdk::form::Result {
    let account_meta = validate(payload, &mut conn)?;
    ft_sdk::println!("Account meta done for {}", account_meta.username);

    let uid = match account_meta.user_id.clone() {
        Some(uid) => {
            auth_provider::update_user(
                &mut conn,
                auth::PROVIDER_ID,
                &uid,
                account_meta.to_provider_data(),
                true,
            )?;
            uid
        }
        None => auth_provider::create_user(
            &mut conn,
            auth::PROVIDER_ID,
            account_meta.to_provider_data(),
        )?,
    };

    let ft_sdk::auth::SessionID(sid) =
        auth_provider::login(&mut conn, &uid, sid.map(ft_sdk::auth::SessionID))?;

    ft_sdk::println!("Create User done for sid {sid}");

    let conf_link = account_meta.confirmation_link(&host);
    ft_sdk::println!("Confirmation link added {conf_link}");

    let (from_name, from_email) = CreateAccount::get_from_address_from_env();
    ft_sdk::println!("Found name and email: {from_name}, {from_email}");

    if let Err(e) = ft_sdk::send_email(
        &mut conn,
        (&from_name, &from_email),
        vec![(&account_meta.name, &account_meta.email)],
        "Confirm you account",
        &CreateAccount::confirm_account_html(&account_meta.name, &conf_link),
        &CreateAccount::confirm_account_text(&account_meta.name, &conf_link),
        None,
        None,
        None,
        "auth_confirm_account",
    ) {
        ft_sdk::println!("auth.wasm: failed to queue email: {:?}", e);
        return Err(e.into());
    }
    ft_sdk::println!("Email added to the queue");

    Ok(ft_sdk::form::redirect("/")?.with_cookie(auth::session_cookie(sid.as_str(), host)?))
}
