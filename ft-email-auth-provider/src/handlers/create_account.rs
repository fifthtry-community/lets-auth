struct CreateAccount {
    email: String,
    #[cfg(feature = "username")]
    username: String,
    name: String,
    hashed_password: String,
    email_confirmation_code: String,
    user_id: Option<ft_sdk::UserId>,
    email_confirmation_sent_at: chrono::DateTime<chrono::Utc>,
}

impl CreateAccount {
    fn to_provider_data(&self) -> ft_sdk::auth::ProviderData {
        let email_sent_at_in_nanos = self
            .email_confirmation_sent_at
            .timestamp_nanos_opt()
            .expect("unexpected out of range datetime");

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
                auth::EMAIL_CONF_SENT_AT: email_sent_at_in_nanos,
                auth::EMAIL_CONF_CODE_KEY: self.email_confirmation_code,
            }),
        }
    }
}

fn validate(
    payload: CreateAccountPayload,
    conn: &mut ft_sdk::Connection,
) -> Result<CreateAccount, ft_sdk::Error> {
    let mut errors = std::collections::HashMap::new();

    payload.validate(conn, &mut errors)?;

    if !errors.is_empty() {
        return Err(ft_sdk::SpecialError::Multi(errors).into());
    }

    // Check if the email is already present in `data -> 'email' -> 'emails'` then
    // check if identity is already created which means user has already an account with the email.
    // If identity is not created this means email is stored because of subscription or other apps.

    use diesel::prelude::*;

    #[derive(diesel::QueryableByName)]
    #[diesel(table_name = ft_sdk::auth::fastn_user)]
    struct Identity {
        identity: Option<String>,
        id: i64,
    }

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
                return Err(ft_sdk::single_error("email", "Email already exists.").into());
            }
            Some(ft_sdk::auth::UserId(identity.id))
        }
        Err(diesel::result::Error::NotFound) => None,
        Err(e) => return Err(e.into()),
    };

    Ok(CreateAccount {
        user_id,
        hashed_password: payload.hashed_password(),
        email: payload.email,
        name: payload.name,
        #[cfg(feature = "username")]
        username: payload.username,
        email_confirmation_code: generate_key(64),
        email_confirmation_sent_at: ft_sdk::env::now(),
    })
}

