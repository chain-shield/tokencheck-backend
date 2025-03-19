use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::server::models::sub::UserSubscription;

#[derive(Debug, Deserialize, ToSchema)]
pub struct SubscriptionCreateRequest {
    pub plan_id: Uuid,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SubscriptionResponse {
    pub subscription: UserSubscription,
    pub token: String,
}
