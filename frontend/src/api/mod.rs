use std::future::Future;

use common::{
    Answer, Avatar, LoginRequest, QuestHistoryPage, QuestId, QuestInfo, QuestPage, RegisterRequest,
    Timestamp, UserId, UserInfo, UserOwnedQuestsPage,
};
use error::{GameError, LoginError, RegisterError};

use crate::GeneralError;

pub mod dummy;
pub mod error;

pub fn static_get_user_info(
    api: &impl Api,
    user_id: UserId,
) -> impl Future<Output = Result<UserInfo, GeneralError>> + Send + Sync {
    let api = api.clone();
    async move { api.get_user_info(user_id).await }
}

pub fn static_quest_history(
    api: &impl Api,
    user_id: UserId,
    page: u32,
) -> impl Future<Output = Result<QuestHistoryPage, GeneralError>> + Send + Sync {
    let api = api.clone();
    async move { api.quest_history(user_id, page).await }
}

pub trait Api: Clone + Send + Sync + 'static {
    fn login(&self, info: LoginRequest) -> impl Future<Output = Result<(), LoginError>> + Send;

    fn logout(&self);

    fn auth_user(&self) -> Result<Option<UserId>, GeneralError>;

    fn require_auth_user(&self) -> Result<UserId, GeneralError> {
        self.auth_user()?.ok_or(GeneralError::RequestLogIn)
    }

    fn register(
        &self,
        info: RegisterRequest,
    ) -> impl Future<Output = Result<(), RegisterError>> + Send + Sync;

    fn get_user_info(
        &self,
        user_id: UserId,
    ) -> impl Future<Output = Result<UserInfo, GeneralError>> + Send + Sync;

    fn set_avatar(
        &self,
        user_id: UserId,
        avatar: Avatar,
    ) -> impl Future<Output = Result<(), GeneralError>>;

    fn quest_history(
        &self,
        user_id: UserId,
        page: u32,
    ) -> impl Future<Output = Result<QuestHistoryPage, GeneralError>> + Sync + Send;

    fn user_quests(
        &self,
        user_id: UserId,
        page: u32,
    ) -> impl Future<Output = Result<UserOwnedQuestsPage, GeneralError>> + Sync + Send;

    fn create_quest(&self) -> impl Future<Output = Result<QuestId, GeneralError>> + Send + Sync;

    fn get_quest_info(
        &self,
        quest_id: QuestId,
    ) -> impl Future<Output = Result<QuestInfo, GeneralError>> + Send + Sync;

    fn set_quest_info(
        &self,
        quest_info: QuestInfo,
    ) -> impl Future<Output = Result<(), GeneralError>> + Send + Sync;

    fn set_page_source(
        &self,
        quest_id: QuestId,
        page: u32,
        source: String,
    ) -> impl Future<Output = Result<(), GeneralError>> + Send + Sync;

    fn get_page_source(
        &self,
        quest_id: QuestId,
        page: u32,
    ) -> impl Future<Output = Result<String, GeneralError>> + Send + Sync;

    fn active_quest(&self) -> Result<Option<(QuestId, u32, Timestamp)>, GeneralError>;

    fn start_quest(
        &self,
        quest_id: QuestId,
    ) -> impl Future<Output = Result<(), GameError>> + Send + Sync;

    fn quest_page(&self)
        -> impl Future<Output = Result<(QuestPage, u32), GameError>> + Send + Sync;

    fn answer_page(
        &self,
        answers: Box<[Answer]>,
    ) -> impl Future<Output = Result<(), GameError>> + Send + Sync;

    fn finish_quest(&self) -> impl Future<Output = Result<(), GameError>> + Send + Sync;
}
