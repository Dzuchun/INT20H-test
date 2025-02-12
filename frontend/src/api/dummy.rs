use std::{
    collections::HashMap,
    ops::RangeBounds,
    sync::{Arc, Mutex, MutexGuard},
    time::Duration,
};

use chrono::TimeDelta;
use common::{
    LoginRequest, QuestHistoryPage, QuestHistoryRecord, QuestId, QuestInfo, RegisterRequest,
    Timestamp, UserId, UserInfo, UserOwnedQuestRecord, UserOwnedQuestsPage,
    QUEST_HISTORY_PAGE_SIZE, USER_OWNED_QUESTS_PAGE_SIZE,
};
use fastrand::Rng as FastRng;
use leptos::logging;
use uuid::Uuid;

use crate::{EntityKind, GeneralError};

use super::{error::GameError, Api, LoginError, QuestPage, RegisterError};

fn now() -> Timestamp {
    chrono::Utc::now().naive_local()
}

async fn server_response() {
    // imitate slow server response
    #[cfg(debug_assertions)]
    wasmtimer::tokio::sleep(Duration::from_millis(200)).await;
}

#[derive(Debug)]
struct Data {
    rng: FastRng,
    users: HashMap<UserId, (String, UserInfo)>,
    name_or_emails: HashMap<String, UserId>,
    auth_user: Option<UserId>,
    quests: HashMap<QuestId, QuestInfo>,
    quest_pages: HashMap<(QuestId, u32), String>,
    user_data: HashMap<UserId, (Vec<QuestHistoryRecord>, Vec<QuestId>)>,
    active_quest: Option<(QuestId, u32, Timestamp)>,
}

impl Data {
    fn new() -> Self {
        Self {
            rng: FastRng::with_seed(42),
            users: HashMap::new(),
            name_or_emails: HashMap::new(),
            auth_user: None,
            quests: HashMap::<QuestId, QuestInfo>::new(),
            quest_pages: HashMap::<(QuestId, u32), String>::new(),
            user_data: HashMap::new(),
            active_quest: None,
        }
    }

    fn server_failure(&mut self) -> Result<(), GeneralError> {
        #[cfg(debug_assertions)]
        if self.rng.f32() < 0.05 {
            leptos::logging::warn!("Don't worry, it's a simulated error");
            return Err(GeneralError::Unknown);
        }

        Ok(())
    }

    fn id<Id: From<Uuid>>(&mut self) -> Id {
        Uuid::from_u128(self.rng.u128(..)).into()
    }

    fn try_register(
        &mut self,
        name: impl Into<String>,
        email: impl Into<String>,
        pass: impl Into<String>,
    ) -> Option<UserId> {
        let name = name.into();
        let email = email.into();
        if self.name_or_emails.contains_key(&name) || self.name_or_emails.contains_key(&email) {
            return None;
        }

        let id = self.id();
        self.name_or_emails.insert(name.clone(), id);
        self.name_or_emails.insert(email.clone(), id);
        self.users
            .insert(id, (pass.into(), UserInfo::new(id, name, email)));
        Some(id)
    }

    fn try_auth(
        &self,
        name_or_email: impl AsRef<str>,
        pass: impl AsRef<str>,
    ) -> Result<UserId, LoginError> {
        let id = self
            .name_or_emails
            .get(name_or_email.as_ref())
            .ok_or(LoginError::InvalidCredentials)?;
        let (correct_pass, info) = self.users.get(id).ok_or(LoginError::InvalidCredentials)?;
        if correct_pass == pass.as_ref() {
            Ok(info.id)
        } else {
            Err(LoginError::InvalidCredentials)
        }
    }

    fn get_user_info(&self, id: UserId) -> Option<UserInfo> {
        self.users.get(&id).map(|(_, info)| info.clone())
    }

    fn require_auth(&self) -> Result<UserId, GeneralError> {
        self.auth_user.ok_or(GeneralError::RequestLogIn)
    }

    fn get_quest_hisory(&self, user_id: UserId) -> &[QuestHistoryRecord] {
        self.user_data.get(&user_id).map_or(&[], |v| &v.0[..])
    }

