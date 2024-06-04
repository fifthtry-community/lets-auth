use crate::handlers::create_account::{
    confirm_account_html_template, confirm_account_text_template, confirmation_link,
    email_from_address_from_env, generate_key,
};
use validator::ValidateEmail;

#[ft_sdk::form]
// TODO: add a rate limit to this endpoint
pub fn resend_confirmation_email(
    mut conn: ft_sdk::Connection,
    ft_sdk::Query(email): ft_sdk::Query<"email">,
    host: ft_sdk::Host,
    mountpoint: ft_sdk::Mountpoint,
) -> ft_sdk::form::Result {
    if !email.validate_email() {
        return Err(ft_sdk::single_error("email", "invalid email format").into());
    }

    let (user_id, data) =
        ft_sdk::auth::provider::user_data_by_email(&mut conn, crate::PROVIDER_ID, &email)?;

    let mut data = data;

    let key = generate_key(64);

    let conf_link = confirmation_link(&key, &email, &host, &mountpoint);
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

    let (from_name, from_email) = email_from_address_from_env();

    ft_sdk::println!("Found email sender: {from_name}, {from_email}");

    let name = data.name.unwrap_or("User".to_string());

    if let Err(e) = ft_sdk::send_email(
        &mut conn,
        (&from_name, &from_email),
        vec![(&name, &email)],
        "Confirm you account",
        &confirm_account_html_template(&name, &conf_link),
        &confirm_account_text_template(&name, &conf_link),
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
