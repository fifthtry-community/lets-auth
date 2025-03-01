#[ft_sdk::form]
pub fn forgot_password(
    mut conn: ft_sdk::Connection,
    ft_sdk::Required(username_or_email): ft_sdk::Required<"username-or-email">,
    ft_sdk::Optional(next): ft_sdk::Optional<"next">,
    app_url: ft_sdk::AppUrl,
    ft_sdk::Config(config): ft_sdk::Config<crate::Config>,
) -> ft_sdk::form::Result {
    let (user_id, email, data) = get_user_data(&mut conn, username_or_email)?;
    let name = data.name.clone().unwrap_or_else(|| email.clone());

    let set_password_url = app_url
        .join("/set-password/")
        .inspect_err(|e| {
            ft_sdk::println!("auth.wasm: failed to join url: {:?}", e);
        })?;

    let reset_link = generate_new_reset_key(data, &user_id, &email, set_password_url, &mut conn)?;

    ft_sdk::println!("======= Password reset link added {reset_link}");

    send_reset_password_email(email, name, &reset_link, &config)?;

    let next = next.unwrap_or_else(|| "/".to_string());
    ft_sdk::form::redirect(next)
}

fn get_user_data(
    conn: &mut ft_sdk::Connection,
    username_or_email: String,
) -> Result<(ft_sdk::UserId, String, ft_sdk::auth::ProviderData), ft_sdk::Error> {
    if username_or_email.contains('@')
        && !validator::ValidateEmail::validate_email(&username_or_email)
    {
        return Err(ft_sdk::single_error("username-or-email", "Incorrect email format.").into());
    }

    let (id, ud) =
        match email_auth::utils::user_data_from_email_or_username(conn, username_or_email) {
            Ok(v) => v,
            Err(ft_sdk::auth::UserDataError::NoDataFound) => {
                return Err(ft_sdk::single_error(
                    "username-or-email",
                    "No account is linked with the provided email",
                )
                .into());
            }
            Err(e) => return Err(e.into()),
        };

    let email = match ud.first_email() {
        Some(e) => e,
        None => {
            return Err(ft_sdk::single_error(
                "username-or-email",
                "No email found for the given user. Password reset email can't be sent.",
            )
            .into())
        }
    };

    Ok((id, email, ud))
}

/// Generate a new password reset key for a given email and update the user table
pub fn generate_new_reset_key(
    mut data: ft_sdk::auth::ProviderData,
    user_id: &ft_sdk::auth::UserId,
    email: &str,
    set_password_url: String,
    conn: &mut ft_sdk::Connection,
) -> ft_sdk::Result<String> {
    let key = ft_sdk::Rng::generate_key(64);

    let reset_link = reset_link(&key, email, set_password_url);

    ft_sdk::println!("Password reset link added {reset_link}");

    // update user probably does not merge the data. Even if it does, I don't want to a construct a
    // whole ProviderData just to insert some custom key values
    data.custom.as_object_mut().unwrap().insert(
        email_auth::PASSWORD_RESET_CODE_KEY.to_string(),
        serde_json::Value::String(key),
    );

    let now = ft_sdk::env::now()
        .timestamp_nanos_opt()
        .expect("unexpected out of range datetime");

    data.custom.as_object_mut().unwrap().insert(
        email_auth::PASSWORD_RESET_CODE_SENT_AT.to_string(),
        serde_json::Value::Number(now.into()),
    );

    ft_sdk::auth::provider::update_user(
        conn,
        email_auth::PROVIDER_ID,
        user_id,
        data.clone(),
        false,
    )?;

    Ok(reset_link)
}

pub fn send_reset_password_email(
    email: String,
    name: String,
    link: &str,
    config: &crate::Config,
) -> Result<(), ft_sdk::Error> {
    let from = config.from_email();

    ft_sdk::println!("Found email sender: {from:?},");

    if let Err(e) = ft_sdk::email::send(&ft_sdk::Email {
        from,
        to: smallvec::smallvec![(name.clone(), email).into()],
        reply_to: Some(smallvec::smallvec![config.reply_to()]),
        cc: Default::default(),
        bcc: Default::default(),
        mkind: "reset-password".to_string(),
        content: ft_sdk::EmailContent::FromMKind {
            context: Some(
                serde_json::json!({
                    "link": link,
                    "name": name,
                })
                .as_object()
                .unwrap()
                .to_owned(),
            ),
        },
    }) {
        ft_sdk::println!("auth.wasm: failed to queue email: {:?}", e);
        return Err(e.into());
    }

    ft_sdk::println!("Email added to the queue");

    Ok(())
}

/// Link to reset password.
pub fn reset_link(key: &str, email: &str, set_password_url: String) -> String {
    format!("{set_password_url}?code={key}&email={email}")
}
