use crate::{ApiResponse, AppState};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::headers::Cookie;
use axum_extra::TypedHeader;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

pub async fn update_rate_comment(
    Path(id): Path<String>,
    TypedHeader(session): TypedHeader<Cookie>,
    state: State<Arc<AppState>>,
    Json((rate, comment)): Json<(u32, String)>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    let quest_uuid = match Uuid::from_str(id.as_str()) {
        Ok(uuid) => uuid,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::Error(String::from("bad quest id"))),
            );
        }
    };

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

    let user_uuid = match state.session_cache.get(&session_uuid).await {
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::Error(String::from("login required"))),
            );
        }
        Some(user_id) => user_id,
    };

    if let Some(quest_info) = state.database.get_quest(quest_uuid).await {
        if !quest_info.published {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::Error(String::from(
                    "cannot coment for unpublished",
                ))),
            );
        }
    } else {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::Error(String::from("there are no such quest"))),
        );
    }

    match state
        .database
        .is_user_finished_quest(user_uuid.0, quest_uuid)
        .await
    {
        Some(is_finished) => {
            if !is_finished {
                return (
                    StatusCode::FORBIDDEN,
                    Json(ApiResponse::Error(String::from("not finished quest"))),
                );
            }
        }
        None => {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::Error(String::from("not joined to quest"))),
            );
        }
    }

    match state
        .database
        .update_user_quest_rate_comment(&user_uuid.0, &quest_uuid, comment.as_str(), rate)
        .await
    {
        Some(_) => (StatusCode::OK, Json(ApiResponse::Response(()))),
        None => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::Error(String::from(
                "internal server error, contact administrator with description of this situation",
            ))),
        ),
    }
}
