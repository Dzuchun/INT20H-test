use crate::{ApiResponse, AppState};
use axum::extract::{Path, State};
use axum::{http::StatusCode, Json};
use common::{UserId, UserInfo};
use std::sync::Arc;

pub async fn get_user_info(
    Path(username_or_email): Path<String>,
    state: State<Arc<AppState>>,
) -> (StatusCode, Json<ApiResponse<UserInfo>>) {
    if let Some((user_id_found, name_found, email_found, avatar_url_found)) = state
        .database
        .find_user_by_email_or_name(username_or_email.as_str())
        .await
    {
        return (
            StatusCode::OK,
            Json(ApiResponse::Response(UserInfo {
                id: UserId(user_id_found),
                name: name_found,
                email: email_found,
                avatar_url: avatar_url_found,
            })),
        );
    }
    (
        StatusCode::NOT_FOUND,
        Json(ApiResponse::Error(String::from("user not found"))),
    )
}
