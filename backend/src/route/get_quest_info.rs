use crate::{ApiResponse, AppState};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use common::{QuestInfo};
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

pub async fn get_quest_info(
    Path(id): Path<String>,
    state: State<Arc<AppState>>,
) -> (StatusCode, Json<ApiResponse<QuestInfo>>) {
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
        return (StatusCode::OK, Json(ApiResponse::Response(quest_info)));
    }

    (
        StatusCode::NOT_FOUND,
        Json(ApiResponse::Error(String::from("quest not found"))),
    )
}
