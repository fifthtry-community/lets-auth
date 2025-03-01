#[ft_sdk::form]
#[allow(clippy::too_many_arguments)]
pub fn set_password(
    mut conn: ft_sdk::Connection,
    ft_sdk::Required(new_password): ft_sdk::Required<"new-password">,
    ft_sdk::Required(new_password2): ft_sdk::Required<"new-password2">,
    ft_sdk::Query(code): ft_sdk::Query<"code", Option<String>>,
    ft_sdk::Query(email): ft_sdk::Query<"email", Option<String>>,
    ft_sdk::Query(next): ft_sdk::Query<"next", Option<String>>,
    app_url: ft_sdk::AppUrl,
    sid: ft_sdk::Cookie<{ ft_sdk::auth::SESSION_KEY }>,
    ft_sdk::Config(config): ft_sdk::Config<crate::Config>,
) -> ft_sdk::form::Result {
    validate_email_and_password(&email, &new_password, &new_password2)?;

    let next = next.unwrap_or_else(|| "/".to_string());

    let (user_id, data) = get_user(&mut conn, sid, code)?;

    let sent_at = data.get_custom(email_auth::PASSWORD_RESET_CODE_SENT_AT);

    if let Some(sent_at) = sent_at {
        let set_password_url = app_url
            .join("/set-password/")
            .inspect_err(|e| {
                ft_sdk::println!("auth.wasm: failed to join url: {:?}", e);
            })?;

        check_expired_and_send_reset_link(
            set_password_url,
            sent_at,
            &data,
            user_id.clone(),
            email,
            &mut conn,
            config,
        )?;
    }

    let data = {
        let mut data = data;

        data.custom
            .as_object_mut()
            .expect("custom is a json object")
            .insert(
                "hashed_password".to_string(),
                serde_json::Value::String(email_auth::handlers::create_account::hashed_password(
                    &new_password,
                )),
            );

        data.custom
            .as_object_mut()
            .expect("custom is a json object")
            .remove(email_auth::PASSWORD_RESET_CODE_KEY);

        data
    };

    ft_sdk::auth::provider::update_user(&mut conn, email_auth::PROVIDER_ID, &user_id, data, false)?;
    ft_sdk::form::redirect(next)
}

/// check if it has been 2 days since the code was sent. The threshold can be
/// configured using RESET_PASSWORD_EXPIRE_DAYS env variable
fn key_expired(sent_at: chrono::DateTime<chrono::Utc>) -> bool {
    let expiry_limit_in_days: u64 = ft_sdk::env::var("RESET_PASSWORD_EXPIRE_DAYS".to_string())
        .map(|v| {
            v.parse()
                .expect("EMAIL_CONFIRMATION_EXPIRE_DAYS should be a number")
        })
        .unwrap_or(2);

    sent_at
        .checked_add_days(chrono::Days::new(expiry_limit_in_days))
        .unwrap()
        <= ft_sdk::env::now()
}

fn validate_email_and_password(
    email: &Option<String>,
    new_password: &str,
    new_password2: &str,
) -> Result<(), ft_sdk::Error> {
    if email.is_some() && !validator::ValidateEmail::validate_email(email.as_ref().unwrap()) {
        return Err(ft_sdk::single_error("email", "Invalid email format.").into());
    }

    if new_password != new_password2 {
        return Err(ft_sdk::single_error(
            "new-password2",
            "Password and Confirm password field do not match.",
        )
        .into());
    }

    if let Some(message) = email_auth::handlers::create_account::is_strong_password(new_password) {
        return Err(ft_sdk::single_error("new-password", message).into());
    }

    Ok(())
}

/// Get logged in user or user with the reset code
fn get_user(
    conn: &mut ft_sdk::Connection,
    sid: ft_sdk::Cookie<{ ft_sdk::auth::SESSION_KEY }>,
    reset_code: Option<String>,
) -> Result<(ft_sdk::UserId, ft_sdk::auth::ProviderData), ft_sdk::Error> {
    let user = ft_sdk::auth::ud(sid, conn).ok().flatten().map(|v| v.id);

    let res = if let Some(user_id) = user {
        // if user is logged in, we can use the user_id to get the user data
        ft_sdk::auth::provider::user_data_by_id(
            conn,
            email_auth::PROVIDER_ID,
            &ft_sdk::UserId(user_id),
        )
        .map(|v| (ft_sdk::UserId(user_id), v))
    } else {
        // if user is not logged in, we can use the code to get the user data
        let reset_code = reset_code
            .ok_or_else(|| ft_sdk::single_error("code", "Invalid reset code and not logged in."))?;

        ft_sdk::auth::provider::user_data_by_custom_attribute(
            conn,
            email_auth::PROVIDER_ID,
            email_auth::PASSWORD_RESET_CODE_KEY,
            &reset_code,
        )
    };

    match res {
        Ok(v) => Ok(v),
        Err(ft_sdk::auth::UserDataError::NoDataFound) => {
            Err(ft_sdk::single_error("code", "Invalid reset code or not logged in.").into())
        }
        Err(e) => Err(e.into()),
    }
}

fn check_expired_and_send_reset_link(
    set_password_url: String,
    sent_at: i64,
    data: &ft_sdk::auth::ProviderData,
    user_id: ft_sdk::UserId,
    email: Option<String>,
    conn: &mut ft_sdk::Connection,
    config: crate::Config,
) -> Result<(), ft_sdk::Error> {
    let sent_at = chrono::DateTime::from_timestamp_nanos(sent_at);

    if !key_expired(sent_at) {
        return Ok(());
    }

    let email = email.ok_or_else(|| {
        ft_sdk::single_error("email", "Email is required if you're not logged in.")
    })?;

    let reset_link = email_auth::handlers::forgot_password::generate_new_reset_key(
        data.clone(),
        &user_id,
        &email,
        set_password_url,
        conn,
    )?;

    let name = data.name.clone().unwrap_or_else(|| email.to_string());

    email_auth::handlers::forgot_password::send_reset_password_email(
        email.to_string(),
        name,
        &reset_link,
        &config,
    )?;

    Err(ft_sdk::single_error(
        "code",
        "Confirmation code expired. A new link has been sent to your email address.",
    )
    .into())
}
