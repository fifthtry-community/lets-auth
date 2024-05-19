use ft_sdk::auth::provider as auth_provider;
use validator::ValidateEmail;

pub struct CreateAccount {
    email: String,
    // TODO: move it behind a feature flag
    username: String,
    name: String,
    hashed_password: String,
}

impl CreateAccount {
    fn to_provider_data(&self) -> Vec<ft_sdk::auth::UserData> {
        vec![
            ft_sdk::auth::UserData::Email(self.email.clone()),
            ft_sdk::auth::UserData::Name(self.name.clone()),
            ft_sdk::auth::UserData::Identity(self.username.clone()),
            ft_sdk::auth::UserData::Custom {
                key: "hashed_password".to_string(),
                value: self.hashed_password.clone().into(),
            },
        ]
    }

    /// create relative path to the confirm account route handler
    ///
    /// this route needs to be prefixed with the mount point that you used in
    /// your fastn app's url-mappings
    fn create_conf_path(
        &self,
        conn: &mut ft_sdk::Connection,
        user_id: ft_sdk::auth::UserId,
    ) -> Result<String, auth_provider::AuthError> {
        let key = CreateAccount::generate_key(64);

        let data = vec![
            ft_sdk::auth::UserData::Custom {
                key: "conf_code".to_string(),
                value: key.clone().into(),
            },
            ft_sdk::auth::UserData::Name(self.name.clone()),
        ];

        // save the conf link for later use
        auth_provider::update_user(&user_id, conn, auth::PROVIDER_ID, &self.username, data)?;

        Ok(CreateAccount::confirmation_link(key))
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
            name = name,
            link = link,
        )
    }

    fn confirm_account_text(name: &str, link: &str) -> String {
        format!(
            r#"
            Hi {name},

            Click the link below to confirm your account
            {link}

            In case you can't click the link, copy and paste it in your browser.
            "#,
        )
    }

    fn generate_key(length: usize) -> String {
        ft_sdk::Rng::generate_key(length)
    }

    fn confirmation_link(key: String) -> String {
        format!(
            "{confirm_email_route}?code={key}",
            confirm_email_route = auth::urls::Route::ConfirmEmail,
        )
    }

    fn get_from_address_from_env() -> (String, String) {
        let email = ft_sdk::env::var("FASTN_SMTP_SENDER_EMAIL".to_string()).unwrap();
        let name = ft_sdk::env::var("FASTN_SMTP_SENDER_NAME".to_string()).unwrap();

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

    if let Some(message) = CreateAccount::is_strong_password(&payload.password, &payload.email, &payload.name) {
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

    if auth_provider::user_data_by_email(conn, auth::PROVIDER_ID, &payload.email).is_ok() {
        return Err(ft_sdk::single_error("email", "email already exists").into());
    }

    let salt = argon2::password_hash::SaltString::generate(&mut ft_sdk::Rng {});

    let argon2 = argon2::Argon2::default();

    let hashed_password =
        argon2::password_hash::PasswordHasher::hash_password(&argon2, payload.password.as_bytes(), &salt)
            .unwrap()
            .to_string();

    Ok(CreateAccount {
        email: payload.email,
        name: payload.name,
        hashed_password,
        username: payload.username,
    })
}

#[derive(serde::Deserialize)]
struct CreateAccountPayload {
    email: String,
    username: String,
    name: String,
    password: String,
    password2: String,
    accept_terms: bool,
}

#[ft_sdk::form]
pub fn create_account(
    mut conn: ft_sdk::Connection,
    ft_sdk::Form(payload): ft_sdk::Form<CreateAccountPayload>,
    ft_sdk::Query(next): ft_sdk::Query<"next">,
) -> ft_sdk::form::Result {
    let account_meta = validate(payload, &mut conn)?;

    let user_id = auth_provider::create_user(
        &mut conn,
        auth::PROVIDER_ID,
        &account_meta.username,
        account_meta.to_provider_data(),
    )
    .map_err(sdk_auth_err_to_http_err)?;

    let resp = auth_provider::login(&mut conn, &user_id, "email", &account_meta.name, &next)?;

    let conf_link = account_meta
        .create_conf_path(&mut conn, user_id)
        .map_err(sdk_auth_err_to_http_err)?;

    let (from_name, from_email) = CreateAccount::get_from_address_from_env();

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
    }

    Ok(resp)
}

fn sdk_auth_err_to_http_err(e: auth_provider::AuthError) -> ft_sdk::Error {
    match e {
        auth_provider::AuthError::NameNotProvided => {
            ft_sdk::single_error("name", "name not provided").into()
        }
        auth_provider::AuthError::IdentityExists => {
            ft_sdk::single_error("username", "username already exists").into()
        }
        e => e.into()
    }
}
