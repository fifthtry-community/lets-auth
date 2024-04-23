use auth::layout::{Auth, AuthError};

pub struct CreateAccount {
    email: String,
    name: String,
    password: String,
    password2: String,
    accept_terms: bool,
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

        if !validate_email(&email) {
            errors.insert("email".to_string(), "invalid email format".to_string());
        }

        if password != password2 {
            errors.insert(
                "password2".to_string(),
                "password and confirm password field do not match".to_string(),
            );
        }

        if validate_strong_password(&password) {
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

        if ft_sdk::auth::check_email(&email) {
            return Err(AuthError::form_error("email", "email already exists"));
        }

        Ok(Self {
            email,
            name,
            password,
            password2,
            accept_terms,
        })
    }

    fn action(&self, c: &mut Auth) -> Result<ft_sdk::ActionOutput, AuthError>
    where
        Self: Sized,
    {
        // TODO: hash password
        // TODO: call ft_sdk::authenticate() // this will add the info and log them in
        // TODO: figure out sending confirmation emails
        // TODO: redirect to ?next
        todo!()
    }
}

fn validate_strong_password(password: &str) -> bool {
    // TODO:
    true
}

fn validate_email(email: &str) -> bool {
    // TODO:
    true
}
