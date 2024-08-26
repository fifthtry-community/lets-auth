#[ft_sdk::processor]
pub fn set_password(
    mut conn: ft_sdk::Connection,
    ft_sdk::Required(new_password): ft_sdk::Required<"new-password">,
    ft_sdk::Required(new_password2): ft_sdk::Required<"new-password2">,
    ft_sdk::Query(code): ft_sdk::Query<"code">,
    ft_sdk::Query(spr): ft_sdk::Query<"spr">,
    ft_sdk::Query(email): ft_sdk::Query<"email">,
    ft_sdk::Query(next): ft_sdk::Query<"next", Option<String>>,
    host: ft_sdk::Host,
    mountpoint: ft_sdk::Mountpoint,
) -> ft_sdk::processor::Result {
    validate_email_and_password(&email, &new_password, &new_password2)?;

    let next = next.unwrap_or_else(|| "/".to_string());

    let (user_id, data) = match ft_sdk::auth::provider::user_data_by_custom_attribute(
        &mut conn,
        email_auth::PROVIDER_ID,
        email_auth::PASSWORD_RESET_CODE_KEY,
        &code,
    ) {
        Ok(value) => value,
        Err(ft_sdk::auth::UserDataError::NoDataFound) => {
            return ft_sdk::processor::temporary_redirect(next)
        }
        Err(e) => return Err(e.into()),
    };

    if data.verified_emails.contains(&email) {
        return ft_sdk::processor::temporary_redirect(next);
    }

    let sent_at = data
        .get_custom(email_auth::PASSWORD_RESET_CODE_SENT_AT)
        .expect("PASSWORD_RESET_CODE_SENT_AT should exists if the account was found");
    let sent_at = chrono::DateTime::from_timestamp_nanos(sent_at);

    if key_expired(sent_at) {
        let reset_link = email_auth::handlers::forgot_password::generate_new_reset_key(
            data.clone(),
            &user_id,
            &email,
            spr,
            &host,
            &mountpoint,
            &mut conn,
        )?;

        let name = data.name.unwrap_or_else(|| email.clone());

        email_auth::handlers::forgot_password::send_reset_password_email(
            &mut conn,
            &email,
            &name,
            &reset_link,
        )?;

        return Err(ft_sdk::single_error(
            "code",
            "Confirmation code expired. A new link has been sent to your email address.",
        )
        .into());
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
    ft_sdk::processor::temporary_redirect(next)
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
    email: &str,
    new_password: &str,
    new_password2: &str,
) -> Result<(), ft_sdk::Error> {
    if !validator::ValidateEmail::validate_email(&email) {
        return Err(ft_sdk::single_error("email", "Invalid email format.").into());
    }

    if new_password != new_password2 {
        return Err(ft_sdk::single_error(
            "new-password2".to_string(),
            "Password and Confirm password field do not match.".to_string(),
        )
        .into());
    }

    if let Some(message) = email_auth::handlers::create_account::is_strong_password(&new_password) {
        return Err(ft_sdk::single_error("new-password".to_string(), message).into());
    }

    Ok(())
}
