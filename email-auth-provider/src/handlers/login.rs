#[derive(Debug)]
pub struct Login {
    user_id: ft_sdk::auth::UserId,
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
    let (user_id, user_data) = if payload.username.contains('@') {
        match ft_sdk::auth::provider::user_data_by_email(conn, auth::PROVIDER_ID, &payload.username)
        {
            Ok(v) => v,
            Err(ft_sdk::auth::UserDataError::NoDataFound) => {
                match ft_sdk::auth::provider::user_data_by_verified_email(
                    conn,
                    auth::PROVIDER_ID,
                    &payload.username,
                ) {
                    Ok(v) => v,
                    Err(ft_sdk::auth::UserDataError::NoDataFound) => {
                        ft_sdk::println!("username not found");
                        return Err(ft_sdk::single_error(
                            "username",
                            "Incorrect username/password.",
                        )
                        .into());
                    }
                    Err(e) => return Err(e.into()),
                }
            }
            Err(e) => return Err(e.into()),
        }
    } else {
        ft_sdk::auth::provider::user_data_by_identity(conn, auth::PROVIDER_ID, &payload.username)?
    };

    if !Login::match_password(&user_data, &payload.password)? {
        // we intentionally send the error against username to avoid leaking the fact that the
        // username exists
        ft_sdk::println!("incorrect password");
        return Err(ft_sdk::single_error("username", "Incorrect username/password.").into());
    }

    Ok(Login { user_id })
}

#[derive(serde::Deserialize, Debug)]
struct LoginPayload {
    username: String,
    password: String,
}

#[ft_sdk::form]
pub fn login(
    mut conn: ft_sdk::Connection,
    ft_sdk::Form(payload): ft_sdk::Form<LoginPayload>,
    ft_sdk::Cookie(sid): ft_sdk::Cookie<{ ft_sdk::auth::SESSION_KEY }>,
    host: ft_sdk::Host,
) -> ft_sdk::form::Result {
    let login_meta = validate(&mut conn, payload)?;

    let ft_sdk::auth::SessionID(sid) = ft_sdk::auth::provider::login(
        &mut conn,
        &login_meta.user_id,
        sid.map(ft_sdk::auth::SessionID),
    )?;

    Ok(ft_sdk::form::redirect("/")?.with_cookie(auth::session_cookie(sid.as_str(), host)?))
}
