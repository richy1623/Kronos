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
    pub fn get_task_by_name(task_name: &str, connection: &mut SqliteConnection) -> Option<Self> {
        task::table
            .filter(task::name.eq(task_name))
            .select(Task::as_select())
            .first(connection)
            .ok()
    }

    pub fn get_task_by_id(task_id: i32, connection: &mut SqliteConnection) -> Option<Self> {
        task::table
            .filter(task::id.eq(task_id))
            .select(Task::as_select())
            .first(&mut *connection)
            .ok()
    }

    pub fn create_task(task_name: &str, connection: &mut SqliteConnection) -> Result<Self, Error> {
        diesel::insert_into(task::table)
            .values(task::name.eq(task_name))
            .returning(Task::as_returning())
            .get_result(&mut *connection)
    }

    pub fn update_task_last_used(
        task_name: &str,
        connection: &mut SqliteConnection,
    ) -> Result<Task, Error> {
        diesel::update(task::table)
            .filter(task::name.eq(task_name))
            .set(task::last_used.eq(chrono::Utc::now().timestamp() as i32))
            .returning(Task::as_returning())
            .get_result(&mut *connection)
    }

    pub fn fetch_most_recent_tasks(max_tasks: i32, connection: &mut SqliteConnection) -> Vec<Self> {
        task::table
            .order(task::last_used.desc())
            .limit(max_tasks.into())
            .select(Task::as_select())
            .load(&mut *connection)
            .unwrap_or(vec![])
    }

    pub fn get_or_create_task(
        task_name: &str,
        connection: &mut SqliteConnection,
    ) -> Result<Self, Error> {
        match Task::get_task_by_name(task_name, connection) {
            Some(task) => Ok(task),
            None => Task::create_task(task_name, connection),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Mutex},
        thread,
        time::Duration,
    };

    use crate::establish_connection;

    use super::*;
    use rstest::*;

    #[fixture]
    #[once]
    pub fn database_connection_fixture() -> Arc<Mutex<SqliteConnection>> {
        let connection = Arc::new(Mutex::new(establish_connection()));
        diesel::delete(task::table)
            .execute(&mut *connection.lock().unwrap())
            .expect("Failed to delete all records from table `task`");
        connection
    }

    #[rstest]
    fn get_task_by_name_valid(database_connection_fixture: &Arc<Mutex<SqliteConnection>>) {
        let mut database_connection_fixture = database_connection_fixture.lock().unwrap();

        let inserted_task =
            Task::create_task("get_task_by_name_valid", &mut database_connection_fixture).unwrap();
        let fetched_task =
            Task::get_task_by_name(&inserted_task.name, &mut database_connection_fixture);
        assert!(fetched_task.is_some());
        let fetched_task = fetched_task.unwrap();
        assert_eq!(fetched_task, inserted_task);
    }

    #[rstest]
    fn get_task_by_name_no_such_task_name(
        database_connection_fixture: &Arc<Mutex<SqliteConnection>>,
    ) {
        let mut database_connection_fixture = database_connection_fixture.lock().unwrap();

        assert!(
            Task::get_task_by_name("i_do_not_exist", &mut database_connection_fixture).is_none()
        );
    }

    #[rstest]
    fn get_task_by_id_valid(database_connection_fixture: &Arc<Mutex<SqliteConnection>>) {
        let mut database_connection_fixture = database_connection_fixture.lock().unwrap();

        let inserted_task =
            Task::create_task("get_task_by_id_valid", &mut database_connection_fixture).unwrap();
        let fetched_task = Task::get_task_by_id(inserted_task.id, &mut database_connection_fixture);
        assert!(fetched_task.is_some());
        let fetched_task = fetched_task.unwrap();
        assert_eq!(fetched_task, inserted_task);
    }

    #[rstest]
    fn get_task_by_id_no_such_task_name(
        database_connection_fixture: &Arc<Mutex<SqliteConnection>>,
    ) {
        let mut database_connection_fixture = database_connection_fixture.lock().unwrap();

        assert!(Task::get_task_by_id(-1, &mut database_connection_fixture).is_none());
    }

    #[rstest]
    fn create_task_valid(database_connection_fixture: &Arc<Mutex<SqliteConnection>>) {
        let mut database_connection_fixture = database_connection_fixture.lock().unwrap();

        let task: Result<Task, Error> =
            Task::create_task("task_name", &mut database_connection_fixture);
        assert!(task.is_ok());
        let task = task.unwrap();
        assert_eq!(task.name, "task_name");
    }

    #[rstest]
    fn create_task_invalid(database_connection_fixture: &Arc<Mutex<SqliteConnection>>) {
        let mut database_connection_fixture = database_connection_fixture.lock().unwrap();

        let _ = Task::create_task("task_name_repeated", &mut database_connection_fixture).unwrap();
        let task: Result<Task, Error> =
            Task::create_task("task_name_repeated", &mut database_connection_fixture);
        assert!(task.is_err());
        let task_err: Error = task.unwrap_err();
        matches!(
            task_err,
            Error::DatabaseError(diesel::result::DatabaseErrorKind::UniqueViolation, _)
        );
    }

    #[rstest]
    fn update_task_last_used(database_connection_fixture: &Arc<Mutex<SqliteConnection>>) {
        let mut database_connection_fixture = database_connection_fixture.lock().unwrap();

        let task_before_update =
            Task::create_task("task_name_to_update", &mut database_connection_fixture).unwrap();
        thread::sleep(Duration::from_millis(1000));
        let task_after_update =
            Task::update_task_last_used("task_name_to_update", &mut database_connection_fixture)
                .unwrap();

        assert!(task_before_update.last_used < task_after_update.last_used);
    }

    #[rstest]
    fn fetch_most_recent_tasks(database_connection_fixture: &Arc<Mutex<SqliteConnection>>) {
        let mut database_connection_fixture = database_connection_fixture.lock().unwrap();

        let task_1 =
            Task::create_task("task_name_recent_1", &mut database_connection_fixture).unwrap();
        let task_2 =
            Task::create_task("task_name_recent_2", &mut database_connection_fixture).unwrap();
        let task_3 =
            Task::create_task("task_name_recent_3", &mut database_connection_fixture).unwrap();

        thread::sleep(Duration::from_millis(1000));
        Task::update_task_last_used("task_name_recent_3", &mut database_connection_fixture)
            .unwrap();

        thread::sleep(Duration::from_millis(1000));
        Task::update_task_last_used("task_name_recent_2", &mut database_connection_fixture)
            .unwrap();

        let recent_tasks = Task::fetch_most_recent_tasks(3, &mut database_connection_fixture);
        assert_eq!(recent_tasks.len(), 3);
        assert_eq!(recent_tasks[0].id, task_2.id);
        assert_eq!(recent_tasks[1].id, task_3.id);
        assert_eq!(recent_tasks[2].id, task_1.id);

        let recent_tasks = Task::fetch_most_recent_tasks(2, &mut database_connection_fixture);
        assert_eq!(recent_tasks.len(), 2);
        assert_eq!(recent_tasks[0].id, task_2.id);
        assert_eq!(recent_tasks[1].id, task_3.id);
    }
}
