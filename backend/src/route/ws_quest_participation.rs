use crate::{ApiResponse, AppState};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{ConnectInfo, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use axum_extra::headers::Cookie;
use axum_extra::TypedHeader;
use common::{QuestInfo, WsClientMessage, WsServerMessage};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

pub async fn ws_quest_participation_handler(
    ws: WebSocketUpgrade,
    TypedHeader(session): TypedHeader<Cookie>,
    Path(id): Path<String>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    state: State<Arc<AppState>>,
) -> impl IntoResponse {
    let quest_uuid = match Uuid::from_str(id.as_str()) {
        Ok(uuid) => uuid,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::Error(String::from("bad quest id"))),
            )
                .into_response();
        }
    };

    let session_value = match session.get("session") {
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::<()>::Error(String::from("login required"))),
            )
                .into_response();
        }
        Some(value) => value,
    };

    let session_uuid = match Uuid::from_str(session_value) {
        Ok(uuid) => uuid,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::Error(String::from("internal server error, contact administrator with description of this situation"))),
            ).into_response();
        }
    };

    let user_uuid = match state.session_cache.get(&session_uuid).await {
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::<()>::Error(String::from("login required"))),
            )
                .into_response();
        }
        Some(user_id) => user_id,
    };

    let quest_info = if let Some(quest_info) = state.database.get_quest(quest_uuid).await {
        if !quest_info.published {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::<()>::Error(String::from(
                    "cannot participate in unpublished quest",
                ))),
            )
                .into_response();
        }
        quest_info
    } else {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::Error(String::from(
                "there are no such quest",
            ))),
        )
            .into_response();
    };

    match state
        .database
        .is_user_finished_quest(user_uuid.0, quest_uuid)
        .await
    {
        Some(is_finished) => {
            if is_finished {
                return (
                    StatusCode::FORBIDDEN,
                    Json(ApiResponse::<()>::Error(String::from("you finished quest"))),
                )
                    .into_response();
            }
        }
        None => {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::<()>::Error(String::from(
                    "not joined to quest",
                ))),
            )
                .into_response();
        }
    }

    ws.on_upgrade(move |socket| handle_socket(socket, addr, state, user_uuid.0, quest_info))
}

async fn handle_socket(
    mut socket: WebSocket,
    _who: SocketAddr,
    state: State<Arc<AppState>>,
    user_id: Uuid,
    quest_info: QuestInfo,
) {
    //forbid user multiple ws conn

    loop {
        // receive, react
        let client_msg = if let Some(msg) = socket.recv().await {
            match msg {
                Ok(Message::Text(data)) => {
                    match serde_json::from_str::<WsClientMessage>(data.as_str()) {
                        Ok(client_msg) => client_msg,
                        Err(_) => {
                            break;
                        }
                    }
                }
                Ok(_) => {
                    // unsupported, idk
                    break;
                }
                _ => {
                    break; //abruptly disconnected
                }
            }
        } else {
            break; // stream closed
        };

        let to_send = match client_msg {
            WsClientMessage::RequestPage(page) => {
                if page >= quest_info.pages {
                    break;
                } // there are no such page at all

                // check if page == completed_pages
                let last_not_completed_page = state
                    .database
                    .get_user_quest_last_completed_page(user_id, quest_info.id.0)
                    .await
                    .unwrap();
                if last_not_completed_page != page {
                    WsServerMessage::ResponsePage(Err(last_not_completed_page))
                } else {
                    // потрібна сторінка, взяти її з кешу

                    {
                        let f = state.quests_cache.lock().await.entry(quest_info.id.0);
                    }

                    //todo
                    WsServerMessage::RequestBail
                }
            }
            WsClientMessage::RequestSubmit(page, answer) => WsServerMessage::RequestBail,
        };
        let to_send = match serde_json::to_string(&to_send) {
            Ok(res) => res,
            Err(_) => {
                break;
            }
        };
        let _ = socket.send(Message::Text(to_send.into())).await;
    }
}
