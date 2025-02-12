use crate::{ApiResponse, AppState};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::headers::Cookie;
use axum_extra::TypedHeader;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

pub async fn update_quest_page(
    state: State<Arc<AppState>>,
    Path(id): Path<String>,
    Path(page): Path<String>,
    TypedHeader(session): TypedHeader<Cookie>,
    Json((source, duration)): Json<(String, Option<Duration>)>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    let quest_id = match Uuid::from_str(id.as_str()) {
        Ok(uuid) => uuid,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::Error(String::from("provided bad quest id"))),
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

    let user_id = match state.session_cache.get(&session_uuid).await {
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::Error(String::from("login required"))),
            );
        }
        Some(user_id) => user_id,
    };

    let mut quest_info = if let Some(quest_info) = state.database.get_quest(quest_id).await {
        if quest_info.owner != user_id {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::Error(String::from(
                    "you do now own this quest",
                ))),
            );
        }
        quest_info
    } else {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::Error(String::from("there are no such quest"))),
        );
    };

    // todo accessible only before publishing in some states

    let quest_page = match page.parse::<u32>() {
        Ok(quest_page) => quest_page,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::Error(String::from("provided bad page number"))),
            );
        }
    };

    // продовжити звідси
    if quest_info.pages > quest_page {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::Error(String::from(
                "provided page number exceeding current possible page number",
            ))),
        );
    }

    if quest_info.pages == quest_page {
        quest_info.pages += 1;
        if let None = state.database.update_quest_pages(&quest_info).await {
            return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::Error(String::from(
                        "internal server error, contact administrator with description of this situation",
                    ))),
                );
        }
    }

    if let None = state
        .database
        .update_quest_page(
            quest_info.id,
            quest_page,
            source,
            duration.map(|x| x.as_secs() as u32),
        )
        .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::Error(String::from(
                "internal server error, contact administrator with description of this situation",
            ))),
        );
    }

    (StatusCode::OK, Json(ApiResponse::Response(())))
}
