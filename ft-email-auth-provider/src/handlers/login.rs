use ft_sdk::{auth::provider as auth_provider, JsonBodyExt, QueryExt};

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

fn validate(in_: ft_sdk::In, conn: &mut ft_sdk::Connection) -> Result<Login, ft_sdk::http::Error> {
    // TODO: should be able to take username/email
    let username: String = in_.req.required("username")?;
    let password: String = in_.req.required("password")?;

    let (user_id, user_data) =
        auth_provider::user_data_by_identity(conn, auth::PROVIDER_ID, &username)
            .map_err(user_data_error_to_http_err)?;

    if !Login::match_password(&user_data, &password) {
        return Err(ft_sdk::http::single_error(
            "password",
            "incorrect email/password",
        ));
    }

    let identity = Login::get_identity(&user_data).expect(
        "Expected identity to be present in user data. All providers must provide an identity",
    );

    Ok(Login { user_id, identity })
}

pub fn handle(in_: ft_sdk::In, conn: &mut ft_sdk::Connection) -> ft_sdk::http::Result {
    let login_meta = validate(in_.clone(), conn)?;

    auth_provider::login(
        conn,
        in_.clone(),
        &login_meta.user_id,
        "email",
        &login_meta.identity,
    )?;

    let query = in_.req.query();
    let next = query.get("next").unwrap_or(auth::DEFAULT_REDIRECT_ROUTE);

    ft_sdk::http::redirect(next)
}

fn user_data_error_to_http_err(e: auth_provider::UserDataError) -> ft_sdk::http::Error {
    match e {
        auth_provider::UserDataError::NoDataFound => {
            ft_sdk::http::single_error("email", "invalid email")
        }
        auth_provider::UserDataError::DatabaseError(d) => ft_sdk::http::Error::Diesel(d),
    }
}
