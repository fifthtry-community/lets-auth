use ft_sdk::{auth::provider as auth_provider, JsonBodyExt, QueryExt};
use validator::ValidateEmail;

pub struct CreateAccount {
    email: String,
    name: String,
    hashed_password: String,
}

impl CreateAccount {
    fn to_provider_data(&self) -> Vec<ft_sdk::auth::UserData> {
        vec![
            ft_sdk::auth::UserData::Email(self.email.clone()),
            ft_sdk::auth::UserData::Name(self.name.clone()),
            ft_sdk::auth::UserData::Identity(self.email.clone()),
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
        auth_provider::update_user(&user_id, conn, auth::PROVIDER_ID, &self.name, data)?;

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

    fn generate_key(length: usize) -> String {
        ft_sdk::Rng::generate_key(length)
    }

    fn confirmation_link(key: String) -> String {
        format!(
            "{confirm_email_route}?code={key}",
            confirm_email_route = auth::urls::Route::ConfirmEmail,
        )
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
    in_: ft_sdk::In,
    conn: &mut ft_sdk::Connection,
) -> Result<CreateAccount, ft_sdk::http::Error> {
    let email: String = in_.req.required("email")?;
    let name: String = in_.req.required("name")?;
    let password: String = in_.req.required("password")?;
    let password2: String = in_.req.required("password2")?;
    let accept_terms: bool = in_.req.required("accept_terms")?;

    let mut errors = std::collections::HashMap::new();

    if !CreateAccount::validate_email(&email) {
        errors.insert("email".to_string(), "invalid email format".to_string());
    }

    if password != password2 {
        errors.insert(
            "password2".to_string(),
            "password and confirm password field do not match".to_string(),
        );
    }

    if let Some(message) = CreateAccount::is_strong_password(&password, &email, &name) {
        errors.insert("password".to_string(), message);
    }

    if !accept_terms {
        errors.insert(
            "accept_terms".to_string(),
            "you must accept the terms and conditions".to_string(),
        );
    }

    if !errors.is_empty() {
        return Err(ft_sdk::http::Error::Form(errors));
    }

    if auth_provider::check_if_verified_email_exists(conn, &email, None)? {
        return Err(ft_sdk::http::single_error("email", "email already exists"));
    }

    let salt = argon2::password_hash::SaltString::generate(&mut ft_sdk::Rng {});

    let argon2 = argon2::Argon2::default();

    let hashed_password =
        argon2::password_hash::PasswordHasher::hash_password(&argon2, password.as_bytes(), &salt)
            .unwrap()
            .to_string();

    Ok(CreateAccount {
        email,
        name,
        hashed_password,
    })
}

pub fn handle(in_: ft_sdk::In, conn: &mut ft_sdk::Connection) -> ft_sdk::http::Result {
    let account_meta = validate(in_.clone(), conn)?;

    let user_id = auth_provider::create_user(
        conn,
        "email",
        &account_meta.name,
        account_meta.to_provider_data(),
    )
    .map_err(sdk_auth_err_to_http_err)?;

    auth_provider::login(conn, in_.clone(), &user_id, "email", &account_meta.name)?;

    let conf_link = account_meta
        .create_conf_path(conn, user_id)
        .map_err(sdk_auth_err_to_http_err)?;

    if let Err(e) = ft_sdk::send_email(
        (&account_meta.name, &account_meta.email),
        "Confirm you account",
        conn,
        &CreateAccount::confirm_account_html(&account_meta.name, &conf_link),
        "auth_confirm_account",
    ) {
        ft_sdk::println!("auth.wasm: failed to queue email: {:?}", e);
    }

    let query = in_.req.query();

    let next = query.get("next").unwrap_or(auth::DEFAULT_REDIRECT_ROUTE);

    ft_sdk::http::redirect(next)
}

fn sdk_auth_err_to_http_err(e: auth_provider::AuthError) -> ft_sdk::http::Error {
    match e {
        auth_provider::AuthError::Diesel(e) => ft_sdk::http::Error::Diesel(e),
        auth_provider::AuthError::NameNotProvided => {
            ft_sdk::http::single_error("name", "name not provided")
        }
    }
}