    fn get_user_quests(&self, user_id: UserId) -> &[QuestId] {
        self.user_data.get(&user_id).map_or(&[], |v| &v.1[..])
    }

    fn timestamp(&mut self, hour_offset: impl RangeBounds<i64>) -> Timestamp {
        let hour_offset = self.rng.i64(hour_offset);
        let now = chrono::Utc::now().naive_local();
        now.checked_sub_signed(
            TimeDelta::new(
                3600 * hour_offset + self.rng.i64(0..3600),
                self.rng.u32(..1_000_000_000),
            )
            .unwrap(),
        )
        .unwrap()
        .into()
    }

    fn create_quest(&mut self, author: UserId) -> QuestId {
        let quest_id = self.id::<QuestId>();
        self.user_data.entry(author).or_default().1.push(quest_id);
        self.quests.insert(
            quest_id,
            QuestInfo {
                id: quest_id,
                owner: author,
                title: String::new(),
                description: String::new(),
                pages: 0,
                published: false,
            },
        );
        quest_id
    }

    fn get_quest_page(&self, quest_id: QuestId, page: u32) -> Option<String> {
        self.quest_pages.get(&(quest_id, page)).cloned()
    }

    fn set_quest_page(
        &mut self,
        quest_id: QuestId,
        page: u32,
        source: String,
    ) -> Result<(), GeneralError> {
        let info = self
            .quests
            .get_mut(&quest_id)
            .ok_or(GeneralError::UnknownEntity(EntityKind::Quest))?;

        if page > info.pages {
            return Err(GeneralError::UnknownEntity(EntityKind::QuestPage));
        }

        if page == info.pages {
            info.pages += 1;
        }
        self.quest_pages.insert((quest_id, page), source);

        Ok(())
    }
}

fn extract_page<const PAGE_SIZE: usize, I, T>(
    data: &[T],
    page: u32,
    mapper: impl Fn(&T) -> I,
) -> (Box<[I]>, u32, u32) {
    // const PAGE_SIZE: usize = 20;
    let len = data.len();
    let data = data[(PAGE_SIZE * page as usize).min(len.saturating_sub(1))
        ..(PAGE_SIZE * (page as usize + 1)).min(len)]
        .iter()
        .map(mapper)
        .collect::<Box<[I]>>();
    (data, page, u32::try_from(len.div_ceil(PAGE_SIZE)).unwrap())
}

#[derive(Debug, Clone)]
pub struct DummyApi {
    data: Arc<Mutex<Data>>,
}

impl Default for DummyApi {
    fn default() -> Self {
        Self::new()
    }
}

impl DummyApi {
    #[allow(clippy::missing_panics_doc)]
    pub fn new() -> Self {
        let mut data = Data::new();

        // regiter admin and assign some quest history to them
        let admin_id = data
            .try_register("admin", "admin@example.com", "admin")
            .unwrap();

        for _ in 0..42 {
            let record = QuestHistoryRecord {
                user_id: admin_id,
                quest_id: data.id(),
                started_at: data.timestamp(2..10),
                finished_at: Some(data.timestamp(0..2)),
                completed_pages: data.rng.u32(0..10),
            };
            data.user_data.entry(admin_id).or_default().0.push(record);
        }

        for _ in 0..20 {
            let _ = data.create_quest(admin_id);
        }

        Self {
            data: Arc::new(Mutex::new(data)),
        }
    }

    fn lock_data(&self) -> Result<MutexGuard<'_, Data>, GeneralError> {
        self.data.lock().map_err(|_| GeneralError::Unknown)
    }
}

impl Api for DummyApi {
    async fn login(
        &self,
        LoginRequest {
            name_or_email,
            pass,
        }: LoginRequest,
    ) -> Result<(), LoginError> {
        logging::log!("Login: name_or_email={name_or_email}, pass={pass}");
        server_response().await;
        let mut data = self.lock_data()?;
        data.server_failure()?;

        let id = data.try_auth(name_or_email, pass)?;
        data.auth_user = Some(id);
        Ok(())
    }

