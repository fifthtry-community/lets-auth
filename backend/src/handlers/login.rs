#[derive(Debug)]
pub struct Login {
    user_id: ft_sdk::auth::UserId,
}

#[ft_sdk::form]
pub fn login(
    mut conn: ft_sdk::Connection,
    ft_sdk::Form(payload): ft_sdk::Form<LoginPayload>,
    ft_sdk::Query(next): ft_sdk::Query<"next", Option<String>>,
    ft_sdk::Cookie(sid): ft_sdk::Cookie<{ ft_sdk::auth::SESSION_KEY }>,
    host: ft_sdk::Host,
) -> ft_sdk::form::Result {
    let login_meta = validate(&mut conn, payload)?;

    let ft_sdk::SessionID(sid) =
        ft_sdk::auth::provider::login(&mut conn, &login_meta.user_id, sid.map(ft_sdk::SessionID))?;

    let next = next.unwrap_or_else(|| "/".to_string());
    Ok(ft_sdk::form::redirect(next)?.with_cookie(common::session_cookie(sid.as_str(), host)?))
}

impl Login {
    /// Check if the password matches the hashed password in the database
    fn match_password(
        ud: &ft_sdk::auth::ProviderData,
        password: &str,
    ) -> Result<bool, ft_sdk::Error> {
        ft_sdk::println!("ud: {ud:?}");
        let stored_password: String = match ud.get_custom("hashed_password") {
            Some(v) => v,
            None => {
                ft_sdk::println!("no hashed password found");
                return Ok(false);
            }
        };

        let parsed_hash = match argon2::PasswordHash::new(stored_password.as_str()) {
            Ok(v) => v,
            Err(e) => {
                ft_sdk::println!("error parsing hash: {:?}", e);
                return Err(ft_sdk::server_error!("error verifying password: {:?}", e).into());
            }
        };

        let password_match = argon2::PasswordVerifier::verify_password(
            &argon2::Argon2::default(),
            password.as_bytes(),
            &parsed_hash,
        );

        match password_match {
            Ok(()) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(ft_sdk::server_error!("error verifying password: {:?}", e).into()),
        }
    }
}

fn validate(conn: &mut ft_sdk::Connection, payload: LoginPayload) -> Result<Login, ft_sdk::Error> {
    let (user_id, user_data) = match email_auth::utils::user_data_from_email_or_username(
        conn,
        payload.username_or_email,
    ) {
        Ok(v) => v,
        Err(ft_sdk::auth::UserDataError::NoDataFound) => {
            ft_sdk::println!("username not found");
            return Err(
                ft_sdk::single_error("username-or-email", "Incorrect username/password.").into(),
            );
        }
        Err(e) => return Err(e.into()),
    };

    if !Login::match_password(&user_data, &payload.password)? {
        // we intentionally send the error against username to avoid leaking the fact that the
        // username exists
        ft_sdk::println!("incorrect password");
        return Err(
            ft_sdk::single_error("username-or-email", "Incorrect username/password.").into(),
        );
    }

    Ok(Login { user_id })
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct LoginPayload {
    username_or_email: String,
    password: String,
}
