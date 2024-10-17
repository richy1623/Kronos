use std::sync::{Arc, Mutex};

use crate::schema::task;
use diesel::{prelude::*, result::Error};

#[derive(Queryable, Selectable, Insertable, Debug, PartialEq, Eq)]
#[diesel(table_name = crate::schema::task)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Task {
    pub id: i32,
    pub name: String,
    pub last_used: i32,
}

impl Task {
    pub fn get_task_by_name(
        task_name: &str,
        connection: &Arc<Mutex<SqliteConnection>>,
    ) -> Option<Self> {
        let mut connection = connection.lock().unwrap();

        task::table
            .filter(task::name.eq(task_name))
            .select(Task::as_select())
            .first(&mut *connection)
            .ok()
    }

    pub fn get_task_by_id(task_id: i32, connection: &Arc<Mutex<SqliteConnection>>) -> Option<Self> {
        let mut connection = connection.lock().unwrap();

        task::table
            .filter(task::id.eq(task_id))
            .select(Task::as_select())
            .first(&mut *connection)
            .ok()
    }

    pub fn create_task(
        task_name: &str,
        connection: &Arc<Mutex<SqliteConnection>>,
    ) -> Result<Task, Error> {
        let mut connection = connection.lock().unwrap();

        diesel::insert_into(task::table)
            .values(task::name.eq(task_name))
            .returning(Task::as_returning())
            .get_result(&mut *connection)
    }

    pub fn update_task_last_used(
        task_name: &str,
        connection: &Arc<Mutex<SqliteConnection>>,
    ) -> Result<Task, Error> {
        let mut connection = connection.lock().unwrap();

        diesel::update(task::table)
            .filter(task::name.eq(task_name))
            .set(task::last_used.eq(chrono::Utc::now().timestamp() as i32))
            .returning(Task::as_returning())
            .get_result(&mut *connection)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::establish_connection;

    use super::*;
    use rstest::*;

    #[fixture]
    #[once]
    pub fn database_connection_fixture() -> Arc<Mutex<SqliteConnection>> {
        use crate::schema::task::dsl::*;
        let connection = Arc::new(Mutex::new(establish_connection()));
        diesel::delete(task)
            .execute(&mut *connection.lock().unwrap())
            .expect("Failed to delete all records from table `task`");
        connection
    }

    #[rstest]
    fn get_task_by_name_valid(database_connection_fixture: &Arc<Mutex<SqliteConnection>>) {
        let inserted_task =
            Task::create_task("get_task_by_name_valid", database_connection_fixture).unwrap();
        let fetched_task = Task::get_task_by_name(&inserted_task.name, database_connection_fixture);
        assert!(fetched_task.is_some());
        let fetched_task = fetched_task.unwrap();
        assert_eq!(fetched_task, inserted_task);
    }

    #[rstest]
    fn get_task_by_name_no_such_task_name(
        database_connection_fixture: &Arc<Mutex<SqliteConnection>>,
    ) {
        assert!(Task::get_task_by_name("i_do_not_exist", database_connection_fixture).is_none());
    }

    #[rstest]
    fn get_task_by_id_valid(database_connection_fixture: &Arc<Mutex<SqliteConnection>>) {
        let inserted_task =
            Task::create_task("get_task_by_id_valid", database_connection_fixture).unwrap();
        let fetched_task = Task::get_task_by_id(inserted_task.id, database_connection_fixture);
        assert!(fetched_task.is_some());
        let fetched_task = fetched_task.unwrap();
        assert_eq!(fetched_task, inserted_task);
    }

    #[rstest]
    fn get_task_by_id_no_such_task_name(
        database_connection_fixture: &Arc<Mutex<SqliteConnection>>,
    ) {
        assert!(Task::get_task_by_id(-1, database_connection_fixture).is_none());
    }

    #[rstest]
    fn create_task_valid(database_connection_fixture: &Arc<Mutex<SqliteConnection>>) {
        let task: Result<Task, Error> = Task::create_task("task_name", database_connection_fixture);
        assert!(task.is_ok());
        let task = task.unwrap();
        assert_eq!(task.name, "task_name");
    }

    #[rstest]
    fn create_task_invalid(database_connection_fixture: &Arc<Mutex<SqliteConnection>>) {
        let _ = Task::create_task("task_name_repeated", database_connection_fixture);
        let task: Result<Task, Error> =
            Task::create_task("task_name_repeated", database_connection_fixture);
        assert!(task.is_err());
        let task_err: Error = task.unwrap_err();
        matches!(
            task_err,
            Error::DatabaseError(diesel::result::DatabaseErrorKind::UniqueViolation, _)
        );
    }
}
