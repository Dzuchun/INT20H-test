mod config;
mod schema;

use crate::config::load_config;
use axum::body::Body;
use axum::extract::{Multipart, Path, State};
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderMap, HeaderValue};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{http::StatusCode, routing::post, Json, Router};
use axum_extra::headers::Cookie;
use axum_extra::TypedHeader;
use common::{LoginRequest, LoginResponse, RegisterRequest, RegisterResponse, UserId, UserInfo};
use diesel::r2d2::{self, ConnectionManager};
use diesel::ExpressionMethods;
use diesel::{PgConnection, QueryDsl, RunQueryDsl};
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;
//todo i do now: докер треба, квести, модераторський статус

// можливо повністю прибрати ttl кеша

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

struct Database {
    pool: DbPool,
}

struct AppState {
    database: Database,
    session_cache: Cache<Uuid, UserId>,
}

impl Database {
    fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    fn get_conn(&self) -> Option<r2d2::PooledConnection<ConnectionManager<PgConnection>>> {
        self.pool.get().ok()
    }

    async fn get_conn_to_death(&self) -> r2d2::PooledConnection<ConnectionManager<PgConnection>> {
        for _ in 0..500 {
            if let Some(conn) = self.get_conn() {
                return conn;
            }
            sleep(Duration::from_millis(10)).await;
        }
        panic!("connection pool is possibly dead");
    }

    async fn find_userid_pass_by_email_or_name(&self, input: &str) -> Option<(Uuid, String)> {
        use crate::schema::users::dsl::*;
        let mut conn = self.get_conn_to_death().await;
        if input.contains('@') {
            users
                .filter(email.eq(input))
                .select((id, password_hash))
                .first::<(Uuid, String)>(&mut conn)
                .ok()
        } else {
            users
                .filter(name.eq(input))
                .select((id, password_hash))
                .first::<(Uuid, String)>(&mut conn)
                .ok()
        }
    }

    async fn is_exist_user_avatar(&self, user_id: Uuid) -> Option<Uuid> {
        use crate::schema::avatars::dsl::*;
        let mut conn = self.get_conn_to_death().await;
        avatars
            .filter(id.eq(user_id))
            .select(id)
            .first::<Uuid>(&mut conn)
            .ok()
    }

    async fn get_user_avatar(&self, user_id: Uuid) -> Option<(Uuid, String, Vec<u8>)> {
        use crate::schema::avatars::dsl::*;
        let mut conn = self.get_conn_to_death().await;
        avatars
            .filter(id.eq(user_id))
            .select((id, content_type, image_data))
            .first::<(Uuid, String, Vec<u8>)>(&mut conn)
            .ok()
    }

    async fn find_user_by_email_or_name(
        &self,
        input: &str,
    ) -> Option<(Uuid, String, String, Option<Uuid>)> {
        use crate::schema::users::dsl::*;
        let mut conn = self.get_conn_to_death().await;
        let res = if input.contains('@') {
            users
                .filter(email.eq(input))
                .select((id, name, email))
                .first::<(Uuid, String, String)>(&mut conn)
                .ok()
        } else {
            users
                .filter(name.eq(input))
                .select((id, name, email))
                .first::<(Uuid, String, String)>(&mut conn)
                .ok()
        };
        match res {
            None => None,
            Some((user_id, user_name, user_email)) => {
                let user_avatar = self.is_exist_user_avatar(user_id).await;
                Some((user_id, user_name, user_email, user_avatar))
            }
        }
    }
    //
    async fn update_user_avatar(&self, user_id: Uuid, content_type_value: &str, new_avatar: &[u8]) {
        let mut conn = self.get_conn_to_death().await;

        use crate::schema::avatars::dsl::*;
        diesel::insert_into(avatars)
            .values((
                id.eq(user_id),
                image_data.eq(new_avatar),
                content_type.eq(content_type_value),
            ))
            .on_conflict(id)
            .do_update()
            .set((
                image_data.eq(new_avatar),
                content_type.eq(content_type_value),
            ))
            .execute(&mut conn)
            .unwrap();
    }

    async fn insert_user(
        &self,
        name_input: &str,
        email_input: &str,
        password_hash_input: &str,
    ) -> Option<Uuid> {
        use crate::schema::users::dsl::*;
        let mut conn = self.get_conn_to_death().await;
        let new_user_id = Uuid::new_v4();
        diesel::insert_into(users)
            .values((
                id.eq(new_user_id),
                name.eq(name_input),
                email.eq(email_input),
                password_hash.eq(password_hash_input),
            ))
            .execute(&mut conn)
            .ok()?;
        Some(new_user_id)
    }
}

//todo potentially redo cache into own struct and use its methods
//todo session keys and other uuid must be unique?

//todo move to common
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiResponse<T> {
    #[serde(rename = "error")]
    Error(String),
    #[serde(untagged)]
    Response(T),
}

#[tokio::main]
async fn main() {
    let config = load_config("config.toml").expect("problem with config loading");

    let db_address = format!(
        "postgres://{}:{}@{}/{}",
        config.database.username,
        config.database.user_password,
        config.database.address,
        config.database.db_name
    );

    let manager = ConnectionManager::<PgConnection>::new(db_address);
    let db_pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool");
    let database = Database::new(db_pool);

    let session_cache = Cache::builder()
        .time_to_idle(Duration::from_secs(5 * 60))
        //if you want to change, also look for Max-Age=300
        .build();

    let app_state = AppState {
        database,
        session_cache,
    };
    let app_state = Arc::new(app_state);

    let app = Router::new()
        .route("/api/login", post(login_user))
        .route("/api/register", post(register_user))
        .route("/api/get_user/{username_or_email}", get(get_user_info))
        .route("/api/update_avatar", post(update_avatar))
        .route("/api/get_avatar/{user_id}", get(get_avatar))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(config.app.address)
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn login_user(
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

async fn register_user(
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

async fn get_user_info(
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

async fn update_avatar(
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

        state
            .database
            .update_user_avatar(user_id.0, content_type.as_str(), data.as_ref())
            .await;
        return (StatusCode::OK, Json(ApiResponse::Response(())));
    }
    (
        StatusCode::OK,
        Json(ApiResponse::Error(String::from("no avatar found"))),
    )
}

async fn get_avatar(Path(user_id): Path<String>, state: State<Arc<AppState>>) -> Response {
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
