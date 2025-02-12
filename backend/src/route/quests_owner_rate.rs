use crate::{ApiResponse, AppState};
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use std::sync::Arc;
use uuid::Uuid;

pub async fn get_quests_owner_rate(
    state: State<Arc<AppState>>,
) -> (StatusCode, Json<ApiResponse<Vec<(Uuid, f64)>>>) {
    // possible ddos, cache it?
    if let Some(x) = state.database.get_quest_avg_rate_per_owner().await {
        (StatusCode::OK, Json(ApiResponse::Response(x)))
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::Error(String::from(
                "internal server error, contact administrator with description of this situation",
            ))),
        )
    }
}
