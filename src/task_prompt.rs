use std::sync::{Arc, Mutex, RwLock};

use crate::model::{
    latest_task::{LatestTask, LatestTaskManager},
    task::Task,
    task_performed::TaskPerformed,
};
use chrono::Local;
use diesel::SqliteConnection;

pub struct TaskPrompt {
    pub task_name_option: String,
    pub task_options: Vec<Task>,
    pub available_task_options: Vec<String>,
    latest_task_performed: LatestTask,
    db_connection: Arc<Mutex<SqliteConnection>>,
    latest_task_manager: Arc<RwLock<LatestTaskManager>>,
}

impl TaskPrompt {
    pub fn new(
        db_connection: Arc<Mutex<SqliteConnection>>,
        latest_task_manager: Arc<RwLock<LatestTaskManager>>,
    ) -> Self {
        let task_options = Task::fetch_most_recent_tasks(1000, &mut db_connection.lock().unwrap());
        let available_task_options = task_options.iter().map(|task| task.name.clone()).collect();
        let latest_task_performed = latest_task_manager
            .read()
            .unwrap()
            .get_latest_task_performed();
        TaskPrompt {
            task_name_option: task_options
                .first()
                .map(|task| task.name.clone())
                .unwrap_or(String::new()),
            task_options,
            available_task_options,
            latest_task_performed,
            db_connection,
            latest_task_manager,
        }
    }

    pub fn get_time_spent_minutes(&self) -> i32 {
        (Local::now() - self.latest_task_performed.date_time_performed)
            .num_minutes()
            .try_into()
            .unwrap()
    }

    pub fn update_task(&mut self) {
        let mut connection = &mut self.db_connection.lock().unwrap();

        let task =
            Task::get_or_create_task_with_update(&self.task_name_option, &mut connection).unwrap();

        let current_date = Local::now().date_naive().to_string();

        let task_performed =
            TaskPerformed::get_task_by_task_id_and_date(task.id, &current_date, &mut connection);

        let time_spent_minutes: i32 = self.get_time_spent_minutes();

        match task_performed {
            Some(mut task_performed) => {
                task_performed.time_spent += time_spent_minutes;
                TaskPerformed::update_task_performed(&task_performed, &mut connection)
                    .expect("Update Failed");
            }
            None => {
                let task_performed = TaskPerformed {
                    date: current_date,
                    task_id: task.id,
                    time_spent: time_spent_minutes,
                };
                TaskPerformed::insert_task_performed(&task_performed, &mut connection)
                    .expect("Insert Failed");
            }
        }

        self.latest_task_performed = self
            .latest_task_manager
            .write()
            .unwrap()
            .update_latest_task_performed(Some(task.id));
    }

    // pub fn set_task_name_option(mut self, task_name: &str) {
    //     self.task_name_option = task_name.to_owned();
    //     self.available_task_options =
    //         Task::filter_all_matching_tasks(&self.task_options, task_name)
    //             .iter()
    //             .map(|task| task.name.clone())
    //             .collect();
    // }

    // TODO decide if we want to use a file to track last logged time (this means that we will be able to track time over the application crashing/shutting down) -> we just store the timestamp of the last update/startup if it was a previous day
}

#[cfg(test)]
mod tests {
    use crate::{
        MIGRATIONS,
        model::latest_task::LatestTaskManager,
        schema::{task, task_performed},
        settings::Settings,
    };

    use super::*;
    use chrono::{Days, TimeDelta};
    use diesel::{Connection, ExpressionMethods, RunQueryDsl, SelectableHelper};
    use diesel_migrations::MigrationHarness;
    use rstest::*;
    use serial_test::serial;
    use tempfile::TempDir;

    #[fixture]
    #[once]
    pub fn db_connection() -> (Arc<Mutex<SqliteConnection>>, TempDir) {
        let temp_dir = TempDir::new().unwrap();

        let path_to_db = temp_dir.path().join("task_prompt_test_database.db");
        let path_to_db = path_to_db.to_str().unwrap();

        let db_connection = Arc::new(Mutex::new(
            SqliteConnection::establish(path_to_db)
                .unwrap_or_else(|_| panic!("Error connecting to {}", path_to_db)),
        ));
        db_connection
            .lock()
            .unwrap()
            .run_pending_migrations(MIGRATIONS)
            .unwrap();

        // Clear any existing tasks in tables
        diesel::delete(task::table)
            .execute(&mut *db_connection.lock().unwrap())
            .expect("Failed to delete all records from table `task`");
        diesel::delete(task_performed::table)
            .execute(&mut *db_connection.lock().unwrap())
            .expect("Failed to delete all records from table `task_performed`");

        (db_connection, temp_dir)
    }

