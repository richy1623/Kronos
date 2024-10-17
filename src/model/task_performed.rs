use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::schema::TaskPerformed)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct TaskPerformed {
    pub date: String,
    pub task_id: i32,
    pub time_spent: i32,
}
