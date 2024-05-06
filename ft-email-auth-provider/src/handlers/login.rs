use auth::layout::{Auth, AuthError};
use ft_sdk::auth::provider as auth_provider;

pub struct Login {
    user_id: ft_sdk::auth::UserId,
    identity: String,
}

impl Login {
    /// Check if the password matches the hashed password in the database
    fn match_password(ud: &Vec<ft_sdk::auth::UserData>, password: &str) -> bool {
        for user_data in ud {
            if let ft_sdk::auth::UserData::Custom { key, value } = user_data {
                if key == "hashed_password" {
                    let stored_password = value.as_str().expect("Expected password to be a string");

                    let parsed_hash = argon2::PasswordHash::new(stored_password)
                        .expect("Expected password to be a valid hash");

                    let password_match = argon2::PasswordVerifier::verify_password(
                        &argon2::Argon2::default(),
                        password.as_bytes(),
                        &parsed_hash,
                    );

                    if password_match.is_ok() {
                        return true;
                    }
                }
            }
        }

        // User probably has no hashed_password. They can set it via reset
        // password feature if they used some other auth provider
        // (github oauth for example)
        false
    }

    fn get_identity(ud: &Vec<ft_sdk::auth::UserData>) -> Option<String> {
        for user_data in ud {
            if let ft_sdk::auth::UserData::Identity(identity) = user_data {
                return Some(identity.clone());
            }
        }

        None
    }
}

impl ft_sdk::Action<Auth, AuthError> for Login {
    fn validate(c: &mut Auth) -> Result<Self, AuthError>
    where
        Self: Sized,
    {
        use auth::utils::get_required_json_field;
        use ft_sdk::JsonBodyExt;

        let body = c.in_.req.json_body().map_err(|e| {
            AuthError::form_error("payload", format!("invalid payload: {:?}", e).as_str())
        })?;

        let mut errors = std::collections::HashMap::new();

        let email = get_required_json_field(&body, "email");
        let password = get_required_json_field(&body, "password");

        if let Err(_) = email {
            errors.insert("email".into(), "email is required".into());
        }

        if let Err(_) = password {
            errors.insert("password".into(), "password is required".into());
        }

        if !errors.is_empty() {
            return Err(AuthError::FormError(errors));
        }

        let email = email.unwrap();
        let password = password.unwrap();

        let (user_id, user_data) =
            auth_provider::get_user_data_by_email(&mut c.conn, auth::PROVIDER_ID, &email)
                .map_err(user_data_error_to_auth_err)?;

        if !Login::match_password(&user_data, &password) {
            return Err(AuthError::form_error(
                "password",
                "incorrect email/password",
            ));
        }

        let identity = Login::get_identity(&user_data).expect(
            "Expected identity to be present in user data. All providers must provide an identity",
        );

        Ok(Login { user_id, identity })
    }

    fn action(&self, c: &mut Auth) -> Result<ft_sdk::ActionOutput, AuthError>
    where
        Self: Sized,
    {
        auth_provider::login(
            &mut c.conn,
            c.in_.clone(),
            &self.user_id,
            "email",
            &self.identity,
        )?;

        let mut resp_json = std::collections::HashMap::new();

        resp_json.insert("message".to_string(), "login successfull".into());
        resp_json.insert("success".to_string(), true.into());

        Ok(ft_sdk::ActionOutput::Data(resp_json))
    }
}

fn user_data_error_to_auth_err(e: auth_provider::UserDataError) -> AuthError {
    match e {
        auth_provider::UserDataError::NoDataFound => {
            AuthError::form_error("email", "invalid email")
        }
        auth_provider::UserDataError::DatabaseError(d) => AuthError::Diesel(d),
    }
}
