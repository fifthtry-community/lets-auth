/// get a required string value for a json field or a form error
pub fn get_required_json_field(
    body: &ft_sdk::JsonBody,
    key: &str,
) -> Result<String, auth::layout::AuthError> {
    let val = body.field::<String>(key)?.ok_or_else(|| {
        auth::layout::AuthError::form_error(key, format!("{} is required", key).as_str())
    })?;

    if val.is_empty() {
        return Err(auth::layout::AuthError::form_error(
            key,
            format!("{} is required", key).as_str(),
        ));
    }

    Ok(val)
}
