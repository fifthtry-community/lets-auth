#[ft_sdk::form]
pub fn create_account(
    mut conn: ft_sdk::Connection,
    ft_sdk::Form(payload): ft_sdk::Form<CreateAccountPayload>,
    ft_sdk::Cookie(sid): ft_sdk::Cookie<{ ft_sdk::auth::SESSION_KEY }>,
    host: ft_sdk::Host,
) -> ft_sdk::form::Result {
    let account_meta = validate_and_account_meta(payload, &mut conn)?;
    ft_sdk::println!("Account meta done for {}", account_meta.username);

    let uid = match account_meta.user_id.clone() {
        Some(uid) => {
            ft_sdk::auth::provider::update_user(
                &mut conn,
                mobile_auth::PROVIDER_ID,
                &uid,
                account_meta.to_provider_data(),
                true,
            )?;
            uid
        }
        None => ft_sdk::auth::provider::create_user(
            &mut conn,
            mobile_auth::PROVIDER_ID,
            account_meta.to_provider_data(),
        )?,
    };

    let ft_sdk::auth::SessionID(sid) =
        ft_sdk::auth::provider::login(&mut conn, &uid, sid.map(ft_sdk::auth::SessionID))?;

    ft_sdk::println!("Create User done for sid {sid}");

    Ok(ft_sdk::form::redirect("/")?.with_cookie(common::session_cookie(sid.as_str(), host)?))
}

struct CreateAccount {
    mobile_number: String,
    username: String,
    user_id: Option<ft_sdk::UserId>
}

impl CreateAccount {
    fn to_provider_data(&self) -> ft_sdk::auth::ProviderData {
        ft_sdk::auth::ProviderData {
            identity: self.username.to_string(),
            username: Some(self.username.to_string()),
            name: None,
            emails: vec![],
            verified_emails: vec![],
            profile_picture: None,
            custom: serde_json::json!({
                "mobile_numbers": vec![self.mobile_number.clone()],
            }),
        }
    }
}

fn validate_and_account_meta(
    payload: CreateAccountPayload,
    conn: &mut ft_sdk::Connection,
) -> Result<CreateAccount, ft_sdk::Error> {
    let mut errors = std::collections::HashMap::new();

    payload.validate(conn, &mut errors)?;

    if !errors.is_empty() {
        return Err(ft_sdk::SpecialError::Multi(errors).into());
    }

    use diesel::prelude::*;

    #[derive(diesel::QueryableByName)]
    #[diesel(table_name = ft_sdk::auth::fastn_user)]
    struct Identity {
        identity: Option<String>,
        id: i64,
    }

    // check if the code is associated with a subscriber that is creating an account
    // if we find a user_id, it means the user is pre_verified
    let user_id = match diesel::sql_query(
        format!(r#"
            SELECT
                id, identity
            FROM fastn_user
            WHERE
                EXISTS (
                    SELECT 1
                    FROM json_each ( data -> '{}' -> 'custom' -> 'mobile_numbers')
                    WHERE value = $1
                )
            "#,  mobile_auth::PROVIDER_ID),
    )
        .bind::<diesel::sql_types::Text, _>(&payload.mobile_number)
        .get_result::<Identity>(conn)
    {
        Ok(identity) => {
            if identity.identity.is_some() {
                return Err(ft_sdk::single_error("mobile_number", "Mobile number already exists.").into());
            }
            Some(ft_sdk::auth::UserId(identity.id))
        }
        Err(diesel::result::Error::NotFound) => None,
        Err(e) => {
            return Err(e.into());
        }
    };

    Ok(CreateAccount {
        user_id,
        mobile_number: payload.mobile_number,
        username: payload.username
    })
}

#[derive(serde::Deserialize)]
pub struct CreateAccountPayload {
    pub(crate) username: String,
    pub(crate) mobile_number: String,
}

impl CreateAccountPayload {
    pub(crate) fn validate(
        &self,
        conn: &mut ft_sdk::Connection,
        errors: &mut std::collections::HashMap<String, String>,
    ) -> Result<(), ft_sdk::Error> {
        validate_mobile_number(self.mobile_number.as_str(), errors)?;
        common::validate_identity("username", &self.username, conn, errors)?;

        Ok(())
    }

}

fn validate_mobile_number(
    mobile_number: &str,
    errors: &mut std::collections::HashMap<String, String>,
) -> Result<(), ft_sdk::Error> {
    let re = regex::Regex::new(r"^\d{10,12}$").unwrap();
    if !re.is_match(mobile_number) {
        errors.insert("mobile_number".to_string(), "Invalid mobile number".to_string());
    }

    Ok(())
}