use actix_session::Session;
use actix_web::{Responder, get, web};
use serde_json::json;

use crate::server::{
    misc::error::{AppError, Res},
    models::user::User,
};

#[utoipa::path(
    get,
    path = "/session",
    tag = "Session",
    summary = "Get session data",
    description = "Returns the current session data if the user is logged in.",
    responses(
        (status = 200, description = "Session data", body = User),
        (status = 401, description = "Unauthorized")
    )
)]
#[get("/session")]
async fn get_session(session: Session) -> Res<impl Responder> {
    let user = session
        .get::<String>("user")
        .map_err(|_| AppError::Internal("Session user error".to_string()))?
        .ok_or_else(|| AppError::Unauthorized("No user data found".to_string()))?;
    let token = session
        .get::<String>("token")
        .map_err(|_| AppError::Internal("Session token error".to_string()))?
        .ok_or_else(|| AppError::Unauthorized("No session token found".to_string()))?;

    Ok(web::Json(json!({
        "token": token,
        "user": serde_json::from_str::<User>(&user).map_err(|_| AppError::Internal("Failed to parse user json".to_string()))?
    })))
}
