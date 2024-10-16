// @generated automatically by Diesel CLI.

diesel::table! {
    Tasks (id) {
        id -> Nullable<Integer>,
        name -> Nullable<Text>,
    }
}

diesel::table! {
    TasksPerformed (id) {
        id -> Nullable<Integer>,
        task_id -> Nullable<Integer>,
        date -> Nullable<Text>,
        time_spent -> Nullable<Integer>,
    }
}

diesel::joinable!(TasksPerformed -> Tasks (task_id));

diesel::allow_tables_to_appear_in_same_query!(
    Tasks,
    TasksPerformed,
);
