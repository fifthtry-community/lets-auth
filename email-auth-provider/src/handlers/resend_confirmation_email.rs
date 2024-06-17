#[ft_sdk::form]
// TODO: add a rate limit to this endpoint
pub fn resend_confirmation_email(
    mut conn: ft_sdk::Connection,
    ft_sdk::Query(email): ft_sdk::Query<"email">,
    host: ft_sdk::Host,
    mountpoint: ft_sdk::Mountpoint,
) -> ft_sdk::form::Result {
    if !validator::ValidateEmail::validate_email(&email) {
        return Err(ft_sdk::single_error("email", "Incorrect email format.").into());
    }

    let (user_id, data) =
        ft_sdk::auth::provider::user_data_by_email(&mut conn, email_auth::PROVIDER_ID, &email)?;

    let conf_link = generate_new_confirmation_key(
        data.clone(),
        &user_id,
        &email,
        &host,
        &mountpoint,
        &mut conn,
    )?;

    let name = data.name.unwrap_or_else(|| "User".to_string());

    send_confirmation_email(&mut conn, &email, &name, &conf_link)?;

    ft_sdk::form::redirect("/")
}

/// Generate a new confirmation key for a given email and update the user table
pub fn generate_new_confirmation_key(
    mut data: ft_sdk::auth::ProviderData,
    user_id: &ft_sdk::auth::UserId,
    email: &str,
    host: &ft_sdk::Host,
    mountpoint: &ft_sdk::Mountpoint,
    conn: &mut ft_sdk::Connection,
) -> Result<String, ft_sdk::Error> {
    let key = email_auth::handlers::create_account::generate_key(64);

    let conf_link =
        email_auth::handlers::create_account::confirmation_link(&key, email, host, mountpoint);

    ft_sdk::println!("Confirmation link added {conf_link}");

    // update user probably does not merge the data. Even if it does, I don't want to a construct a
    // whole ProviderData just to insert some custom key values
    data.custom.as_object_mut().unwrap().insert(
        email_auth::EMAIL_CONF_CODE_KEY.to_string(),
        serde_json::Value::String(key),
    );

    data.custom.as_object_mut().unwrap().insert(
        email_auth::EMAIL_CONF_SENT_AT.to_string(),
        serde_json::Value::String(ft_sdk::env::now().to_rfc3339()),
    );

    ft_sdk::auth::provider::update_user(conn, email_auth::PROVIDER_ID, user_id, data.clone(), false)?;

    Ok(conf_link)
}

pub fn send_confirmation_email(
    conn: &mut ft_sdk::Connection,
    email: &str,
    name: &str,
    conf_link: &str,
) -> Result<(), ft_sdk::Error> {
    let (from_name, from_email) = email_auth::handlers::create_account::email_from_address_from_env();

    ft_sdk::println!("Found email sender: {from_name}, {from_email}");

    if let Err(e) = ft_sdk::send_email(
        conn,
        (&from_name, &from_email),
        vec![(name, email)],
        "Confirm you account",
        &email_auth::handlers::create_account::confirm_account_html_template(name, conf_link),
        &email_auth::handlers::create_account::confirm_account_text_template(name, conf_link),
        None,
        None,
        None,
        "auth_confirm_account_request",
    ) {
        ft_sdk::println!("auth.wasm: failed to queue email: {:?}", e);
        return Err(e.into());
    }

    ft_sdk::println!("Email added to the queue");

    Ok(())
}
