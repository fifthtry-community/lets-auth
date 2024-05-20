use ft_sdk::auth::{provider as auth_provider};
use validator::ValidateEmail;

pub struct CreateAccount {
    email: String,
    // TODO: move it behind a feature flag
    username: String,
    name: String,
    hashed_password: String,
    email_confirmation_code: String,
}

impl CreateAccount {
    fn to_provider_data(&self) -> ft_sdk::auth::ProviderData {
        ft_sdk::auth::ProviderData {
            identity: self.username.to_string(),
            username: Some(self.username.to_string()),
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

    fn confirmation_link(&self) -> String {
        format!(
            "{confirm_email_route}?code={key}",
            key = self.email_confirmation_code,
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
        email_confirmation_code: CreateAccount::generate_key(64)
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
) -> ft_sdk::form::Result {
    let account_meta = validate(payload, &mut conn)?;

    auth_provider::create_user(
        &mut conn,
        None,
        auth::PROVIDER_ID,
        &account_meta.username,
        account_meta.to_provider_data(),
    )?;

    let conf_link = account_meta.confirmation_link();


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
        return Err(e.into());
    }

    ft_sdk::form::redirect("/")
}

