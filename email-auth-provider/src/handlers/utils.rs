pub(crate) fn user_data_from_email_or_username<S: AsRef<str>>(
    conn: &mut ft_sdk::Connection,
    email_or_username: S,
) -> Result<(ft_sdk::UserId, ft_sdk::auth::ProviderData), ft_sdk::auth::UserDataError> {
    let email_or_username = email_or_username.as_ref();

    if email_or_username.contains('@') {
        match ft_sdk::auth::provider::user_data_by_email(
            conn,
            email_auth::PROVIDER_ID,
            email_or_username,
        ) {
            Ok(v) => return Ok(v),
            Err(ft_sdk::auth::UserDataError::NoDataFound) => {
                return ft_sdk::auth::provider::user_data_by_verified_email(
                    conn,
                    email_auth::PROVIDER_ID,
                    email_or_username,
                );
            }
            Err(e) => return Err(e),
        }
    } else {
        return ft_sdk::auth::provider::user_data_by_identity(
            conn,
            email_auth::PROVIDER_ID,
            email_or_username,
        );
    };
}
