use ft_sdk::auth::provider as auth_provider;

pub struct Login {
    user_id: ft_sdk::auth::UserId,
    identity: String,
}

impl Login {
    /// Check if the password matches the hashed password in the database
    fn match_password(ud: &ft_sdk::auth::ProviderData, password: &str) -> bool {
        let stored_password: String = match ud.get_custom("hashed_password") {
            Some(v) => v,
            None => return false,
        };

        let parsed_hash = argon2::PasswordHash::new(stored_password.as_str())
            .expect("Expected password to be a valid hash");

        let password_match = argon2::PasswordVerifier::verify_password(
            &argon2::Argon2::default(),
            password.as_bytes(),
            &parsed_hash,
        );

        if password_match.is_ok() {
            return true;
        }


        // User probably has no hashed_password. They can set it via reset
        // password feature if they used some other auth provider
        // (github oauth for example)
        false
    }
}

fn validate(
    conn: &mut ft_sdk::Connection,
    payload: LoginPayload,
) -> Result<Login, ft_sdk::Error> {
    let (user_id, user_data) =
        auth_provider::user_data_by_identity(conn, auth::PROVIDER_ID, &payload.username)?;

    if !Login::match_password(&user_data, &payload.password) {
        return Err(ft_sdk::single_error("password", "incorrect username/password").into());
    }

    Ok(Login { user_id, identity: user_data.identity })
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
        auth::PROVIDER_ID,
        &login_meta.identity,
        &next,
    )?;

    Ok(resp)
}

