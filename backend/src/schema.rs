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

diesel::allow_tables_to_appear_in_same_query!(avatars, users,);
