use crate::schema::{task, task_performed};
use diesel::{prelude::*, result::Error};
use regex::Regex;

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

    pub fn get_or_create_task_with_update(
        task_name: &str,
        connection: &mut SqliteConnection,
    ) -> Result<Self, Error> {
        let task = Task::get_task_by_name(task_name, connection);
        match task {
            Some(task) => {
                Task::update_task_last_used(&task.name, connection)
            }
            None => Task::create_task(task_name, connection),
        }
    }

    pub fn get_all_matching_tasks(
        search_string: &str,
        connection: &mut SqliteConnection,
    ) -> Vec<Task> {
        let regex = Task::create_task_search_regex(search_string);

        let most_recent_tasks = Task::fetch_most_recent_tasks(1000, connection);

        most_recent_tasks
            .into_iter()
            .filter(|task| regex.is_match(&task.name))
            .take(10)
            .collect()
    }

    fn create_task_search_regex(search_string: &str) -> Regex {
        Regex::new(&format!(
            "(?i).*{}",
            search_string
                .chars()
                .map(|character| format!("{}.*", regex::escape(&character.to_string())))
                .collect::<String>()
        ))
        .unwrap()
    }

    pub fn filter_all_matching_tasks<'a>(
        task_names: &'a Vec<String>,
        search_string: &str,
    ) -> Vec<&'a String> {
        let regex = Task::create_task_search_regex(search_string);
        task_names
            .into_iter()
            .filter(|task| regex.is_match(&task))
            .take(10)
            .collect()
    }

    //TODO call this on delete_task_performed or on startup
    pub fn delete_unused_tasks(connection: &mut SqliteConnection) -> Result<usize, Error> {
        diesel::delete(task::table)
            .filter(
                task::id.ne_all(
                    task_performed::table
                        .select(task_performed::task_id)
                        .filter(task_performed::task_id.eq(task::id)),
                ),
            )
            .execute(connection)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        sync::{Arc, Mutex},
        thread,
        time::Duration,
    };

    use crate::{model::task_performed::TaskPerformed, MIGRATIONS};

    use super::*;
    use diesel_migrations::MigrationHarness;
    use rstest::*;

    const DATABASE_URL: &str = "test/task_test_database.db";

    #[fixture]
    #[once]
    pub fn connection() -> Arc<Mutex<SqliteConnection>> {
        fs::create_dir_all("test").unwrap();
        let connection = Arc::new(Mutex::new(
            SqliteConnection::establish(&DATABASE_URL)
                .unwrap_or_else(|_| panic!("Error connecting to {}", DATABASE_URL)),
        ));
        connection
            .lock()
            .unwrap()
            .run_pending_migrations(MIGRATIONS)
            .unwrap();

        diesel::delete(task::table)
            .execute(&mut *connection.lock().unwrap())
            .expect("Failed to delete all records from table `task`");
        connection
    }

    #[rstest]
    fn get_task_by_name_valid(connection: &Arc<Mutex<SqliteConnection>>) {
        let mut connection = connection.lock().unwrap();

        let inserted_task =
            Task::create_task("get_task_by_name_valid", &mut connection).unwrap();
        let fetched_task =
            Task::get_task_by_name(&inserted_task.name, &mut connection);
        assert!(fetched_task.is_some());
        let fetched_task = fetched_task.unwrap();
        assert_eq!(fetched_task, inserted_task);
    }

    #[rstest]
    fn get_task_by_name_no_such_task_name(
        connection: &Arc<Mutex<SqliteConnection>>,
    ) {
        let mut connection = connection.lock().unwrap();

        assert!(
            Task::get_task_by_name("i_do_not_exist", &mut connection).is_none()
        );
    }

    #[rstest]
    fn get_task_by_id_valid(connection: &Arc<Mutex<SqliteConnection>>) {
        let mut connection = connection.lock().unwrap();

        let inserted_task =
            Task::create_task("get_task_by_id_valid", &mut connection).unwrap();
        let fetched_task = Task::get_task_by_id(inserted_task.id, &mut connection);
        assert!(fetched_task.is_some());
        let fetched_task = fetched_task.unwrap();
        assert_eq!(fetched_task, inserted_task);
    }

    #[rstest]
    fn get_task_by_id_no_such_task_name(
        connection: &Arc<Mutex<SqliteConnection>>,
    ) {
        let mut connection = connection.lock().unwrap();

        assert!(Task::get_task_by_id(-1, &mut connection).is_none());
    }

    #[rstest]
    fn create_task_valid(connection: &Arc<Mutex<SqliteConnection>>) {
        let mut connection = connection.lock().unwrap();

        let task: Result<Task, Error> =
            Task::create_task("task_name", &mut connection);
        assert!(task.is_ok());
        let task = task.unwrap();
        assert_eq!(task.name, "task_name");
    }

    #[rstest]
    fn create_task_invalid(connection: &Arc<Mutex<SqliteConnection>>) {
        let mut connection = connection.lock().unwrap();

        let _ = Task::create_task("task_name_repeated", &mut connection).unwrap();
        let task: Result<Task, Error> =
            Task::create_task("task_name_repeated", &mut connection);
        assert!(task.is_err());
        let task_err: Error = task.unwrap_err();
        matches!(
            task_err,
            Error::DatabaseError(diesel::result::DatabaseErrorKind::UniqueViolation, _)
        );
    }

    #[rstest]
    fn update_task_last_used(connection: &Arc<Mutex<SqliteConnection>>) {
        let mut connection = connection.lock().unwrap();

        let task_before_update =
            Task::create_task("task_name_to_update", &mut connection).unwrap();
        thread::sleep(Duration::from_millis(1000));
        let task_after_update =
            Task::update_task_last_used("task_name_to_update", &mut connection)
                .unwrap();

        assert!(task_before_update.last_used < task_after_update.last_used);
    }

    #[rstest]
    fn get_or_create_task(connection: &Arc<Mutex<SqliteConnection>>) {
        let mut connection = connection.lock().unwrap();

        let task =
            Task::create_task("get_or_create_task", &mut connection).unwrap();
        thread::sleep(Duration::from_millis(1000));

        let same_task_fetched = Task::get_or_create_task(
            "get_or_create_task",
            &mut connection,
        )
        .unwrap();
        assert_eq!(task.id, same_task_fetched.id);
        assert_eq!(task.last_used, same_task_fetched.last_used);

        let new_task_fetched = Task::get_or_create_task(
            "get_or_create_task_new",
            &mut connection,
        )
        .unwrap();
        assert_ne!(task.id, new_task_fetched.id);
        assert_ne!(task.last_used, new_task_fetched.last_used);
    }

    #[rstest]
    fn get_or_create_task_with_update(connection: &Arc<Mutex<SqliteConnection>>) {
        let mut connection = connection.lock().unwrap();

        let task_before_update = Task::create_task(
            "get_or_create_task_with_update",
            &mut connection,
        )
        .unwrap();

        thread::sleep(Duration::from_millis(1000));

        let task_after_update = Task::get_or_create_task_with_update(
            "get_or_create_task_with_update",
            &mut connection,
        )
        .unwrap();
        assert_eq!(task_before_update.id, task_after_update.id);
        assert!(task_before_update.last_used < task_after_update.last_used, 
            "Expected task before to have an earlier time than after the update. Times were [task_before_update: {}, task_after_update: {}]",
            task_before_update.last_used,task_after_update.last_used);

        let new_task = Task::get_or_create_task_with_update(
            "get_or_create_task_with_update_new",
            &mut connection,
        )
        .unwrap();
        assert_ne!(task_before_update.id, new_task.id);
        assert!(task_before_update.last_used < new_task.last_used);
    }

    #[rstest]
    fn fetch_most_recent_tasks(connection: &Arc<Mutex<SqliteConnection>>) {
        let mut connection = connection.lock().unwrap();

        let task_1 =
            Task::create_task("task_name_recent_1", &mut connection).unwrap();
        let task_2 =
            Task::create_task("task_name_recent_2", &mut connection).unwrap();
        let task_3 =
            Task::create_task("task_name_recent_3", &mut connection).unwrap();

        thread::sleep(Duration::from_millis(1000));
        Task::update_task_last_used("task_name_recent_3", &mut connection)
            .unwrap();

        thread::sleep(Duration::from_millis(1000));
        Task::update_task_last_used("task_name_recent_2", &mut connection)
            .unwrap();

        let recent_tasks = Task::fetch_most_recent_tasks(3, &mut connection);
        assert_eq!(recent_tasks.len(), 3);
        assert_eq!(recent_tasks[0].id, task_2.id);
        assert_eq!(recent_tasks[1].id, task_3.id);
        assert_eq!(recent_tasks[2].id, task_1.id);

        let recent_tasks = Task::fetch_most_recent_tasks(2, &mut connection);
        assert_eq!(recent_tasks.len(), 2);
        assert_eq!(recent_tasks[0].id, task_2.id);
        assert_eq!(recent_tasks[1].id, task_3.id);
    }

    
    #[rstest]
    fn test_get_all_matching_tasks(connection: &Arc<Mutex<SqliteConnection>>){
        let mut connection: std::sync::MutexGuard<'_, SqliteConnection> =
            connection.lock().unwrap();
        let task_1 = Task::create_task("task_match_1", &mut connection).unwrap();
        let task_2 = Task::create_task("task_match_2", &mut connection).unwrap();
        let task_3 = Task::create_task("task_match_3", &mut connection).unwrap();

        assert_eq!(Task::get_all_matching_tasks("task_match", &mut connection), vec![task_3, task_2, task_1]);
        assert_eq!(Task::get_all_matching_tasks("no_match", &mut connection), vec![]);
    }

    #[rstest]
    #[case("", r"(?i).*")]
    #[case("cat", r"(?i).*c.*a.*t.*")]
    #[case("fix", r"(?i).*f.*i.*x.*")]
    #[case("call", r"(?i).*c.*a.*l.*l.*")]
    #[case("xyz", r"(?i).*x.*y.*z.*")]
    fn test_create_task_search_regex(#[case] search_string: &str, #[case] expected_pattern: &str) {
        // Call the function to generate the regex
        let regex = Task::create_task_search_regex(search_string);

        // Check the generated regex pattern
        assert_eq!(regex.to_string(), expected_pattern);
    }

    #[rstest]
    fn delete_unused_tasks(connection: &Arc<Mutex<SqliteConnection>>) {
        let mut connection: std::sync::MutexGuard<'_, SqliteConnection> =
            connection.lock().unwrap();

        diesel::delete(task::table)
            .execute(&mut *connection)
            .expect("Failed to delete all records from table `task`");

        let task_to_delete =
            Task::create_task("orphaned_task", &mut connection).unwrap();
        let task_to_save =
            Task::create_task("relevant_task", &mut connection).unwrap();

        TaskPerformed::insert_task_performed(
            &TaskPerformed {
                date: String::from("1999-09-05"),
                task_id: task_to_save.id,
                time_spent: 21,
            },
            &mut connection,
        )
        .unwrap();

        let deleted_tasks = Task::delete_unused_tasks(&mut connection).unwrap();

        assert_eq!(deleted_tasks, 1);

        assert_eq!(
            Task::get_task_by_id(task_to_delete.id, &mut connection),
            None
        );

        assert_eq!(
            Task::get_task_by_id(task_to_save.id, &mut connection),
            Some(task_to_save)
        );
    }

    #[rstest]
    #[case("cat", vec![
        "Complete the cat report",
        "Buy a caterpillar plushie",
        "Catalog new books"
    ])]
    #[case("fix", vec!["Fix the faucet"])]
    #[case("call m", vec!["Call mom"])]
    #[case("out", vec!["Take out the trash"])]
    #[case("xyz", vec![])] // No matches
    fn test_filter_all_matching_tasks(
        #[case] search_string: &str,
        #[case] expected_matches: Vec<&str>,
    ) {
        let tasks = vec![
            String::from("Complete the cat report"),
            String::from("Buy a caterpillar plushie"),
            String::from("Catalog new books"),
            String::from("Take out the trash"),
            String::from("Fix the faucet"),
            String::from("Call mom"),
        ];

        // Call the function to filter tasks
        let matches = Task::filter_all_matching_tasks(&tasks, search_string);

        // Extract the string values of the matches for comparison
        let match_strings: Vec<&str> = matches.iter().map(|s| s.as_str()).collect();

        // Assert results
        assert_eq!(
            match_strings, expected_matches,
            "Failed for search_string: '{}'. Got: {:?}, Expected: {:?}",
            search_string, match_strings, expected_matches
        );
    }
}
