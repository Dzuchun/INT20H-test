// @generated automatically by Diesel CLI.

diesel::table! {
    avatars (id) {
        id -> Uuid,
        image_data -> Bytea,
        #[max_length = 255]
        content_type -> Varchar,
    }
}

diesel::table! {
    quests (id) {
        id -> Uuid,
        owner -> Uuid,
        title -> Nullable<Text>,
        description -> Nullable<Text>,
        pages -> Int4,
    }
}

diesel::table! {
    quests_applied (user_id, quest_id) {
        user_id -> Uuid,
        quest_id -> Uuid,
        started_at -> Timestamp,
        finished_at -> Nullable<Timestamp>,
        completed_pages -> Int4,
    }
}

diesel::table! {
    quests_pages (id, page) {
        id -> Uuid,
        page -> Int4,
        source -> Text,
        time_limit_seconds -> Nullable<Int4>,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 32]
        name -> Varchar,
        #[max_length = 320]
        email -> Varchar,
        #[max_length = 255]
        password_hash -> Varchar,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    avatars,
    quests,
    quests_applied,
    quests_pages,
    users,
);
