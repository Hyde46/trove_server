// @generated automatically by Diesel CLI.

diesel::table! {
    api_token (id) {
        id -> Int4,
        token -> Text,
        user_id_fk -> Int4,
        revoked -> Bool,
        created_at -> Timestamp,
    }
}

diesel::table! {
    trove (id) {
        id -> Int4,
        trove_text -> Text,
        user_id_fk -> Int4,
        created_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        first_name -> Text,
        last_name -> Text,
        email -> Text,
        pw_hash -> Text,
        verified -> Bool,
        created_at -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    api_token,
    trove,
    users,
);