    fn logout(&self) {
        let Ok(mut data) = self.lock_data() else {
            logging::warn!("Data poison on logout");
            return;
        };

        data.auth_user = None;
    }

    fn auth_user(&self) -> Result<Option<UserId>, GeneralError> {
        let data = self.lock_data()?;
        Ok(data.auth_user)
    }

    async fn register(
        &self,
        RegisterRequest { email, name, pass }: RegisterRequest,
    ) -> Result<(), RegisterError> {
        server_response().await;
        let mut data = self.lock_data()?;
        data.server_failure()?;

        let id = data
            .try_register(name, email, pass)
            .ok_or(RegisterError::NameInUse)?;
        data.auth_user = Some(id);
        Ok(())
    }

    async fn get_user_info(&self, id: common::UserId) -> Result<UserInfo, GeneralError> {
        server_response().await;
        let mut data = self.lock_data()?;
        data.server_failure()?;

        data.get_user_info(id)
            .ok_or(GeneralError::UnknownEntity(EntityKind::User))
    }

    async fn set_avatar(&self, id: UserId, avatar: common::Avatar) -> Result<(), GeneralError> {
        server_response().await;
        let mut data = self.lock_data()?;
        data.server_failure()?;

        let my_id = data.require_auth()?;
        // for now, only allow getting your own avatar
        if my_id != id {
            return Err(GeneralError::Unauthorized);
        }

        data.auth_user.ok_or(GeneralError::Unauthorized)?;
        logging::warn!("Set avatar to user id={id:?}, avatar={avatar:?}. There's no server behind dummy API, so avatars will never load");
        Ok(())
    }

    async fn quest_history(&self, id: UserId, page: u32) -> Result<QuestHistoryPage, GeneralError> {
        server_response().await;
        let mut data = self.lock_data()?;
        data.server_failure()?;

        let my_id = data.require_auth()?;
        // for now, only allow getting your own quest history
        if my_id != id {
            return Err(GeneralError::Unauthorized);
        }

        let (data, page, total_pages) = extract_page::<QUEST_HISTORY_PAGE_SIZE, _, _>(
            data.get_quest_hisory(id),
            page,
            QuestHistoryRecord::clone,
        );
        Ok(QuestHistoryPage {
            data,
            page,
            total_pages,
        })
    }

    async fn user_quests(
        &self,
        user_id: UserId,
        page: u32,
    ) -> Result<common::UserOwnedQuestsPage, GeneralError> {
        server_response().await;
        let mut data = self.lock_data()?;
        data.server_failure()?;

        // let my_id = data.require_auth()?;

        let (data, page, total_pages) = extract_page::<USER_OWNED_QUESTS_PAGE_SIZE, _, _>(
            data.get_user_quests(user_id),
            page,
            |&quest_id| UserOwnedQuestRecord { id: quest_id },
        );
        Ok(UserOwnedQuestsPage {
            data,
            page,
            total_pages,
        })
    }

    async fn create_quest(&self) -> Result<QuestId, GeneralError> {
        server_response().await;
        let mut data = self.lock_data()?;
        data.server_failure()?;

        let auth_user = data.require_auth()?;

        Ok(data.create_quest(auth_user))
    }

    async fn get_quest_info(&self, quest_id: QuestId) -> Result<QuestInfo, GeneralError> {
        server_response().await;
        let mut data = self.lock_data()?;
        data.server_failure()?;

        data.quests
            .get(&quest_id)
            .cloned()
            .ok_or(GeneralError::UnknownEntity(EntityKind::Quest))
    }

    async fn set_quest_info(&self, quest_info: QuestInfo) -> Result<(), GeneralError> {
        server_response().await;
        let mut data = self.lock_data()?;
        data.server_failure()?;

        let auth_user = data.require_auth()?;

        let prev_info = data
            .quests
            .get_mut(&quest_info.id)
            .ok_or(GeneralError::UnknownEntity(EntityKind::Quest))?;

        if prev_info.owner != auth_user {
            return Err(GeneralError::Unauthorized);
        }

        if prev_info.owner != quest_info.owner {
            logging::error!("Changing quest's owner is not supported (?)");
            return Err(GeneralError::Unknown);
        }

        *prev_info = quest_info;

        Ok(())
    }