    #[fixture]
    #[once]
    pub fn latest_task_manager(
        db_connection: &(Arc<Mutex<SqliteConnection>>, TempDir),
    ) -> Arc<RwLock<LatestTaskManager>> {
        // TODO
        let (_db_connection, temp_dir) = db_connection;
        Arc::new(RwLock::new(LatestTaskManager::new(Arc::new(Mutex::new(
            Settings::from_dir(temp_dir.path().to_path_buf()),
        )))))
    }

    #[rstest]
    #[serial]
    fn test_create_new_task_prompt(
        db_connection: &(Arc<Mutex<SqliteConnection>>, TempDir),
        latest_task_manager: &Arc<RwLock<LatestTaskManager>>,
    ) {
        let (db_connection, _temp_dir) = db_connection;
        let mut connection = db_connection.lock().unwrap();
        let mut latest_task_manager_setup = latest_task_manager.write().unwrap();

        let task_1 = Task::create_task("task_prompt_create_1", &mut connection).unwrap();
        let task_2 = diesel::insert_into(task::table)
            .values((
                task::name.eq("task_prompt_create_2"),
                task::last_used.eq::<i32>(
                    Local::now()
                        .checked_add_days(Days::new(1))
                        .unwrap()
                        .timestamp()
                        .try_into()
                        .unwrap(),
                ),
            ))
            .returning(Task::as_returning())
            .get_result(&mut *connection)
            .unwrap();
        latest_task_manager_setup.update_latest_task_performed(Some(task_2.id));

        std::mem::drop(connection);
        std::mem::drop(latest_task_manager_setup);

        let task_prompt = TaskPrompt::new(db_connection.clone(), latest_task_manager.clone());

        assert!(task_prompt.available_task_options.contains(&task_2.name));
        assert!(task_prompt.available_task_options.contains(&task_1.name));
        assert_eq!(&task_prompt.task_name_option, &task_2.name);
        assert!(task_prompt.task_options.contains(&task_2));
        assert!(task_prompt.task_options.contains(&task_1));
        assert_eq!(
            task_prompt.latest_task_performed,
            latest_task_manager
                .read()
                .unwrap()
                .get_latest_task_performed()
        );
    }

    #[rstest]
    #[serial]
    fn test_get_time_spent_minutes(
        db_connection: &(Arc<Mutex<SqliteConnection>>, TempDir),
        latest_task_manager: &Arc<RwLock<LatestTaskManager>>,
    ) {
        let (db_connection, _temp_dir) = db_connection;
        let current_time = Local::now();

        let earlier = current_time
            .checked_sub_signed(TimeDelta::minutes(5))
            .unwrap();

        let task_prompt = TaskPrompt {
            task_name_option: String::new(),
            task_options: vec![],
            available_task_options: vec![],
            latest_task_performed: LatestTask {
                task_id: None,
                date_time_performed: earlier,
            },
            db_connection: db_connection.clone(),
            latest_task_manager: latest_task_manager.clone(),
        };

        assert_eq!(task_prompt.get_time_spent_minutes(), 5);
    }

    #[rstest]
    #[serial]
    fn test_update_task_with_exiting_task(
        db_connection: &(Arc<Mutex<SqliteConnection>>, TempDir),
        latest_task_manager: &Arc<RwLock<LatestTaskManager>>,
    ) {
        let (db_connection, _temp_dir) = db_connection;
        let connection = db_connection.clone();
        let mut connection = connection.lock().unwrap();
        let mut latest_task_manager_setup = latest_task_manager.write().unwrap();

        let task = Task::create_task("update_task_1", &mut connection).unwrap();

        latest_task_manager_setup.update_latest_task_performed(None);

        let current_date = Local::now().date_naive().to_string();

        std::mem::drop(connection);
        std::mem::drop(latest_task_manager_setup);

        let mut task_prompt = TaskPrompt::new(db_connection.clone(), latest_task_manager.clone());
        // Update task time spent
        let latest_task = LatestTask {
            task_id: None,
            date_time_performed: Local::now()
                .checked_sub_signed(TimeDelta::minutes(5))
                .unwrap(),
        };
        task_prompt.latest_task_performed = latest_task.clone();
        // Update to an existing task
        task_prompt.task_name_option = String::from("update_task_1");

        task_prompt.update_task();

        // Validate
        std::mem::drop(task_prompt);
        let mut connection = db_connection.lock().unwrap();
        let latest_task_manager_validate = latest_task_manager.read().unwrap();

        assert_eq!(
            TaskPerformed::get_task_by_task_id_and_date(task.id, &current_date, &mut connection)
                .unwrap(),
            TaskPerformed {
                date: current_date.clone(),
                task_id: task.id,
                time_spent: 5,
            }
        );

        assert!(
            latest_task_manager_validate
                .get_latest_task_performed()
                .date_time_performed
                > latest_task.date_time_performed
        );
        assert_eq!(
            latest_task_manager_validate
                .get_latest_task_performed()
                .task_id,
            Some(task.id)
        );
    }

