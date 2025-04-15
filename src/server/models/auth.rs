use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Claims {
    pub user_id: Uuid,
    pub exp: usize,
    pub plan_id: Uuid,
    pub sub_status: String,
}
