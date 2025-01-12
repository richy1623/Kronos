// @generated automatically by Diesel CLI.

diesel::table! {
    task (id) {
        id -> Integer,
        name -> Text,
        last_used -> Integer,
    }
}

diesel::table! {
    task_performed (date, task_id) {
        date -> Text,
        task_id -> Integer,
        time_spent -> Integer,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    task,
    task_performed,
);