    #[rstest]
    #[serial]
    fn test_update_task_with_exiting_task_no_task_performed(
        db_connection: &(Arc<Mutex<SqliteConnection>>, TempDir),
        latest_task_manager: &Arc<RwLock<LatestTaskManager>>,
    ) {
        let (db_connection, _temp_dir) = db_connection;
        let mut connection = db_connection.lock().unwrap();
        let mut latest_task_manager_setup = latest_task_manager.write().unwrap();

        let current_date = Local::now().date_naive().to_string();

        let task = Task::create_task("update_task_2", &mut connection).unwrap();
        let task_performed = TaskPerformed {
            date: current_date.clone(),
            task_id: task.id,
            time_spent: 5,
        };
        TaskPerformed::insert_task_performed(&task_performed, &mut connection).unwrap();
        latest_task_manager_setup.update_latest_task_performed(None);

        std::mem::drop(connection);
        std::mem::drop(latest_task_manager_setup);

        // Perform prompt update
        let mut task_prompt = TaskPrompt::new(db_connection.clone(), latest_task_manager.clone());

        task_prompt.task_name_option = String::from("update_task_2");

        // Update task time spent
        let latest_task = LatestTask {
            task_id: None,
            date_time_performed: Local::now()
                .checked_sub_signed(TimeDelta::minutes(5))
                .unwrap(),
        };
        task_prompt.latest_task_performed = latest_task.clone();
        // Update to an existing task
        task_prompt.update_task();

        // Validate
        std::mem::drop(task_prompt);
        let latest_task_manager_validate = latest_task_manager.read().unwrap();
        let mut connection = db_connection.lock().unwrap();

        assert_eq!(
            TaskPerformed::get_task_by_task_id_and_date(task.id, &current_date, &mut connection)
                .unwrap(),
            TaskPerformed {
                date: current_date.clone(),
                task_id: task.id,
                time_spent: 10,
            }
        );
        assert!(
            latest_task_manager_validate
                .get_latest_task_performed()
                .date_time_performed
                > latest_task.date_time_performed
        );
        assert_eq!(
            latest_task_manager_validate
                .get_latest_task_performed()
                .task_id,
            Some(task.id)
        );
    }

    #[rstest]
    #[serial]
    fn test_update_task_with_new_task(
        db_connection: &(Arc<Mutex<SqliteConnection>>, TempDir),
        latest_task_manager: &Arc<RwLock<LatestTaskManager>>,
    ) {
        let (db_connection, _temp_dir) = db_connection;
        let mut latest_task_manager_setup = latest_task_manager.write().unwrap();
        latest_task_manager_setup.update_latest_task_performed(None);

        let current_date = Local::now().date_naive().to_string();

        std::mem::drop(latest_task_manager_setup);

        let mut task_prompt = TaskPrompt::new(db_connection.clone(), latest_task_manager.clone());

        task_prompt.task_name_option = String::from("update_task_3");

        // Update task time spent
        let latest_task = LatestTask {
            task_id: None,
            date_time_performed: Local::now()
                .checked_sub_signed(TimeDelta::minutes(5))
                .unwrap(),
        };
        task_prompt.latest_task_performed = latest_task.clone();

        task_prompt.update_task();

        // Validate
        std::mem::drop(task_prompt);

        let mut connection = db_connection.lock().unwrap();
        let latest_task_manager_validate = latest_task_manager.read().unwrap();

        let task = Task::get_task_by_name("update_task_3", &mut connection).unwrap();

        assert_eq!(
            TaskPerformed::get_task_by_task_id_and_date(task.id, &current_date, &mut connection)
                .unwrap(),
            TaskPerformed {
                date: current_date.clone(),
                task_id: task.id,
                time_spent: 5,
            }
        );
        assert!(
            latest_task_manager_validate
                .get_latest_task_performed()
                .date_time_performed
                > latest_task.date_time_performed
        );
        assert_eq!(
            latest_task_manager_validate
                .get_latest_task_performed()
                .task_id
                .unwrap(),
            task.id
        );
    }
}
