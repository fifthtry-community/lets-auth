diesel::table! {
    fastn_folder (guid) {
        // guid is the primary key
        guid -> Text,
        name -> Text,
        // kind is "Folder" by default, but can be "Team", "Client", "Playlist", "Project", etc.
        kind -> Nullable<Text>,
        // for the root folder, parents will be an empty array
        // this is a JSON encoded list of strings
        parents -> Text,

        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    fastn_user (id) {
        id -> Int8,
        name -> Nullable<Text>,
        identity -> Nullable<Text>,
        data -> Text,

        folders -> Text,
        denormalized_folders -> Text,

        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    fastn_session (id) {
        id -> Text,
        uid -> Nullable<Int8>,
        data -> Text,

        updated_at -> Timestamptz,
        created_at -> Timestamptz,
    }
}

diesel::joinable!(fastn_session -> fastn_user (uid));
diesel::allow_tables_to_appear_in_same_query!(fastn_user, fastn_session,);
