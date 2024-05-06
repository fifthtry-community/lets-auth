use auth::layout::{Auth, AuthError};

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
            ft_sdk::auth::UserData::Identity(self.name.clone()),
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
    ) -> Result<String, ft_sdk::auth_provider::AuthError> {
        let key = CreateAccount::generate_key(64);

        let data = vec![
            ft_sdk::auth::UserData::Custom {
                key: "conf_code".to_string(),
                value: key.clone().into(),
            },
            ft_sdk::auth::UserData::Name(self.name.clone()),
        ];

        // save the conf link for later use
        ft_sdk::auth_provider::update_user(&user_id, conn, auth::PROVIDER_ID, &self.name, data)?;

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

    fn is_strong_password(_password: &str) -> bool {
        // TODO:
        true
    }

    fn validate_email(_email: &str) -> bool {
        // TODO:
        true
    }
}

impl ft_sdk::Action<Auth, AuthError> for CreateAccount {
    fn validate(c: &mut Auth) -> Result<Self, AuthError>
    where
        Self: Sized,
    {
        use auth::utils::get_required_json_field;
        use ft_sdk::JsonBodyExt;

        let body = c.in_.req.json_body().map_err(|e| {
            AuthError::form_error("payload", format!("invalid payload: {:?}", e).as_str())
        })?;

        // TODO: this can be done with a macro, maybe our version of validator crate in ft-sdk?
        let email = get_required_json_field(&body, "email");
        let name = get_required_json_field(&body, "name");
        let password = get_required_json_field(&body, "password");
        let password2 = get_required_json_field(&body, "password2");
        let accept_terms = body.field::<bool>("accept_terms")?;

        let mut errors = std::collections::HashMap::new();

        if let Err(_) = email {
            errors.insert("email".to_string(), "email is required".to_string());
        }

        if let Err(_) = name {
            errors.insert("name".to_string(), "name is required".to_string());
        }

        if let Err(_) = password {
            errors.insert("password".to_string(), "password is required".to_string());
        }

        if let Err(_) = password2 {
            errors.insert(
                "password2".to_string(),
                "confirm password is required".to_string(),
            );
        }

        if let None = accept_terms {
            errors.insert(
                "accept_terms".to_string(),
                "you must accept the terms and conditions".to_string(),
            );
        }

        if !errors.is_empty() {
            return Err(AuthError::FormError(errors));
        }

        let email = email.unwrap();
        let name = name.unwrap();
        let password = password.unwrap();
        let password2 = password2.unwrap();
        let accept_terms = accept_terms.unwrap();

        if !CreateAccount::validate_email(&email) {
            errors.insert("email".to_string(), "invalid email format".to_string());
        }

        if password != password2 {
            errors.insert(
                "password2".to_string(),
                "password and confirm password field do not match".to_string(),
            );
        }

        if !CreateAccount::is_strong_password(&password) {
            errors.insert("password".to_string(), "password is too weak".to_string());
        }

        if !accept_terms {
            errors.insert(
                "accept_terms".to_string(),
                "you must accept the terms and conditions".to_string(),
            );
        }

        if !errors.is_empty() {
            return Err(AuthError::FormError(errors));
        }

        if ft_sdk::auth_provider::check_if_verified_email_exists(&mut c.conn, &email, None)? {
            return Err(AuthError::form_error("email", "email already exists"));
        }

        let salt = argon2::password_hash::SaltString::generate(&mut ft_sdk::Rng {});

        let argon2 = argon2::Argon2::default();

        let hashed_password = argon2::password_hash::PasswordHasher::hash_password(
            &argon2,
            password.as_bytes(),
            &salt,
        )
        .map_err(|e| AuthError::HashingError(e.to_string()))?
        .to_string();

        Ok(Self {
            email,
            name,
            hashed_password,
        })
    }

    fn action(&self, c: &mut Auth) -> Result<ft_sdk::ActionOutput, AuthError>
    where
        Self: Sized,
    {
        let user_id = ft_sdk::auth_provider::create_user(
            &mut c.conn,
            "email",
            &self.name,
            self.to_provider_data(),
        )
        .map_err(sdk_auth_err_to_auth_err)?;

        ft_sdk::auth_provider::login(&mut c.conn, c.in_.clone(), &user_id, "email", &self.name)?;

        let conf_link = self
            .create_conf_path(&mut c.conn, user_id)
            .map_err(sdk_auth_err_to_auth_err)?;

        if let Err(e) = ft_sdk::send_email(
            (&self.name, &self.email),
            "Confirm you account",
            &mut c.conn,
            &CreateAccount::confirm_account_html(&self.name, &conf_link),
            "auth_confirm_account",
        ) {
            ft_sdk::println!("auth.wasm: failed to queue email: {:?}", e);
        }

        let mut resp_json = std::collections::HashMap::new();

        resp_json.insert("message".to_string(), "account created".into());
        resp_json.insert("success".to_string(), true.into());

        Ok(ft_sdk::ActionOutput::Data(resp_json))
    }
}

fn sdk_auth_err_to_auth_err(e: ft_sdk::auth_provider::AuthError) -> AuthError {
    match e {
        ft_sdk::auth_provider::AuthError::Diesel(e) => AuthError::Diesel(e),
        ft_sdk::auth_provider::AuthError::NameNotProvided => {
            AuthError::form_error("name", "name not provided")
        }
    }
}
