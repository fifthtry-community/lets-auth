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

fn validate(
    conn: &mut ft_sdk::Connection,
    payload: LoginPayload,
) -> Result<Login, ft_sdk::Error> {
    let (user_id, user_data) =
        auth_provider::user_data_by_identity(conn, auth::PROVIDER_ID, &payload.username)
            .map_err(user_data_error_to_http_err)?;

    if !Login::match_password(&user_data, &payload.password) {
        return Err(ft_sdk::single_error("password", "incorrect email/password").into());
    }

    let identity = Login::get_identity(&user_data).expect(
        "Expected identity to be present in user data. All providers must provide an identity",
    );

    Ok(Login { user_id, identity })
}

#[derive(serde::Deserialize)]
struct LoginPayload {
    username: String,
    password: String,
}

#[ft_sdk::form]
pub fn login(
    mut conn: ft_sdk::Connection,
    ft_sdk::Form(payload): ft_sdk::Form<LoginPayload>,
    ft_sdk::Query(next): ft_sdk::Query<"next">,
) -> ft_sdk::form::Result {
    let login_meta = validate(&mut conn, payload)?;

    let resp = auth_provider::login(
        &mut conn,
        &login_meta.user_id,
        "email",
        &login_meta.identity,
        &next,
    )?;

    Ok(resp)
}

fn user_data_error_to_http_err(e: auth_provider::UserDataError) -> ft_sdk::Error {
    match e {
        auth_provider::UserDataError::NoDataFound => ft_sdk::single_error("email", "invalid email").into(),
        auth_provider::UserDataError::DatabaseError(d) => d.into(),
    }
}
