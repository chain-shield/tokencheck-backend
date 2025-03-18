use uuid::Uuid;

use crate::misc::user::UserVerificationOrigin;

pub struct UserCreateRequest {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub company_name: Option<String>,
    pub verification_origin: UserVerificationOrigin,
}
pub struct AuthProviderCreateRequest {
    pub user_id: Uuid,
    pub provider: String,
    pub provider_user_id: String,
}
