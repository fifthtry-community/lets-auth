use crate::handlers::create_account::CreateAccount;
use validator::ValidateEmail;

#[ft_sdk::form]
pub fn confirm_email(
    mut conn: ft_sdk::Connection,
    ft_sdk::Query(code): ft_sdk::Query<"code">,
    ft_sdk::Query(email): ft_sdk::Query<"email">,
) -> ft_sdk::form::Result {
    if !email.validate_email() {
        return Err(ft_sdk::single_error("email", "invalid email format").into());
    }

    let (id, data) = ft_sdk::auth::provider::user_data_by_custom_attribute(
        &mut conn,
        crate::PROVIDER_ID,
        crate::EMAIL_CONF_CODE_KEY,
        &code,
    )?;

    let sent_at = data
        .custom
        .as_object()
        .expect("custom is a json object")
        .get(crate::EMAIL_CONF_SENT_AT)
        .expect("email_conf_sent_at should exists if the account was found")
        .as_str()
        .expect("value must be a datetime string")
        .parse::<chrono::DateTime<chrono::Utc>>()
        .expect("chrono parse must work");

    if key_expired(sent_at) {
        return Err(ft_sdk::single_error("code", "confirmation code expired").into());
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
        .remove(crate::EMAIL_CONF_CODE_KEY);

    // remove email from unverified emails
    data.emails = data.emails.into_iter().filter(|e| *e != email).collect();

    ft_sdk::auth::provider::update_user(&mut conn, crate::PROVIDER_ID, &id, data, false)?;

    ft_sdk::form::redirect("/")
}

#[ft_sdk::form]
// TODO: add a rate limit to this endpoint
pub fn resend_confirmation_email(
    mut conn: ft_sdk::Connection,
    ft_sdk::Query(email): ft_sdk::Query<"email">,
    host: ft_sdk::Host,
) -> ft_sdk::form::Result {
    if !email.validate_email() {
        return Err(ft_sdk::single_error("email", "invalid email format").into());
    }

    let (user_id, data) =
        ft_sdk::auth::provider::user_data_by_email(&mut conn, crate::PROVIDER_ID, &email)?;

    let mut data = data;

    let key = CreateAccount::generate_key(64);

    let conf_link = CreateAccount::confirmation_link(&key, &email, &host);
    ft_sdk::println!("Confirmation link added {conf_link}");

    // update user probably does not merge the data. Even if it does, I don't want to a construct a
    // whole ProviderData just to insert some custom key values
    data.custom.as_object_mut().unwrap().insert(
        crate::EMAIL_CONF_CODE_KEY.to_string(),
        serde_json::Value::String(key),
    );

    data.custom.as_object_mut().unwrap().insert(
        crate::EMAIL_CONF_SENT_AT.to_string(),
        serde_json::Value::String(ft_sdk::env::now().to_rfc3339()),
    );

    ft_sdk::auth::provider::update_user(
        &mut conn,
        crate::PROVIDER_ID,
        &user_id,
        data.clone(),
        false,
    )?;

    let (from_name, from_email) = CreateAccount::get_from_address_from_env();

    ft_sdk::println!("Found email sender: {from_name}, {from_email}");

    let name = data.name.unwrap_or("User".to_string());

    if let Err(e) = ft_sdk::send_email(
        &mut conn,
        (&from_name, &from_email),
        vec![(&name, &email)],
        "Confirm you account",
        &CreateAccount::confirm_account_html(&name, &conf_link),
        &CreateAccount::confirm_account_text(&name, &conf_link),
        None,
        None,
        None,
        "auth_confirm_account_request",
    ) {
        ft_sdk::println!("auth.wasm: failed to queue email: {:?}", e);
        return Err(e.into());
    }
    ft_sdk::println!("Email added to the queue");

    ft_sdk::form::redirect("/")
}

/// check if it has been 3 days since the verification code was sent thresold can be configured
/// using EMAIL_CONFIRMATION_EXPIRE_DAYS env variable
fn key_expired(sent_at: chrono::DateTime<chrono::Utc>) -> bool {
    let expiry_limit_in_days: u64 = ft_sdk::env::var("EMAIL_CONFIRMATION_EXPIRE_DAYS".to_string())
        .unwrap_or("3".to_string())
        .parse()
        .expect("EMAIL_CONFIRMATION_EXPIRE_DAYS should be a number");

    sent_at
        .checked_add_days(chrono::Days::new(expiry_limit_in_days))
        .unwrap()
        <= ft_sdk::env::now()
}
