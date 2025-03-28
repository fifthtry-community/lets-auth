diesel::table! {
    fastn_folder (id) {
        id           -> Int8,
        guid         -> Text,
        name         -> Text,
        is_exception -> Bool,

        created_at   -> Timestamptz,
        updated_at   -> Timestamptz,
    }
}

diesel::table! {
    fastn_folder_user (id) {
        id         -> Int8,
        fguid      -> Text, // reference guid of the fastn_folder
        uid        -> Int8,

        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    fastn_folder_permission (id) {
        id          -> Int8,
        fguid       -> Text, // reference guid of the fastn_folder
        permission  -> Text,

        valid_since -> Timestamptz,
        valid_till  -> Timestamptz,
        two_factor  -> Bool,

        created_at  -> Timestamptz,
        updated_at  -> Timestamptz,
    }
}

diesel::table! {
    fastn_user (id) {
        id                   -> Int8,
        name                 -> Nullable<Text>,
        identity             -> Nullable<Text>,
        data                 -> Text,

        folders              -> Text,
        denormalized_folders -> Text,

        created_at           -> Timestamptz,
        updated_at           -> Timestamptz,
    }
}

diesel::table! {
    fastn_session (id) {
        id         -> Text,
        uid        -> Nullable<Int8>,
        data       -> Text,

        updated_at -> Timestamptz,
        created_at -> Timestamptz,
    }
}

diesel::joinable!(fastn_session -> fastn_user (uid));
diesel::joinable!(fastn_folder_user -> fastn_user (uid));
diesel::allow_tables_to_appear_in_same_query!(fastn_user, fastn_session,);
