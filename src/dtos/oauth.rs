use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
    pub code: String,
}

#[derive(Debug)]
pub struct OAuthUserData {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub provider_user_id: String,
}
