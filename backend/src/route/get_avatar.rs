use crate::{ApiResponse, AppState};
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::header::CONTENT_TYPE;
use axum::response::{IntoResponse, Response};
use axum::{http::StatusCode, Json};
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

pub async fn get_avatar(Path(user_id): Path<String>, state: State<Arc<AppState>>) -> Response {
    let user_uuid = match Uuid::from_str(user_id.as_str()) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Json(ApiResponse::<()>::Error(String::from("bad user id"))).into_response();
        }
    };

    match state.database.get_user_avatar(user_uuid).await {
        None => Json(ApiResponse::<()>::Error(String::from(
            "there are no avatar for such id",
        )))
        .into_response(),
        Some((_, content_type, avatar_data)) => Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, content_type)
            .body(Body::from(avatar_data))
            .unwrap(),
    }
}
