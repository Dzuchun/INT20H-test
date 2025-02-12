use crate::{ApiResponse, AppState};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::headers::Cookie;
use axum_extra::TypedHeader;
use common::{QuestId, UserOwnedQuestRecord, UserOwnedQuestsPage};
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

pub async fn get_user_owned_quests(
    state: State<Arc<AppState>>,
    Path(page): Path<String>,
    TypedHeader(session): TypedHeader<Cookie>,
) -> (StatusCode, Json<ApiResponse<UserOwnedQuestsPage>>) {
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

    let requested_page = match page.parse::<u32>() {
        Ok(requested_page) => requested_page,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::Error(String::from("provided bad page number"))),
            );
        }
    };

    if let Some((quest_info, total_pages)) = state
        .database
        .get_owned_quests(user_id.0, requested_page)
        .await
    {
        let data = quest_info
            .iter()
            .map(|x| UserOwnedQuestRecord {
                id: QuestId(x.clone()),
            })
            .collect();
        return (
            StatusCode::OK,
            Json(ApiResponse::Response(UserOwnedQuestsPage {
                data,
                page: requested_page,
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
