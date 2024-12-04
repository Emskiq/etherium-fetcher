// @generated automatically by Diesel CLI.

diesel::table! {
    transactions (transaction_hash) {
        transaction_hash -> Text,
        transaction_status -> Bool,
        block_hash -> Text,
        block_number -> Int8,
        from -> Text,
        to -> Nullable<Text>,
        contract_address -> Nullable<Text>,
        logs_count -> Int8,
        input -> Text,
        value -> Text,
    }
}

diesel::table! {
    users_searches (id) {
        id -> Int4,
        username -> Text,
        transaction_hash -> Text,
    }
}

diesel::joinable!(users_searches -> transactions (transaction_hash));

diesel::allow_tables_to_appear_in_same_query!(
    transactions,
    users_searches,
);
