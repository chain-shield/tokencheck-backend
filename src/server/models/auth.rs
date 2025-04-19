use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct JwtClaims {
    pub user_id: Uuid,
    pub stripe_customer_id: String,
    pub exp: u32,
}

pub struct ClaimsSpec {
    pub user_id: Uuid,
    pub stripe_customer_id: String,
}
