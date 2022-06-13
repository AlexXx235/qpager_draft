// @generated automatically by Diesel CLI.

diesel::table! {
    chat_members (id) {
        id -> Int4,
        chat_id -> Int4,
        user_id -> Int4,
    }
}

diesel::table! {
    chat_messages (id) {
        id -> Int4,
        text -> Text,
        timestamp -> Timestamp,
        key_index -> Int4,
        chat_id -> Int4,
        sender_id -> Int4,
    }
}

diesel::table! {
    chats (id) {
        id -> Int4,
        name -> Text,
    }
}

diesel::table! {
    private_messages (id) {
        id -> Int4,
        text -> Text,
        timestamp -> Timestamp,
        key_index -> Int4,
        receiver_id -> Int4,
        sender_id -> Int4,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        login -> Text,
        password -> Text,
        salt -> Text,
    }
}

diesel::joinable!(chat_members -> chats (chat_id));
diesel::joinable!(chat_members -> users (user_id));
diesel::joinable!(chat_messages -> chats (chat_id));
diesel::joinable!(chat_messages -> users (sender_id));

diesel::allow_tables_to_appear_in_same_query!(
    chat_members,
    chat_messages,
    chats,
    private_messages,
    users,
);
