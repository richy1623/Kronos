use diesel::{prelude::*, result::Error};

use crate::schema::task_performed;

/// A struct to represent a task performed.
#[derive(Queryable, Selectable, Insertable, Debug, PartialEq, Eq, Clone)]
#[diesel(table_name = crate::schema::task_performed)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct TaskPerformed {
    /// The date the task was performed.
    pub date: String,
    /// The ID of the task that was performed.
    pub task_id: i32,
    /// The time spent performing the task.
    pub time_spent: i32,
    /// If the current task performed has been synced with the external server
    pub is_synced_to_server: bool,
}

impl TaskPerformed {
    /// Retrieves a task performed by its `task_id` and `date`.
    ///
    /// # Arguments
    ///
    /// * `task_id`: The ID of the task that was performed.
    /// * `date`: The date the task was performed.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `TaskPerformed` if found, or `None` otherwise.
    pub fn get_task_by_task_id_and_date(
        task_id: i32,
        date: &str,
        connection: &mut SqliteConnection,
    ) -> Option<Self> {
        task_performed::table
            .filter(task_performed::date.eq(date))
            .filter(task_performed::task_id.eq(task_id))
            .select(TaskPerformed::as_select())
            .first(&mut *connection)
            .ok()
    }

    /// Retrieves all tasks performed by a given `task_id`.
    ///
    /// # Arguments
    ///
    /// * `task_id`: The ID of the task.
    ///
    /// # Returns
    ///
    /// A `Vec` of `TaskPerformed` structs if found, or an empty vector otherwise.
    pub fn get_all_tasks_by_task_id(task_id: i32, connection: &mut SqliteConnection) -> Vec<Self> {
        task_performed::table
            .filter(task_performed::task_id.eq(task_id))
            .select(TaskPerformed::as_select())
            .load(&mut *connection)
            .unwrap_or(vec![])
    }

    /// Retrieves all tasks performed on a given `date`.
    ///
    /// # Arguments
    ///
    /// * `date`: The date the task was performed.
    ///
    /// # Returns
    ///
    /// A `Vec` of `TaskPerformed` structs if found, or an empty vector otherwise.
    pub fn get_all_tasks_by_date(date: &str, connection: &mut SqliteConnection) -> Vec<Self> {
        task_performed::table
            .filter(task_performed::date.eq(date))
            .select(TaskPerformed::as_select())
            .load(&mut *connection)
            .unwrap_or(vec![])
    }

    /// Updates a `TaskPerformed` record.
    ///
    /// # Arguments
    ///
    /// * `task_performed`: The updated `TaskPerformed` record.
    ///
    /// # Returns
    ///
    /// A `Result` containing the updated `TaskPerformed` if successful, or an `Error` otherwise.
    pub fn update_task_performed(
        task_performed: &TaskPerformed,
        connection: &mut SqliteConnection,
    ) -> Result<TaskPerformed, Error> {
        diesel::update(task_performed::table)
            .filter(task_performed::task_id.eq(task_performed.task_id))
            .filter(task_performed::date.eq(task_performed.date.clone()))
            .set(task_performed::time_spent.eq(task_performed.time_spent))
            .returning(TaskPerformed::as_returning())
            .get_result(&mut *connection)
    }

    /// Inserts a new `TaskPerformed` record.
    ///
    /// # Arguments
    ///
    /// * `task_performed`: The `TaskPerformed` to insert.
    ///
    /// # Returns
    ///
    /// A `Result` containing the inserted `TaskPerformed` if successful, or an `Error` otherwise.
    pub fn insert_task_performed(
        task_performed: &TaskPerformed,
        connection: &mut SqliteConnection,
    ) -> Result<TaskPerformed, Error> {
        // TODO should task_performed consume the calling task?
        diesel::insert_into(task_performed::table)
            .values(task_performed)
            .returning(TaskPerformed::as_returning())
            .get_result(&mut *connection)
    }

    pub fn insert_or_overwrite_task_performed(
        task_performed: &TaskPerformed,
        connection: &mut SqliteConnection,
    ) -> Result<TaskPerformed, Error> {
        let optional_task_performed = TaskPerformed::get_task_by_task_id_and_date(
            task_performed.task_id,
            &task_performed.date,
            connection,
        );
        match optional_task_performed {
            Some(_) => TaskPerformed::update_task_performed(&task_performed, connection),
            None => TaskPerformed::insert_task_performed(task_performed, connection),
        }
    }

    /// Deletes a `TaskPerformed` record.
    ///
    /// # Arguments
    ///
    /// * `task_id`: The ID of the task that was performed.
    /// * `date`: The date the task was performed.
    ///
    /// Removes a single `TaskPerformed` record from the database.
    ///
    /// # Returns
    ///
    /// A `Result` containing the number of affected rows if successful, or an `Error` otherwise.
    pub fn delete_task_performed(
        task_id: i32,
        date: &str,
        connection: &mut SqliteConnection,
    ) -> Result<usize, Error> {
        diesel::delete(task_performed::table)
            .filter(task_performed::date.eq(date))
            .filter(task_performed::task_id.eq(task_id))
            .execute(&mut *connection)
    }

