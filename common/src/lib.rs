use derive_more::{Display, From, FromStr, Into, TryFrom};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

mod questions;
pub use questions::*;

#[derive(
    Debug, Serialize, Deserialize, From, Into, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(transparent)]
pub struct UserId(pub Uuid);

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
    pub avatar_url: Option<Uuid>,
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

pub type Timestamp = chrono::NaiveDateTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestHistoryRecord {
    pub user_id: UserId,
    pub quest_id: QuestId,
    pub started_at: Timestamp,
    pub finished_at: Option<Timestamp>,
    pub completed_pages: u32,
}

pub const QUEST_HISTORY_PAGE_SIZE: usize = 20;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestHistoryPage {
    /// max len is limited in the constant above
    pub data: Box<[QuestHistoryRecord]>,
    pub page: u32,
    pub total_pages: u32,
}

// TODO: multiple moderators
#[derive(Debug, Clone, Copy, Serialize, Deserialize, TryFrom)]
#[try_from(repr)]
#[repr(u8)]
pub enum QuestState {
    /// While author still edits the quest and did not make it public yet
    Unpublished = 0,
    /// Author finished creating a quest and asked moderator for a review
    Submitted = 1, // <-- moderator can edit this
    /// After moderator editing.
    Returned = 2, // <-- moderator can edit this
    /// Quest is published by author
    Published = 3,
    /// Quest is published and reviewed by moderator
    PublishedReviewed = 4,
    /// Quest was locked down by a moderator. Any user completing a quest should be bailed out, author can view, but not edit and/or publish the quest.
    ///
    /// Terminal state
    Locked = 5, // <-- issued if moderator edits this quest, creating a new one to propose to the author
}

impl From<QuestState> for u8 {
    fn from(value: QuestState) -> Self {
        value as u8
    }
}

/// Provided by server for quest lists
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOwnedQuestRecord {
    pub id: QuestId,
    /* proly more fields? */
}

pub const USER_OWNED_QUESTS_PAGE_SIZE: usize = 20;

/// Provided by server for quest lists
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOwnedQuestsPage {
    /// max len is limited in the constant above
    pub data: Box<[UserOwnedQuestRecord]>,
    pub page: u32,
    pub total_pages: u32,
}

/// /api/quests/:id/info
#[derive(Debug, Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
pub struct QuestInfo {
    pub id: QuestId,
    pub owner: UserId,
    pub title: String,
    pub description: String,
    pub pages: u32,
    pub published: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
pub struct ImageRectangle {
    pub left: u32,
    pub top: u32,
    pub width: u32,
    pub height: u32,
}

impl ImageRectangle {
    fn contains(&self, left: u32, top: u32) -> bool {
        let Some(dx) = left.checked_sub(self.left) else {
            return false;
        };
        let Some(dy) = top.checked_sub(self.top) else {
            return false;
        };

        dx <= self.width && dy <= self.height
    }
}

/// fn parse(String) -> Vec<Question>
/// (parsed from source)
#[derive(Debug, Serialize, Deserialize, Clone, Hash, PartialEq, Eq)] // (for DB)
pub enum Question {
    Opened(String),
    Choice {
        variants: Box<[String]>,
        correct: u32,
    },
    MultipleChoice {
        variants: Box<[String]>,
        correct: Box<[u32]>,
    },
    Image {
        src: String,
        correct_bounds: ImageRectangle,
    },
}

impl From<&Question> for AskQuestion {
    fn from(value: &Question) -> Self {
        match value {
            Question::Opened(_) => AskQuestion::Opened,
            Question::Choice {
                variants,
                correct: _,
            } => AskQuestion::Choice {
                variants: variants.clone(),
            },
            Question::MultipleChoice {
                variants,
                correct: _,
            } => AskQuestion::MultipleChoice {
                variants: variants.clone(),
            },
            Question::Image {
                src,
                correct_bounds: _,
            } => AskQuestion::Image { src: src.clone() },
        }
    }
}

/// server -> client
#[derive(Debug, Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
pub enum AskQuestion {
    Opened,
    Choice { variants: Box<[String]> },
    MultipleChoice { variants: Box<[String]> },
    Image { src: String },
}

/// server <- client
#[derive(Debug, Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
pub enum Answer {
    Opened(String),
    Choice(u32),
    MultipleChoice(Box<[u32]>),
    Image { left: u32, top: u32 },
}

#[derive(Debug, Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
pub enum QuestPageElement {
    Text(Box<str>),
    Question(Question),
}

pub type QuestPage = Box<[QuestPageElement]>;

#[derive(Debug, Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
pub enum AskQuestPageElement {
    Text(Box<str>),
    Question(AskQuestion),
}

pub type AskQuestPage = Box<[AskQuestPageElement]>;

/// POST /api/quests/create
/// - returns [`QuestId`]
/// - title, desc, source, etc. are empty
///
/// GET /api/quests/:id/info
/// - return [`QuestInfo`]
///
/// POST /api/quests/:id/info
/// - accepts [`QuestInfo`]
///
/// GET /api/quests/:id/page/:page
/// - return [`String`] (source) //possibly return also time limit
///
/// POST /api/quests/:id/page/:page
/// - accepts [`String`] (source), [`Option<Duration>`] (time limit for page)
/// - server saves source (and [`Vec<Question>`], after quest is published)
/// - check for OK
///
/// // below is not final
/// POST /api/quests/:id/answer/:page
/// - accepts [`Vec<Answer>`]
/// - check for OK
///
/// qid = POST /quests/create
/// GET /api/quests/qid/info -- returns [`QuestInfo`] with pages=0
/// POST /api/quests/qid/page/0 "lalalal, question" -- creates page 0
/// GET /api/quests/qid/info -- returns [`QuestInfo`] with pages=1
/// POST /api/quests/qid/page/1 "lalalal, question2" -- creates page 1
/// GET /api/quests/qid/info -- returns [`QuestInfo`] with pages=2
/// POST /api/quests/qid/page/0 "lalalal, question ;)" -- updates page 0
/// POST /api/quests/qid/info "Title; description" -- update title/description
/// GET /api/quests/qid/info -- returns [`QuestInfo`] with pages=2
mod doc {
    use super::*;
}

// websocket is opened to POST /api/ws/quest/:id
//
// ~ flow ~
//
// *opened connection* (new / continue)
//

#[derive(Debug, Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
pub enum WsServerMessage {
    // ok - quest page to answer
    // err - only pages up to err are available
    ResponsePage(Result<AskQuestPage, u32>),
    // ok - answers were submitted
    // err - only pages up to err are available
    ResponseSubmit(Result<(), String>),
    // inform client about bail
    RequestBail,
}

#[derive(Debug, Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
pub enum WsClientMessage {
    RequestPage(u32),
    RequestSubmit(u32, Box<[Answer]>),
}
