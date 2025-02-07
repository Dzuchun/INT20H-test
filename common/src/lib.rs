use derive_more::{From, Into};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(
    Debug, Serialize, Deserialize, From, Into, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(transparent)]
pub struct UserId(Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub name: String,
    pub pass: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub name_or_email: String,
    pub pass: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: UserId,
    pub name: String,
    pub email: String, // probably will be changed for some sort of enum representing identity (?)
    pub avatar_url: Option<String>,
}

impl UserInfo {
    pub fn new(id: UserId, name: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            email: email.into(),
            avatar_url: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, From)]
#[serde(transparent)]
pub struct SessionKey(Uuid);

// TODO: probably check maximum size and file format
// (and convert for .webp?)
#[derive(Debug, Clone, From)]
pub struct Avatar(pub Vec<u8>);
