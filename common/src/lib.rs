pub fn validate_identity(
    field: &str,
    identity: &str,
    conn: &mut ft_sdk::Connection,
    errors: &mut std::collections::HashMap<String, String>,
) -> Result<(), ft_sdk::Error> {
    use diesel::prelude::*;

    if ft_sdk::auth::fastn_user::table
        .select(diesel::dsl::count_star())
        .filter(ft_sdk::auth::fastn_user::identity.eq(identity))
        .get_result::<i64>(conn)?
        > 0
    {
        errors.insert(field.to_string(), "Username already exists.".to_string());
    }

    Ok(())
}

pub fn session_cookie(sid: &str, host: ft_sdk::Host) -> Result<http::HeaderValue, ft_sdk::Error> {
    // DO NOT CHANGE THINGS HERE, consult logout code in fastn.
    let cookie = cookie::Cookie::build((ft_sdk::auth::SESSION_KEY, sid))
        .domain(host.without_port())
        .path("/")
        .max_age(cookie::time::Duration::seconds(34560000))
        .same_site(cookie::SameSite::Strict)
        .build();

    Ok(http::HeaderValue::from_str(cookie.to_string().as_str())?)
}