    async fn set_page_source(
        &self,
        quest_id: QuestId,
        page: u32,
        source: String,
    ) -> Result<(), GeneralError> {
        server_response().await;
        let mut data = self.lock_data()?;
        data.server_failure()?;

        let auth_user = data.require_auth()?;

        let info = data
            .quests
            .get(&quest_id)
            .ok_or(GeneralError::UnknownEntity(EntityKind::Quest))?;

        if info.owner != auth_user {
            return Err(GeneralError::Unauthorized);
        }

        data.set_quest_page(quest_id, page, source)
    }

    async fn get_page_source(&self, quest_id: QuestId, page: u32) -> Result<String, GeneralError> {
        server_response().await;
        let mut data = self.lock_data()?;
        data.server_failure()?;

        let auth_user = data.require_auth()?;

        let info = data
            .quests
            .get(&quest_id)
            .ok_or(GeneralError::UnknownEntity(EntityKind::Quest))?;

        if info.owner != auth_user {
            return Err(GeneralError::Unauthorized);
        }

        if info.pages == page {
            data.set_quest_page(quest_id, page, String::new())?;
        }

        data.get_quest_page(quest_id, page)
            .ok_or(GeneralError::UnknownEntity(EntityKind::QuestPage))
    }

    async fn start_quest(&self, quest_id: QuestId) -> Result<(), GameError> {
        server_response().await;
        let mut data = self.lock_data()?;
        data.server_failure()?;

        let _auth_user = data.require_auth()?;

        if let Some((active, _, _)) = data.active_quest {
            return if active == quest_id {
                Ok(())
            } else {
                Err(GameError::AlreadyActiveQuest)
            };
        }

        data.quests.contains_key(&quest_id);
        data.active_quest = Some((quest_id, 0, now()));
        Ok(())
    }

    fn active_quest(&self) -> Result<Option<(QuestId, u32, Timestamp)>, GeneralError> {
        Ok(self.lock_data()?.active_quest)
    }

    async fn quest_page(&self) -> Result<(QuestPage, u32), GameError> {
        server_response().await;
        let mut data = self.lock_data()?;
        data.server_failure()?;

        let Some((active_id, active_page, _)) = data.active_quest else {
            return Err(GameError::NoActiveQuest);
        };

        /*
        if page != active_page {
            return Err(GameError::PageOutOfOrder);
        }
        */

        let source = data
            .get_quest_page(active_id, active_page)
            .ok_or(GeneralError::UnknownEntity(EntityKind::QuestPage))?;

        Ok((
            common::parse_quest_page(source)
                .map_err(GeneralError::from)
                .map_err(GameError::from)?,
            active_page,
        ))
    }

    async fn answer_page(&self, answers: Box<[common::Answer]>) -> Result<(), GameError> {
        server_response().await;
        let mut data = self.lock_data()?;
        data.server_failure()?;

        let Some((active_id, active_page, _)) = data.active_quest else {
            return Err(GameError::NoActiveQuest);
        };

        logging::log!(
            "quest:{active_id}, page:{active_page}, answers: {:?}",
            answers
        );
        Ok(())
    }

    async fn finish_quest(&self) -> Result<(), GameError> {
        server_response().await;
        let mut data = self.lock_data()?;
        data.server_failure()?;

        let Some((active_id, active_page, started_at)) = data.active_quest else {
            return Err(GameError::NoActiveQuest);
        };

        let auth_user = data.require_auth()?;

        data.user_data
            .entry(auth_user)
            .or_default()
            .0
            .push(QuestHistoryRecord {
                user_id: auth_user,
                quest_id: active_id,
                started_at,
                finished_at: Some(now()),
                completed_pages: active_page,
            });

        Ok(())
    }
}
