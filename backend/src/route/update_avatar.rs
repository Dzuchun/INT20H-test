use crate::{ApiResponse, AppState};
use axum::extract::{Multipart, State};
use axum::{http::StatusCode, Json};
use axum_extra::headers::Cookie;
use axum_extra::TypedHeader;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

pub async fn update_avatar(
    state: State<Arc<AppState>>,
    TypedHeader(session): TypedHeader<Cookie>,
    mut multipart: Multipart,
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
    while let Ok(Some(field)) = multipart.next_field().await {
        //todo unwraps get out, also maybe filter some content types
        let content_type = field.content_type().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        if let None = state
            .database
            .update_user_avatar(user_id.0, content_type.as_str(), data.as_ref())
            .await
        {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::Error(String::from("internal server error, contact administrator with description of this situation"))),
            );
        }
        return (StatusCode::OK, Json(ApiResponse::Response(())));
    }
    (
        StatusCode::OK,
        Json(ApiResponse::Error(String::from("no avatar found"))),
    )
}
