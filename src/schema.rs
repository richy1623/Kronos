// @generated automatically by Diesel CLI.

diesel::table! {
    task (id) {
        id -> Integer,
        name -> Text,
        is_synced_to_server -> Bool,
        last_used -> Integer,
    }
}

diesel::table! {
    task_performed (date, task_id) {
        date -> Text,
        task_id -> Integer,
        time_spent -> Integer,
        is_synced_to_server -> Bool,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    task,
    task_performed,
);
