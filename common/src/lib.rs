use derive_more::{Display, From, FromStr, Into};
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

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, From, FromStr, Display,
)]
#[display("{_0}")]
pub struct QuestId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize, From)]
pub struct Timestamp(pub chrono::NaiveDateTime);

#[derive(Debug, Clone, Serialize, Deserialize, From)]
pub struct Completion {
    pub completed: u32,
    pub total_pages: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestHistoryRecord {
    pub user_id: UserId,
    pub quest_id: QuestId,
    pub started_at: Timestamp,
    pub finished_at: Timestamp,
    pub completion: Completion,
}

pub const QUEST_HISTORY_PAGE_SIZE: usize = 20;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestHistoryPage {
    /// max len is defined in the constant above
    pub data: Box<[QuestHistoryRecord]>,
    pub page: u32,
    pub total_pages: u32,
}

// TODO: multiple moderators
#[derive(Debug, Clone, Copy, Serialize, Deserialize, TryFrom)]
#[try_from(repr)]
#[repr(u8)]
pub enum QuestState {
    Dummy = 255,
}

impl From<QuestState> for u8 {
    fn from(value: QuestState) -> Self {
        value as u8
    }
}

/// A quest info required to display a quest on user's home/profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOwnedQuestRecord {
    pub id: QuestId,
    pub user_id: UserId,
    pub state: QuestState,
    /* proly more fields? */
}

pub const USER_OWNED_QUESTS_PAGE_SIZE: usize = 20;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOwnedQuestsPage {
    /// max len is limited in the constant above
    pub data: Box<[UserOwnedQuestRecord]>,
    pub page: u32,
    pub total_pages: u32,
}
