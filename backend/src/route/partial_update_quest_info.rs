use std::str::FromStr;
use std::sync::Arc;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::headers::Cookie;
use axum_extra::TypedHeader;
use uuid::Uuid;
use common::{ QuestInfo};
use crate::{ApiResponse, AppState};

pub async fn partial_update_quest_info(
    Path(id): Path<String>,
    state: State<Arc<AppState>>,
    TypedHeader(session): TypedHeader<Cookie>,
    Json(payload): Json<QuestInfo>,
) -> (StatusCode, Json<ApiResponse<()>>) {
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

    let quest_id = match Uuid::from_str(id.as_str()) {
        Ok(uuid) => uuid,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::Error(String::from("internal server error, contact administrator with description of this situation"))),
            );
        }
    };

    if let Some(quest_info) = state.database.get_quest(quest_id).await {
        if quest_info.owner != user_id {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::Error(String::from("you do now own this quest"))),
            );
        }
    } else {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::Error(String::from("there are no such quest"))),
        );
    }

    if let Some(quest_info) = state.database.update_quest(payload).await {
        return (StatusCode::OK, Json(ApiResponse::Response(quest_info)));
    }

    (
        StatusCode::NOT_FOUND,
        Json(ApiResponse::Error(String::from("quest not found"))),
    )
}
