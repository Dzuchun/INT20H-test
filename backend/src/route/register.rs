use crate::{ApiResponse, AppState};
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue};
use axum::{http::StatusCode, Json};
use common::{RegisterRequest, RegisterResponse, UserId};
use std::sync::Arc;
use uuid::Uuid;

pub async fn register_user(
    state: State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> (StatusCode, HeaderMap, Json<ApiResponse<RegisterResponse>>) {
    if !payload.email.contains('@') || payload.email.len() > 320 {
        return (
            StatusCode::BAD_REQUEST,
            HeaderMap::new(),
            Json(ApiResponse::Error(String::from(
                "email must contain '@' and be with length less than 320",
            ))),
        );
    }

    if payload.name.chars().any(|char| !char.is_alphanumeric()) || payload.name.len() > 32 {
        return (
            StatusCode::BAD_REQUEST,
            HeaderMap::new(),
            Json(ApiResponse::Error(String::from(
                "name must contain only alphanumeric symbols and be with length less than 33",
            ))),
        );
    }

    let password_hash_input = match bcrypt::hash(payload.pass, bcrypt::DEFAULT_COST) {
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                HeaderMap::new(),
                Json(ApiResponse::Error(String::from("choose another pass"))),
            );
        }
        Ok(hash) => hash,
    };

    if let Some(user_id) = state
        .database
        .insert_user(
            payload.name.as_str(),
            payload.email.as_str(),
            password_hash_input.as_str(),
        )
        .await
    {
        let session_key = Uuid::new_v4();
        state
            .session_cache
            .insert(session_key, UserId(user_id))
            .await;
        let mut headers = HeaderMap::new();
        //if you want to change, also look for 5*60 and other Max-Age=300
        headers.insert(
            "Set-Cookie",
            HeaderValue::from_str(&format!("session={}; Max-Age=300", session_key)).unwrap(),
        );
        return (
            StatusCode::CREATED,
            headers,
            Json(ApiResponse::Response(RegisterResponse {
                id: UserId(user_id),
            })),
        );
    }
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        HeaderMap::new(),
        Json(ApiResponse::Error(String::from(
            "internal error, try again later",
        ))),
    )
}
