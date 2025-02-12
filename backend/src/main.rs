mod config;
mod schema;

use crate::config::load_config;
use crate::database::Database;
use crate::route::create_quest::create_quest;
use crate::route::get_avatar::get_avatar;
use crate::route::get_quest_info::get_quest_info;
use crate::route::get_quests_page::get_quest_page;
use crate::route::get_user_info::get_user_info;
use crate::route::get_user_owned_quests::get_user_owned_quests;
use crate::route::login::login_user;
use crate::route::partial_update_quest_info::partial_update_quest_info;
use crate::route::register::register_user;
use crate::route::update_avatar::update_avatar;
use crate::route::update_quest_page::update_quest_page;
use axum::routing::get;
use axum::{routing::post, Router};
use common::UserId;
use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;
// todo i do now: докер треба, квест історію (ті в які увійшов, не про власностворені),
// модераторський статус юзера, видавання файлів фронтенду

mod database;
mod route;
// можливо повністю прибрати ttl кеша

struct AppState {
    pub database: Database,
    pub session_cache: Cache<Uuid, UserId>,
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
        .route("/api/quests/create", post(create_quest))
        .route("/api/quests/{id}/info", get(get_quest_info))
        .route("/api/quests/{id}/info", post(partial_update_quest_info))
        .route("/api/quests/{id}/page/{page}", get(get_quest_page))
        .route("/api/quests/{id}/page/{page}", post(update_quest_page))
        .route("/api/owned_quests/page/{page}", get(get_user_owned_quests))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(config.app.address)
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