    // TODO Do we need this method? We can probably just call cascade delete on a Task
    // pub fn delete_all_tasks_performed_by_task_id(
    //     task_id: i32,
    //     connection: &mut SqliteConnection,
    // ) -> Result<usize, Error> {
    //     diesel::delete(task_performed::table)
    //         .filter(task_performed::task_id.eq(task_id))
    //         .execute(&mut *connection)
    // }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        sync::{Arc, Mutex},
    };

    use crate::{model::task::Task, schema::task, MIGRATIONS};

    use super::*;
    use diesel_migrations::MigrationHarness;
    use rstest::*;

    const DATABASE_URL: &str = "test/task_performed_test_database.db";

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

        diesel::delete(task_performed::table)
            .execute(&mut *connection.lock().unwrap())
            .expect("Failed to delete all records from table `task_preformed`");
        diesel::delete(task::table)
            .execute(&mut *connection.lock().unwrap())
            .expect("Failed to delete all records from table `task`");
        connection
    }

    #[rstest]
    fn get_task_by_task_id_and_date(connection: &Arc<Mutex<SqliteConnection>>) {
        let mut connection = connection.lock().unwrap();

        let task = Task::create_task("task_performed", &mut connection).unwrap();

        let task_inserted = TaskPerformed::insert_task_performed(
            &TaskPerformed {
                date: String::from("2000-08-14"),
                task_id: task.id,
                time_spent: 21,
                is_synced_to_server: false,
            },
            &mut connection,
        )
        .unwrap();

        let task =
            TaskPerformed::get_task_by_task_id_and_date(task.id, "2000-08-14", &mut connection)
                .unwrap();

        assert_eq!(task, task_inserted);
    }

    #[rstest]
    fn get_all_tasks_by_task_id(connection: &Arc<Mutex<SqliteConnection>>) {
        let mut connection = connection.lock().unwrap();

        let task1 = Task::create_task("task_performed_with_id_1", &mut connection).unwrap();
        let task2 = Task::create_task("task_performed_with_id_2", &mut connection).unwrap();

        let task_inserted1 = TaskPerformed::insert_task_performed(
            &TaskPerformed {
                date: String::from("2000-08-14"),
                task_id: task1.id,
                time_spent: 21,
                is_synced_to_server: false,
            },
            &mut connection,
        )
        .unwrap();

        let task_inserted2 = TaskPerformed::insert_task_performed(
            &TaskPerformed {
                date: String::from("2000-08-15"),
                task_id: task1.id,
                time_spent: 21,
                is_synced_to_server: false,
            },
            &mut connection,
        )
        .unwrap();

        let task_inserted3 = TaskPerformed::insert_task_performed(
            &TaskPerformed {
                date: String::from("2000-08-16"),
                task_id: task2.id,
                time_spent: 21,
                is_synced_to_server: false,
            },
            &mut connection,
        )
        .unwrap();

        let tasks = TaskPerformed::get_all_tasks_by_task_id(task1.id, &mut connection);

        assert_eq!(tasks, vec![task_inserted1, task_inserted2]);

        let tasks = TaskPerformed::get_all_tasks_by_task_id(task2.id, &mut connection);

        assert_eq!(tasks, vec![task_inserted3]);
    }

    #[rstest]
    fn get_all_tasks_by_date(connection: &Arc<Mutex<SqliteConnection>>) {
        let mut connection = connection.lock().unwrap();

        let task1 = Task::create_task("task_performed_with_date_1", &mut connection).unwrap();
        let task2 = Task::create_task("task_performed_with_date_2", &mut connection).unwrap();

        let task_inserted1 = TaskPerformed::insert_task_performed(
            &TaskPerformed {
                date: String::from("2000-11-05"),
                task_id: task1.id,
                time_spent: 21,
                is_synced_to_server: false,
            },
            &mut connection,
        )
        .unwrap();

        let task_inserted2 = TaskPerformed::insert_task_performed(
            &TaskPerformed {
                date: String::from("1999-09-05"),
                task_id: task1.id,
                time_spent: 21,
                is_synced_to_server: false,
            },
            &mut connection,
        )
        .unwrap();

        let task_inserted3 = TaskPerformed::insert_task_performed(
            &TaskPerformed {
                date: String::from("2000-11-05"),
                task_id: task2.id,
                time_spent: 21,
                is_synced_to_server: false,
            },
            &mut connection,
        )
        .unwrap();

        let tasks = TaskPerformed::get_all_tasks_by_date("2000-11-05", &mut connection);

        assert_eq!(tasks, vec![task_inserted1, task_inserted3]);
        let tasks = TaskPerformed::get_all_tasks_by_date("1999-09-05", &mut connection);

        assert_eq!(tasks, vec![task_inserted2]);
    }

    #[rstest]
    fn update_task_performed(connection: &Arc<Mutex<SqliteConnection>>) {
        let mut connection = connection.lock().unwrap();

        let task = Task::create_task("task_performed_update", &mut connection).unwrap();

        let _task_inserted = TaskPerformed::insert_task_performed(
            &TaskPerformed {
                date: String::from("2000-08-14"),
                task_id: task.id,
                time_spent: 21,
                is_synced_to_server: false,
            },
            &mut connection,
        )
        .unwrap();

        let updated_task = TaskPerformed {
            date: String::from("2000-08-14"),
            task_id: task.id,
            time_spent: 27,
            is_synced_to_server: false,
        };

        let updated_task =
            TaskPerformed::update_task_performed(&updated_task, &mut connection).unwrap();

        assert_eq!(updated_task.time_spent, 27);

        let current_task = TaskPerformed::get_task_by_task_id_and_date(
            updated_task.task_id,
            &updated_task.date,
            &mut connection,
        )
        .expect("No such task after update");

        assert_eq!(current_task.time_spent, 27);
    }

    #[rstest]
    fn insert_task_performed(connection: &Arc<Mutex<SqliteConnection>>) {
        let mut connection = connection.lock().unwrap();

        let task = Task::create_task("task_performed_insert", &mut connection).unwrap();

        let task_to_insert = TaskPerformed {
            date: String::from("2000-08-14"),
            task_id: task.id,
            time_spent: 21,
            is_synced_to_server: false,
        };

        let task_inserted =
            TaskPerformed::insert_task_performed(&task_to_insert, &mut connection).unwrap();

        assert_eq!(task_inserted, task_to_insert);

        let task_inserted = TaskPerformed::get_task_by_task_id_and_date(
            task_to_insert.task_id,
            &task_inserted.date,
            &mut connection,
        )
        .unwrap();

        assert_eq!(task_inserted, task_to_insert);
    }

    #[rstest]
    fn delete_task_performed(connection: &Arc<Mutex<SqliteConnection>>) {
        let mut connection = connection.lock().unwrap();

        let task = Task::create_task("task_performed_delete", &mut connection).unwrap();

        let task_to_delete = TaskPerformed {
            date: String::from("2000-08-14"),
            task_id: task.id,
            time_spent: 21,
            is_synced_to_server: false,
        };

        TaskPerformed::insert_task_performed(&task_to_delete, &mut connection).unwrap();

        let delete_task_performed = TaskPerformed::delete_task_performed(
            task_to_delete.task_id,
            &task_to_delete.date,
            &mut connection,
        )
        .unwrap();

        assert_eq!(delete_task_performed, 1);

        let task_deleted = TaskPerformed::get_task_by_task_id_and_date(
            task_to_delete.task_id,
            &task_to_delete.date,
            &mut connection,
        );

        assert!(task_deleted.is_none());
    }

    #[rstest]
    fn delete_task_performed_no_such_task(connection: &Arc<Mutex<SqliteConnection>>) {
        let mut connection: std::sync::MutexGuard<'_, SqliteConnection> =
            connection.lock().unwrap();

        let delete_task_performed =
            TaskPerformed::delete_task_performed(-1, &"2000-08-14", &mut connection).unwrap();

        assert_eq!(delete_task_performed, 0);
    }

    #[rstest]
    fn test_insert_or_overwrite_task_performed(connection: &Arc<Mutex<SqliteConnection>>) {
        let mut connection: std::sync::MutexGuard<'_, SqliteConnection> =
            connection.lock().unwrap();

        let task = Task::create_task("task_performed_to_overwrite", &mut connection).unwrap();

        assert!(TaskPerformed::get_task_by_task_id_and_date(
            task.id,
            &String::from("2000-08-14"),
            &mut connection
        )
        .is_none());

        let task_to_insert = TaskPerformed {
            date: String::from("2000-08-14"),
            task_id: task.id,
            time_spent: 5,
            is_synced_to_server: false,
        };

        let inserted_task =
            TaskPerformed::insert_or_overwrite_task_performed(&task_to_insert, &mut connection)
                .unwrap();

        assert_eq!(inserted_task, task_to_insert);

        let task_to_overwrite = TaskPerformed {
            date: String::from("2000-08-14"),
            task_id: task.id,
            time_spent: 10,
            is_synced_to_server: false,
        };

        let overwritten_task =
            TaskPerformed::insert_or_overwrite_task_performed(&task_to_overwrite, &mut connection)
                .unwrap();

        assert_eq!(overwritten_task, task_to_overwrite);
    }
}
