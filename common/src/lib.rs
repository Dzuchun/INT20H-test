use derive_more::{Display, From, FromStr, Into, TryFrom};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, From, PartialEq, Eq, Hash)]
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
    pub user_id: UserId,
    pub state: QuestState,
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

/// quests/:id/info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestInfo {
    pub id: QuestId,
    pub owner: UserId,
    pub title: String,
    pub description: String,
    pub pages: u32,
}

/// fn parse(String) -> Vec<Question>
/// (parsed from source)
#[derive(Debug, Serialize, Deserialize)] // (for DB)
pub enum Question {
    Opened(/* correct answer */),
    Choice(/* variants and correct option */),
    MultipleChoice(/* variants and correct option(s) */),
    Image(/* image & correct rectangle */),
}

/// server -> client
pub enum AskQuestion {
    Opened,
    Choice(/* variants */),
    MultipleChoice(/* variants */),
    Image(/* image */),
}

/// server <- client
#[derive(Debug, Serialize, Deserialize)]
pub enum Answer {
    Opened(/* user's answer */),
    Choice(/* user's choice */),
    MultipleChoice(/* user's choice(s) */),
    Image(/* point (x,y) */),
}

/// POST /quests/create
/// - returns [`QuestId`]
/// - title, desc, source, etc. are empty
///
/// GET /quests/:id/info
/// - return [`QuestInfo`]
///
/// POST /quests/:id/info
/// - accepts [`QuestInfo`]
///
/// GET /quests/:id/page/:page
/// - return [`String`] (source)
///
/// POST /quests/:id/page/:page
/// - accepts [`String`] (source), [`Option<Duration>`] (time limit for page)
/// - server saves source (and [`Vec<Question>`], after quest is published)
/// - check for OK
///
/// // below is not final
/// POST /quests/:id/answer/:page
/// - accepts [`Vec<Answer>`]
/// - check for OK
///
/// /quests/:id
///
/// qid = POST /quests/create
/// GET /quests/qid/info -- returns [`QuestInfo`] with pages=0
/// POST /quests/qid/page/0 "lalalal, question" -- creates page 0
/// GET /quests/qid/info -- returns [`QuestInfo`] with pages=1
/// POST /quests/qid/page/1 "lalalal, question2" -- creates page 1
/// GET /quests/qid/info -- returns [`QuestInfo`] with pages=2
/// POST /quests/qid/page/0 "lalalal, question ;)" -- updates page 0
/// POST /quests/qid/info "Title; description" -- update title/description
/// GET /quests/qid/info -- returns [`QuestInfo`] with pages=2
mod doc {}
