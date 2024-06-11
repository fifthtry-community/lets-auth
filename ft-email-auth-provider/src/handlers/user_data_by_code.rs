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
    use diesel::prelude::*;

    #[derive(diesel::QueryableByName)]
    #[diesel(table_name = ft_sdk::auth::fastn_user)]
    struct UD {
        data: String,
    }

    let data = match diesel::sql_query(
        r#"
            SELECT
                data -> 'email' as data
            FROM fastn_user
            WHERE
                EXISTS (
                    SELECT 1
                    FROM json_each ( data -> 'subscription' -> 'subscription' -> 'confirmation-code')
                    WHERE value = $1
                )
        "#,
    )
    .bind::<diesel::sql_types::Text, _>(&code)
    .get_result::<UD>(&mut conn)
    {
        Ok(ud) => ud.data,
        Err(diesel::result::Error::NotFound) => {
            return Err(ft_sdk::single_error("code", "No user found with this code.").into());
        }
        Err(e) => return Err(e.into()),
    };

    let data: ft_sdk::auth::ProviderData = serde_json::from_str(&data)?;

    ft_sdk::println!("got data for pre filling: {:?}", data);

    let resp = PreFillData {
        email: data
            .emails
            .first()
            .cloned()
            .expect("imported data must have one email"),
    };

    ft_sdk::data::json(resp)
}
