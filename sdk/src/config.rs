// #[derive(Debug)] // TODO: uncomment when ft-sdk 0.6.3 is out
pub struct Config {
    pub app_url: lets_auth::AppUrl,
    pub email_sender_name: String,
    pub email_reply_to: String,
    pub super_user_id: i64,
    pub is_personal_site: bool,
}

impl ft_sdk::FromRequest for Config {
    fn from_request(_req: &http::Request<serde_json::Value>) -> Result<Self, ft_sdk::Error> {
        todo!()
        // TODO: Uncomment when ft-sdk 0.6.3 is out
        // #[derive(Debug, serde::Deserialize)]
        // #[serde(rename_all = "kebab-case")]
        // pub struct C {
        //     email_sender_name: String,
        //     email_reply_to: String,
        //     super_user_id: i64,
        //     is_personal_site: bool,
        // }
        //
        // let c: C =
        //     ft_sdk::Config::from_request_for_key(lets_auth::SYSTEM, req).map(|c| c.into())?;
        //
        // Ok(Config {
        //     app_url: ft_sdk::FromRequest::from_request(req)?,
        //     email_sender_name: c.email_sender_name,
        //     email_reply_to: c.email_reply_to,
        //     super_user_id: c.super_user_id,
        //     is_personal_site: c.is_personal_site,
        // })
    }
}
