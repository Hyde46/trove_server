// @generated automatically by Diesel CLI.

diesel::table! {
    api_token (id) {
        id -> Int4,
        token -> Text,
        user_id -> Int4,
        revoked -> Bool,
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

diesel::joinable!(api_token -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(api_token, users,);