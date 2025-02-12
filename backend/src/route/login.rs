use crate::{ApiResponse, AppState};
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue};
use axum::{http::StatusCode, Json};
use common::{LoginRequest, LoginResponse, UserId};
use std::sync::Arc;
use uuid::Uuid;

pub async fn login_user(
    state: State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> (StatusCode, HeaderMap, Json<ApiResponse<LoginResponse>>) {
    if let Some((user_id, db_pass_hash)) = state
        .database
        .find_userid_pass_by_email_or_name(&payload.name_or_email)
        .await
    {
        let valid = match bcrypt::verify(payload.pass, db_pass_hash.as_str()) {
            Ok(b) => b,
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    HeaderMap::new(),
                    Json(ApiResponse::Error(String::from(
                        "such password cannot be verified",
                    ))),
                );
            }
        };

        return {
            if valid {
                let session_key = Uuid::new_v4();
                state
                    .session_cache
                    .insert(session_key, UserId(user_id))
                    .await;
                let mut headers = HeaderMap::new();
                //if you want to change, also look for 5*60 and other Max-Age=300
                headers.insert(
                    "Set-Cookie",
                    HeaderValue::from_str(&format!("session={}; Max-Age=300", session_key))
                        .unwrap(),
                );
                (
                    StatusCode::OK,
                    headers,
                    Json(ApiResponse::Response(LoginResponse {
                        id: UserId(user_id),
                    })),
                )
            } else {
                (
                    StatusCode::BAD_REQUEST,
                    HeaderMap::new(),
                    Json(ApiResponse::Error(String::from("access denied"))),
                )
            }
        };
    }
    (
        StatusCode::UNAUTHORIZED,
        HeaderMap::new(),
        Json(ApiResponse::Error(String::from("user was not found"))),
    )
}
