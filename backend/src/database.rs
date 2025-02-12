use common::{QuestId, QuestInfo, UserId, USER_OWNED_QUESTS_PAGE_SIZE};
use diesel::r2d2::{self, ConnectionManager};
use diesel::{BoolExpressionMethods, ExpressionMethods};
use diesel::{PgConnection, QueryDsl, RunQueryDsl};
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub struct Database {
    pool: DbPool,
}

impl Database {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn get_conn(&self) -> Option<r2d2::PooledConnection<ConnectionManager<PgConnection>>> {
        self.pool.get().ok()
    }

    pub async fn get_conn_to_death(
        &self,
    ) -> r2d2::PooledConnection<ConnectionManager<PgConnection>> {
        for _ in 0..500 {
            if let Some(conn) = self.get_conn() {
                return conn;
            }
            sleep(Duration::from_millis(10)).await;
        }
        panic!("connection pool is possibly dead");
    }

    pub async fn find_userid_pass_by_email_or_name(&self, input: &str) -> Option<(Uuid, String)> {
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

    pub async fn is_exist_user_avatar(&self, user_id: Uuid) -> Option<Uuid> {
        use crate::schema::avatars::dsl::*;
        let mut conn = self.get_conn_to_death().await;
        avatars
            .filter(id.eq(user_id))
            .select(id)
            .first::<Uuid>(&mut conn)
            .ok()
    }

    pub async fn get_user_avatar(&self, user_id: Uuid) -> Option<(Uuid, String, Vec<u8>)> {
        use crate::schema::avatars::dsl::*;
        let mut conn = self.get_conn_to_death().await;
        avatars
            .filter(id.eq(user_id))
            .select((id, content_type, image_data))
            .first::<(Uuid, String, Vec<u8>)>(&mut conn)
            .ok()
    }

    pub async fn find_user_by_email_or_name(
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

    pub async fn update_user_avatar(
        &self,
        user_id: Uuid,
        content_type_value: &str,
        new_avatar: &[u8],
    ) -> Option<()> {
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
            .ok()
            .map(|_| ())
    }

    pub async fn insert_user(
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

    pub async fn create_quest(&self, user_id: Uuid) -> Option<Uuid> {
        use crate::schema::quests::dsl::*;
        let mut conn = self.get_conn_to_death().await;
        let quest_uuid = Uuid::new_v4();
        diesel::insert_into(quests)
            .values((id.eq(quest_uuid), owner.eq(user_id), pages.eq(0)))
            .execute(&mut conn)
            .ok()
            .map(|_x| quest_uuid)
    }

    pub async fn get_quest(&self, quest_id: Uuid) -> Option<QuestInfo> {
        use crate::schema::quests::dsl::*;
        let mut conn = self.get_conn_to_death().await;
        let result = quests
            .filter(id.eq(quest_id))
            .select((id, owner, title, description, pages))
            .first::<(Uuid, Uuid, Option<String>, Option<String>, i32)>(&mut conn)
            .ok();
        result.map(
            |(got_id, got_owner, got_title, got_description, got_pages)| QuestInfo {
                id: QuestId(got_id),
                owner: UserId(got_owner),
                title: got_title.unwrap_or(String::from("")),
                description: got_description.unwrap_or(String::from("")),
                pages: got_pages as u32, //todo possibly not good, but i want to see guy who will create 2 billion pages
            },
        )
    }

    pub async fn update_quest(&self, quest_info: QuestInfo) -> Option<()> {
        // Some on success
        use crate::schema::quests::dsl::*;
        let mut conn = self.get_conn_to_death().await;
        let updated_rows = diesel::update(quests)
            .filter(id.eq(quest_info.id.0))
            .set((
                title.eq(quest_info.title),
                description.eq(quest_info.description),
            )) //todo look at me
            .execute(&mut conn)
            .ok();
        match updated_rows {
            Some(1) => Some(()),
            _ => None,
        }
    }

    pub async fn get_quest_page(&self, quest_id: Uuid, page_input: u32) -> Option<String> {
        use crate::schema::quests_pages::dsl::*;
        let mut conn = self.get_conn_to_death().await;
        quests_pages
            .filter(id.eq(quest_id).and(page.eq(page_input as i32)))
            .select(source)
            .first::<String>(&mut conn)
            .ok()
    }

    pub async fn update_quest_pages(&self, quest_info: &QuestInfo) -> Option<()> {
        // Some on success
        use crate::schema::quests::dsl::*;
        let mut conn = self.get_conn_to_death().await;
        let updated_rows = diesel::update(quests)
            .filter(id.eq(quest_info.id.0))
            .set(pages.eq(quest_info.pages as i32))
            .execute(&mut conn)
            .ok();
        match updated_rows {
            Some(1) => Some(()),
            _ => None,
        }
    }

    pub async fn get_owned_quests(&self, owner_id: Uuid, page: u32) -> Option<(Vec<Uuid>, u32)> {
        use crate::schema::quests::dsl::*;
        let mut conn = self.get_conn_to_death().await;

        let total_pages = ((quests
            .filter(owner.eq(owner_id))
            .count()
            .get_result::<i64>(&mut conn)
            .ok()? as f64)
            / (USER_OWNED_QUESTS_PAGE_SIZE as f64))
            .ceil() as u32;

        if total_pages == 0 || page > total_pages {
            return None;
        }

        Some((
            quests
                .filter(owner.eq(owner_id))
                .select(id)
                .offset((USER_OWNED_QUESTS_PAGE_SIZE * (page as usize)) as i64)
                .limit(USER_OWNED_QUESTS_PAGE_SIZE as i64)
                .load::<Uuid>(&mut conn)
                .ok()?,
            total_pages,
        ))
    }

    pub async fn update_quest_page(
        // or insert new
        &self,
        quest_id: QuestId,
        page_input: u32,
        source_input: String,
        time_limit_seconds_input: Option<u32>,
    ) -> Option<()> {
        // Some on success
        use crate::schema::quests_pages::dsl::*;
        let mut conn = self.get_conn_to_death().await;

        let source_input_clone = source_input.clone();
        let time_limit_seconds_input = time_limit_seconds_input.map(|x| x as i32);
        let time_limit_seconds_input_clone = time_limit_seconds_input.clone();

        diesel::insert_into(quests_pages)
            .values((
                id.eq(quest_id.0),
                page.eq(page_input as i32),
                source.eq(source_input),
                time_limit_seconds.eq(time_limit_seconds_input),
            ))
            .on_conflict((id, page))
            .do_update()
            .set((
                source.eq(source_input_clone),
                time_limit_seconds.eq(time_limit_seconds_input_clone),
            ))
            .execute(&mut conn)
            .ok()
            .map(|_x| ())
    }
}
