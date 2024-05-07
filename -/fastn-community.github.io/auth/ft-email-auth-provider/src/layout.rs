pub struct Auth {
    pub in_: ft_sdk::In,
    pub conn: ft_sdk::Connection,
}

impl ft_sdk::Layout for Auth {
    type Error = AuthError;

    fn from_in(in_: ft_sdk::In, _ty: ft_sdk::RequestType) -> Result<Self, Self::Error> {
        let mut conn = ft_sdk::default_connection()?;

        ft_sdk::migrate(
            &mut conn,
            "auth.wasm",
            include_dir::include_dir!("migrations"),
            vec![],
            &in_.now,
        )
        .unwrap();

        Ok(Self { in_, conn })
    }

    fn json(&mut self, page: serde_json::Value) -> Result<serde_json::Value, Self::Error> {
        Ok(serde_json::json!({
            "page": page,
        }))
    }

    fn render_error(err: Self::Error) -> http::Response<bytes::Bytes> {
        match err {
            AuthError::FormError(errors) => {
                ft_sdk::println!("form error: {errors:?}");
                ft_sdk::json_response(serde_json::json!({"errors": errors}), None)
            }
            AuthError::Sdk(error) => {
                ft_sdk::server_error!("sdk error: {error:?}").unwrap()
            }
            AuthError::Diesel(error) => {
                ft_sdk::server_error!("diesel error: {error:?}").unwrap()
            }
            AuthError::CantDeserializeInput(message) => {
                ft_sdk::server_error!("serde error: {message:?}").unwrap()
            }
            AuthError::Unauthorized(message) => {
                ft_sdk::not_found!("unauthorized error: {message}").unwrap()
            }
            AuthError::UsageError(message) => {
                ft_sdk::not_found!("{message}").unwrap()
            }
            AuthError::HashingError(message) => {
                ft_sdk::not_found!("{message}").unwrap()
            }
            AuthError::LoginError(e) => {
                ft_sdk::server_error!("login error: {e:?}").unwrap()
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("form error: {0:?}")]
    FormError(std::collections::HashMap<String, String>),
    #[error("sdk error: {0}")]
    Sdk(#[from] ft_sdk::Error),
    #[error("Diesel error: {0}")]
    Diesel(#[from] diesel::result::Error),
    #[error("cant deserialize input: {0}")]
    CantDeserializeInput(#[from] serde_json::Error),
    #[error("not authorised: {0}")]
    Unauthorized(String),
    #[error("usage error: {0}")]
    UsageError(String),
    #[error("password hash error: {0}")]
    HashingError(String),
    #[error("login error: {0:?}")]
    LoginError(#[from] ft_sdk::auth::provider::LoginError),
}

impl AuthError {
    pub fn form_error(field: &str, error: &str) -> Self {
        Self::FormError(std::collections::HashMap::from([(
            field.to_string(),
            error.to_string(),
        )]))
    }
}
