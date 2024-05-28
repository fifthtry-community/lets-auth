use ft_sdk::auth::provider as auth_provider;

#[ft_sdk::data]
pub fn confirm_email(
    mut conn: ft_sdk::Connection,
    ft_sdk::Query(code): ft_sdk::Query<"code">,
    ft_sdk::Query(email): ft_sdk::Query<"email">,
) -> ft_sdk::data::Result {

    ft_sdk::println!("code: {code}, email: {email}");
    let (user_id, mut provider_data) = match auth_provider::user_data_by_email(&mut conn, auth::PROVIDER_ID, email.as_str()) {
        Ok(u) => u,
        Err(ft_sdk::auth::UserDataError::NoDataFound) => {
            ft_sdk::println!("username not found");
            return Err(ft_sdk::single_error(
                "username",
                "incorrect username/password",
            )
                .into());
        }
        Err(e) => return Err(e.into()),
    };
    let custom: auth::Custom = serde_json::from_value(provider_data.custom.clone()).unwrap();

    if custom.email_confirmation_code.eq(&code) {
        if !provider_data.verified_emails.contains(&email) {
            provider_data.verified_emails.push(email);
            let ft_sdk::auth::UserId(_) = auth_provider::update_user(
                &mut conn,
                &user_id,
                auth::PROVIDER_ID,
                provider_data.clone(),
            )?;
            ft_sdk::data::api_ok("Email Verified.")
        } else {
            ft_sdk::data::api_ok("Email Already Verified.")
        }
    } else {
        ft_sdk::data::api_error(std::collections::HashMap::from([("code".to_string(), "Code Mismatch.".to_string())]))
    }
}