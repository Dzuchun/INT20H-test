use super::Api;

pub struct RealApi {}

impl Api for RealApi {
    fn login(
        &self,
        info: common::LoginRequest,
    ) -> impl std::future::Future<Output = Result<(), super::error::LoginError>> + Send {
        todo!()
    }

    fn logout(&self) {
        todo!()
    }

    fn auth_user(&self) -> Result<Option<common::UserId>, crate::GeneralError> {
        todo!()
    }

    fn register(
        &self,
        info: common::RegisterRequest,
    ) -> impl std::future::Future<Output = Result<(), super::error::RegisterError>> + Send + Sync
    {
        todo!()
    }

    fn get_user_info(
        &self,
        user_id: common::UserId,
    ) -> impl std::future::Future<Output = Result<common::UserInfo, crate::GeneralError>> + Send + Sync
    {
        todo!()
    }

    fn set_avatar(
        &self,
        user_id: common::UserId,
        avatar: common::Avatar,
    ) -> impl std::future::Future<Output = Result<(), crate::GeneralError>> {
        todo!()
    }

    fn quest_history(
        &self,
        user_id: common::UserId,
        page: u32,
    ) -> impl std::future::Future<Output = Result<common::QuestHistoryPage, crate::GeneralError>>
           + Sync
           + Send {
        todo!()
    }

    fn user_quests(
        &self,
        user_id: common::UserId,
        page: u32,
    ) -> impl std::future::Future<Output = Result<common::UserOwnedQuestsPage, crate::GeneralError>>
           + Sync
           + Send {
        todo!()
    }

    fn create_quest(
        &self,
    ) -> impl std::future::Future<Output = Result<common::QuestId, crate::GeneralError>> + Send + Sync
    {
        todo!()
    }

    fn get_quest_info(
        &self,
        quest_id: common::QuestId,
    ) -> impl std::future::Future<Output = Result<common::QuestInfo, crate::GeneralError>> + Send + Sync
    {
        todo!()
    }

    fn set_quest_info(
        &self,
        quest_info: common::QuestInfo,
    ) -> impl std::future::Future<Output = Result<(), crate::GeneralError>> + Send + Sync {
        todo!()
    }

    fn set_page_source(
        &self,
        quest_id: common::QuestId,
        page: u32,
        source: String,
    ) -> impl std::future::Future<Output = Result<(), crate::GeneralError>> + Send + Sync {
        todo!()
    }

    fn get_page_source(
        &self,
        quest_id: common::QuestId,
        page: u32,
    ) -> impl std::future::Future<Output = Result<String, crate::GeneralError>> + Send + Sync {
        todo!()
    }

    fn active_quest(
        &self,
    ) -> Result<Option<(common::QuestId, u32, common::Timestamp)>, crate::GeneralError> {
        todo!()
    }

    fn start_quest(
        &self,
        quest_id: common::QuestId,
    ) -> impl std::future::Future<Output = Result<(), super::error::GameError>> + Send + Sync {
        todo!()
    }

    fn quest_page(
        &self,
    ) -> impl std::future::Future<Output = Result<(common::QuestPage, u32), super::error::GameError>>
           + Send
           + Sync {
        todo!()
    }

    fn answer_page(
        &self,
        answers: Box<[common::Answer]>,
    ) -> impl std::future::Future<Output = Result<(), super::error::GameError>> + Send + Sync {
        todo!()
    }

    fn finish_quest(
        &self,
    ) -> impl std::future::Future<Output = Result<(), super::error::GameError>> + Send + Sync {
        todo!()
    }
}
