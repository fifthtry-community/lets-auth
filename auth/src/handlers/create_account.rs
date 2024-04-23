use auth::layout::{Auth, AuthError};

pub struct CreateAccount {
    username: String,
    password: String,
}

impl ft_sdk::Action<Auth, AuthError> for CreateAccount {
    fn validate(c: &mut Auth) -> Result<Self, AuthError>
    where
        Self: Sized,
    {
        todo!()
    }

    fn action(&self, c: &mut Auth) -> Result<ft_sdk::ActionOutput, AuthError>
    where
        Self: Sized,
    {
        todo!()
    }
}
