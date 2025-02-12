use crate::{ApiResponse, AppState};
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum_extra::headers::Cookie;
use axum_extra::TypedHeader;
use common::QuestId;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

pub async fn create_quest(
    state: State<Arc<AppState>>,
    TypedHeader(session): TypedHeader<Cookie>,
) -> (StatusCode, Json<ApiResponse<QuestId>>) {
    let session_value = match session.get("session") {
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::Error(String::from("login required"))),
            );
        }
        Some(value) => value,
    };

    let session_uuid = match Uuid::from_str(session_value) {
        Ok(uuid) => uuid,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::Error(String::from("internal server error, contact administrator with description of this situation"))),
            );
        }
    };

    let user_id = match state.session_cache.get(&session_uuid).await {
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::Error(String::from("login required"))),
            );
        }
        Some(user_id) => user_id,
    };

    match state.database.create_quest(user_id.0).await {
        None => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::Error(String::from(
                "internal server error, contact administrator with description of this situation",
            ))),
        ),
        Some(quest_id) => (
            StatusCode::OK,
            Json(ApiResponse::Response(QuestId(quest_id))),
        ),
    }
}
