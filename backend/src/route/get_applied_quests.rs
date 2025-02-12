use crate::{ApiResponse, AppState};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::headers::Cookie;
use axum_extra::TypedHeader;
use common::{QuestHistoryPage, QuestHistoryRecord, QuestId};
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

pub async fn get_applied_quests(
    state: State<Arc<AppState>>,
    Path(page): Path<String>,
    TypedHeader(session): TypedHeader<Cookie>,
) -> (StatusCode, Json<ApiResponse<QuestHistoryPage>>) {
    let quest_history_page = match page.parse::<u32>() {
        Ok(quest_history_page) => quest_history_page,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::Error(String::from("provided bad page number"))),
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

    if let Some((user_quests, total_pages)) = state
        .database
        .get_user_quest_history(user_id.0, quest_history_page)
        .await
    {
        let data = user_quests
            .iter()
            .map(
                |(quest_id, started_at, finished_at, completed_pages)| QuestHistoryRecord {
                    user_id,
                    quest_id: QuestId(quest_id.clone()),
                    started_at: started_at.clone(),
                    finished_at: finished_at.clone(),
                    completed_pages: *completed_pages,
                },
            )
            .collect();
        return (
            StatusCode::OK,
            Json(ApiResponse::Response(QuestHistoryPage {
                data,
                page: quest_history_page,
                total_pages,
            })),
        );
    } else {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::Error(String::from("there are no such page"))),
        );
    }
}
