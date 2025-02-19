#[ft_sdk::form]
// TODO: add a rate limit to this endpoint
pub fn resend_confirmation_email(
    mut conn: ft_sdk::Connection,
    ft_sdk::Query(email): ft_sdk::Query<"email">,
    host: ft_sdk::Host,
    app_url: ft_sdk::AppUrl,
    ft_sdk::Config(config): ft_sdk::Config<crate::Config>,
) -> ft_sdk::form::Result {
    if !validator::ValidateEmail::validate_email(&email) {
        return Err(ft_sdk::single_error("email", "Incorrect email format.").into());
    }

    let (user_id, data) =
        ft_sdk::auth::provider::user_data_by_email(&mut conn, email_auth::PROVIDER_ID, &email)?;

    let conf_link =
        generate_new_confirmation_key(data.clone(), &user_id, &email, &host, app_url, &mut conn)?;

    let name = data.name.unwrap_or_else(|| "User".to_string());

    email_auth::handlers::create_account::send_confirmation_email(
        email, name, &conf_link, &config,
    )?;

    ft_sdk::form::redirect("/")
}

/// Generate a new confirmation key for a given email and update the user table
pub fn generate_new_confirmation_key(
    mut data: ft_sdk::auth::ProviderData,
    user_id: &ft_sdk::auth::UserId,
    email: &str,
    host: &ft_sdk::Host,
    app_url: ft_sdk::AppUrl,
    conn: &mut ft_sdk::Connection,
) -> Result<String, ft_sdk::Error> {
    let key = email_auth::handlers::create_account::generate_key(64);

    let conf_link =
        email_auth::handlers::create_account::confirmation_link(&key, email, host, app_url);

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

    ft_sdk::auth::provider::update_user(
        conn,
        email_auth::PROVIDER_ID,
        user_id,
        data.clone(),
        false,
    )?;

    Ok(conf_link)
}
