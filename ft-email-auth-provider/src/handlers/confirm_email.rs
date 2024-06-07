#[ft_sdk::form]
pub fn confirm_email(
    mut conn: ft_sdk::Connection,
    ft_sdk::Query(code): ft_sdk::Query<"code">,
    ft_sdk::Query(email): ft_sdk::Query<"email">,
    host: ft_sdk::Host,
    mountpoint: ft_sdk::Mountpoint,
) -> ft_sdk::form::Result {
    if !validator::ValidateEmail::validate_email(&email) {
        return Err(ft_sdk::single_error("email", "invalid email format").into());
    }

    let (user_id, data) = ft_sdk::auth::provider::user_data_by_custom_attribute(
        &mut conn,
        auth::PROVIDER_ID,
        auth::EMAIL_CONF_CODE_KEY,
        &code,
    )?;

    let sent_at = data
        .custom
        .as_object()
        .expect("custom is a json object")
        .get(auth::EMAIL_CONF_SENT_AT)
        .expect("email_conf_sent_at should exists if the account was found")
        .as_str()
        .expect("value must be a datetime string")
        .parse::<chrono::DateTime<chrono::Utc>>()
        .expect("chrono parse must work");

    if key_expired(sent_at) {
        let conf_link = auth::handlers::resend_confirmation_email::generate_new_confirmation_key(
            data.clone(),
            &user_id,
            &email,
            &host,
            &mountpoint,
            &mut conn,
        )?;

        let name = data.name.unwrap_or("User".to_string());

        auth::handlers::resend_confirmation_email::send_confirmation_email(
            &mut conn, &email, &name, &conf_link,
        )?;

        return Err(ft_sdk::single_error(
            "code",
            "Confirmation code expired. A new link has been sent to your email address.",
        )
        .into());
    }

    let email = data
        .clone()
        .emails
        .into_iter()
        .find(|e| *e == email)
        .ok_or_else(|| ft_sdk::single_error("email", "provided email not found for this user"))?;

    let mut data = data;

    data.verified_emails.push(email.clone());

    data.custom
        .as_object_mut()
        .expect("custom is a json object")
        .remove(auth::EMAIL_CONF_CODE_KEY);

    ft_sdk::auth::provider::update_user(&mut conn, auth::PROVIDER_ID, &user_id, data, false)?;

    ft_sdk::form::redirect("/")
}

/// check if it has been 90 days since the verification code was sent. The thresold can be
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