/// Create account handler, this is available on /create-account/ route
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
    mountpoint: ft_sdk::Mountpoint,
) -> ft_sdk::form::Result {
    let account_meta = validate(payload, &mut conn)?;
    ft_sdk::println!("Account meta done for {}", account_meta.username);

    let uid = match account_meta.user_id.clone() {
        Some(uid) => {
            ft_sdk::auth::provider::update_user(
                &mut conn,
                auth::PROVIDER_ID,
                &uid,
                account_meta.to_provider_data(),
                true,
            )?;
            uid
        }
        None => ft_sdk::auth::provider::create_user(
            &mut conn,
            auth::PROVIDER_ID,
            account_meta.to_provider_data(),
        )?,
    };

    let ft_sdk::auth::SessionID(sid) =
        ft_sdk::auth::provider::login(&mut conn, &uid, sid.map(ft_sdk::auth::SessionID))?;

    ft_sdk::println!("Create User done for sid {sid}");

    let conf_link = confirmation_link(
        &account_meta.email_confirmation_code,
        &account_meta.email,
        &host,
        &mountpoint,
    );
    ft_sdk::println!("Confirmation link added {conf_link}");

    let (from_name, from_email) = email_from_address_from_env();
    ft_sdk::println!("Found email sender: {from_name}, {from_email}");

    if let Err(e) = ft_sdk::send_email(
        &mut conn,
        (&from_name, &from_email),
        vec![(&account_meta.name, &account_meta.email)],
        "Confirm you account",
        &confirm_account_html_template(&account_meta.name, &conf_link),
        &confirm_account_text_template(&account_meta.name, &conf_link),
        None,
        None,
        None,
        "auth_confirm_account_request",
    ) {
        ft_sdk::println!("auth.wasm: failed to queue email: {:?}", e);
        return Err(e.into());
    }
    ft_sdk::println!("Email added to the queue");

    Ok(ft_sdk::form::redirect("/")?.with_cookie(auth::session_cookie(sid.as_str(), host)?))
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

impl CreateAccountPayload {
    pub(crate) fn validate(
        &self,
        conn: &mut ft_sdk::Connection,
        errors: &mut std::collections::HashMap<String, String>,
    ) -> Result<(), ft_sdk::Error> {
        if !validator::ValidateEmail::validate_email(&self.email) {
            errors.insert("email".to_string(), "Invalid email format.".to_string());
        }

        if self.password != self.password2 {
            errors.insert(
                "password2".to_string(),
                "Password and Confirm password field do not match.".to_string(),
            );
        }

        if let Some(message) = self.is_strong_password() {
            errors.insert("password".to_string(), message);
        }

        if !self.accept_terms {
            errors.insert(
                "accept_terms".to_string(),
                "You must accept the terms and conditions.".to_string(),
            );
        }

        #[cfg(feature = "username")]
        {
            validate_identity("username", &self.username, conn, errors)?;
        }

        #[cfg(not(feature = "username"))]
        {
            validate_identity("email", &self.email, conn, errors)?;
        }

        validate_verified_email(&self.email, conn, errors)?;

        Ok(())
    }

    pub(crate) fn hashed_password(&self) -> String {
        let salt = argon2::password_hash::SaltString::generate(&mut ft_sdk::Rng {});
        let argon2 = argon2::Argon2::default();
        argon2::password_hash::PasswordHasher::hash_password(
            &argon2,
            self.password.as_bytes(),
            &salt,
        )
        .unwrap()
        .to_string()
    }

    pub(crate) fn is_strong_password(&self) -> Option<String> {
        // TODO: better password validation
        if self.password.len() < 4 {
            return Some("password is too short".to_string());
        }

        None
    }
}

fn validate_identity(
    field: &str,
    identity: &str,
    conn: &mut ft_sdk::Connection,
    errors: &mut std::collections::HashMap<String, String>,
) -> Result<(), ft_sdk::Error> {
    use diesel::prelude::*;

    if ft_sdk::auth::fastn_user::table
        .select(diesel::dsl::count_star())
        .filter(ft_sdk::auth::fastn_user::identity.eq(identity))
        .get_result::<i64>(conn)?
        > 0
    {
        errors.insert(field.to_string(), "Username already exists.".to_string());
    }

    Ok(())
}

fn validate_verified_email(
    email: &str,
    conn: &mut ft_sdk::Connection,
    errors: &mut std::collections::HashMap<String, String>,
) -> Result<(), ft_sdk::Error> {
    use diesel::prelude::*;

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
    .bind::<diesel::sql_types::Text, _>(email)
    .get_result::<ft_sdk::auth::Counter>(conn)?
    .count
        > 0
    {
        errors.insert("email".to_string(), "Email already exists.".to_string());
    }

    Ok(())
}

pub fn confirm_account_html_template(name: &str, link: &str) -> String {
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

pub fn confirm_account_text_template(name: &str, link: &str) -> String {
    format!(
        r#"
            Hi {name},

            Click the link below to confirm your account:

            {link}

            In case you can't click the link, copy and paste it in your browser.
            "#,
    )
}

pub fn generate_key(length: usize) -> String {
    ft_sdk::Rng::generate_key(length)
}

/// TODO: get mount point of this wasm
pub fn confirmation_link(
    key: &str,
    email: &str,
    ft_sdk::Host(host): &ft_sdk::Host,
    ft_sdk::Mountpoint(mountpoint): &ft_sdk::Mountpoint,
) -> String {
    format!(
        "https://{host}{mountpoint}{confirm_email_route}?code={key}&email={email}",
        confirm_email_route = auth::urls::Route::ConfirmEmail,
        mountpoint = mountpoint.trim_end_matches('/'),
    )
}

pub fn email_from_address_from_env() -> (String, String) {
    let email = ft_sdk::env::var("FASTN_SMTP_SENDER_EMAIL".to_string())
        .unwrap_or_else(|| "support@fifthtry.com".to_string());
    let name = ft_sdk::env::var("FASTN_SMTP_SENDER_NAME".to_string())
        .unwrap_or_else(|| "FifthTry Team".to_string());

    (name, email)
}
