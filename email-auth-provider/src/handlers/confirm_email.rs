#[ft_sdk::processor]
pub fn confirm_email(
    mut conn: ft_sdk::Connection,
    ft_sdk::Query(code): ft_sdk::Query<"code">,
    ft_sdk::Query(email): ft_sdk::Query<"email">,
    ft_sdk::Query(next): ft_sdk::Query<"next", Option<String>>,
    host: ft_sdk::Host,
    mountpoint: ft_sdk::AppUrl,
) -> ft_sdk::processor::Result {
    if !validator::ValidateEmail::validate_email(&email) {
        return Err(ft_sdk::single_error("email", "Invalid email format.").into());
    }

    let next = next.unwrap_or_else(|| "/".to_string());
    let (user_id, data) = match ft_sdk::auth::provider::user_data_by_custom_attribute(
        &mut conn,
        email_auth::PROVIDER_ID,
        email_auth::EMAIL_CONF_CODE_KEY,
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
        .get_custom(email_auth::EMAIL_CONF_SENT_AT)
        .expect("email_conf_sent_at should exists if the account was found");

    let sent_at = chrono::DateTime::from_timestamp_nanos(sent_at);

    if key_expired(sent_at) {
        let conf_link =
            email_auth::handlers::resend_confirmation_email::generate_new_confirmation_key(
                data.clone(),
                &user_id,
                &email,
                &host,
                &mountpoint,
                &mut conn,
            )?;

        let name = data.name.unwrap_or_else(|| email.clone());

        email_auth::handlers::create_account::send_confirmation_email(email, name, &conf_link)?;

        return Err(ft_sdk::single_error(
            "code",
            "Confirmation code expired. A new link has been sent to your email address.",
        )
        .into());
    }

    let email = data
        .emails
        .iter()
        .find(|e| **e == email)
        .ok_or_else(|| ft_sdk::single_error("email", "Provided email not found for this user."))?
        .clone();

    let data = {
        let mut data = data;
        data.verified_emails.push(email.clone());
        data.custom
            .as_object_mut()
            .expect("custom is a json object")
            .remove(email_auth::EMAIL_CONF_CODE_KEY);

        data
    };

    ft_sdk::auth::provider::update_user(&mut conn, email_auth::PROVIDER_ID, &user_id, data, false)?;
    ft_sdk::processor::temporary_redirect(next)
}

/// check if it has been 90 days since the verification code was sent. The threshold can be
/// configured using EMAIL_CONFIRMATION_EXPIRE_DAYS env variable
fn key_expired(sent_at: chrono::DateTime<chrono::Utc>) -> bool {
    let expiry_limit_in_days: u64 = ft_sdk::env::var("EMAIL_CONFIRMATION_EXPIRE_DAYS".to_string())
        .map(|v| {
            v.parse()
                .expect("EMAIL_CONFIRMATION_EXPIRE_DAYS should be a number")
        })
        .unwrap_or(90);

    sent_at
        .checked_add_days(chrono::Days::new(expiry_limit_in_days))
        .unwrap()
        <= ft_sdk::env::now()
}
