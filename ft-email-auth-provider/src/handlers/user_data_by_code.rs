#[derive(serde::Serialize)]
struct PreFillData {
    email: String,
}

/// Get user data from `code`. This code is sent to user's email for promotions.
#[ft_sdk::data]
fn user_data_by_code(
    mut conn: ft_sdk::Connection,
    ft_sdk::Query(code): ft_sdk::Query<"code">,
) -> ft_sdk::data::Result {
    let (_, data) = ft_sdk::auth::provider::user_data_by_custom_attribute(
        &mut conn,
        crate::SUBSCRIPTION_PROVIDER_ID,
        "confirmation-code",
        &code,
    )?;

    ft_sdk::println!("got data for pre filling: {:?}", data);

    let resp = PreFillData {
        email: data
            .emails
            .first()
            .cloned()
            .expect("at least one email must exist for imported users"),
    };

    ft_sdk::data::json(resp)
}
