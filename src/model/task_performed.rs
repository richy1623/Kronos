use std::sync::{Arc, Mutex};

use diesel::{prelude::*, result::Error};

use crate::schema::task_performed;

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::schema::task_performed)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct TaskPerformed {
    pub date: String,
    pub task_id: i32,
    pub time_spent: i32,
}

impl TaskPerformed {
    pub fn get_task_by_task_id_and_date(
        task_id: i32,
        date: &str,
        connection: &Arc<Mutex<SqliteConnection>>,
    ) -> Option<Self> {
        let mut connection = connection.lock().unwrap();

        task_performed::table
            .filter(task_performed::date.eq(date))
            .filter(task_performed::task_id.eq(task_id))
            .select(TaskPerformed::as_select())
            .first(&mut *connection)
            .ok()
    }

    pub fn get_all_tasks_by_task_id(
        task_id: i32,
        connection: &Arc<Mutex<SqliteConnection>>,
    ) -> Vec<Self> {
        let mut connection = connection.lock().unwrap();

        task_performed::table
            .filter(task_performed::task_id.eq(task_id))
            .select(TaskPerformed::as_select())
            .load(&mut *connection)
            .unwrap_or(vec![])
    }

    pub fn get_all_tasks_by_date(
        date: &str,
        connection: &Arc<Mutex<SqliteConnection>>,
    ) -> Vec<Self> {
        let mut connection = connection.lock().unwrap();

        task_performed::table
            .filter(task_performed::date.eq(date))
            .select(TaskPerformed::as_select())
            .load(&mut *connection)
            .unwrap_or(vec![])
    }

    pub fn update_task_performed(
        task_performed: TaskPerformed,
        connection: &Arc<Mutex<SqliteConnection>>,
    ) -> Result<TaskPerformed, Error> {
        let mut connection = connection.lock().unwrap();

        diesel::update(task_performed::table)
            .filter(task_performed::task_id.eq(task_performed.task_id))
            .filter(task_performed::date.eq(task_performed.date))
            .set(task_performed::time_spent.eq(task_performed.time_spent))
            .returning(TaskPerformed::as_returning())
            .get_result(&mut *connection)
    }

    pub fn insert_task_performed(
        task_performed: TaskPerformed,
        connection: &Arc<Mutex<SqliteConnection>>,
    ) -> Result<TaskPerformed, Error> {
        let mut connection = connection.lock().unwrap();

        diesel::insert_into(task_performed::table)
            .values(task_performed)
            .returning(TaskPerformed::as_returning())
            .get_result(&mut *connection)
    }

    pub fn delete_task_performed(
        task_id: i32,
        date: &str,
        connection: &Arc<Mutex<SqliteConnection>>,
    ) -> Result<TaskPerformed, Error> {
        let mut connection = connection.lock().unwrap();

        diesel::delete(task_performed::table)
            .filter(task_performed::date.eq(date))
            .filter(task_performed::task_id.eq(task_id))
            .returning(TaskPerformed::as_returning())
            .get_result(&mut *connection)
    }

    pub fn delete_all_tasks_performed_by_task_id(
        task_id: i32,
        connection: &Arc<Mutex<SqliteConnection>>,
    ) -> Result<usize, Error> {
        let mut connection = connection.lock().unwrap();

        diesel::delete(task_performed::table)
            .filter(task_performed::task_id.eq(task_id))
            .execute(&mut *connection)
    }
}
