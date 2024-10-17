use diesel::{prelude::*, result::Error};

use crate::schema::task_performed;

#[derive(Queryable, Selectable, Insertable, Debug, PartialEq, Eq)]
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
        connection: &mut SqliteConnection,
    ) -> Option<Self> {
        task_performed::table
            .filter(task_performed::date.eq(date))
            .filter(task_performed::task_id.eq(task_id))
            .select(TaskPerformed::as_select())
            .first(&mut *connection)
            .ok()
    }

    pub fn get_all_tasks_by_task_id(task_id: i32, connection: &mut SqliteConnection) -> Vec<Self> {
        task_performed::table
            .filter(task_performed::task_id.eq(task_id))
            .select(TaskPerformed::as_select())
            .load(&mut *connection)
            .unwrap_or(vec![])
    }

    pub fn get_all_tasks_by_date(date: &str, connection: &mut SqliteConnection) -> Vec<Self> {
        task_performed::table
            .filter(task_performed::date.eq(date))
            .select(TaskPerformed::as_select())
            .load(&mut *connection)
            .unwrap_or(vec![])
    }

    pub fn update_task_performed(
        task_performed: TaskPerformed,
        connection: &mut SqliteConnection,
    ) -> Result<TaskPerformed, Error> {
        diesel::update(task_performed::table)
            .filter(task_performed::task_id.eq(task_performed.task_id))
            .filter(task_performed::date.eq(task_performed.date))
            .set(task_performed::time_spent.eq(task_performed.time_spent))
            .returning(TaskPerformed::as_returning())
            .get_result(&mut *connection)
    }

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

    pub fn delete_task_performed(
        task_id: i32,
        date: &str,
        connection: &mut SqliteConnection,
    ) -> Result<TaskPerformed, Error> {
        diesel::delete(task_performed::table)
            .filter(task_performed::date.eq(date))
            .filter(task_performed::task_id.eq(task_id))
            .returning(TaskPerformed::as_returning())
            .get_result(&mut *connection)
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
    use std::sync::{Arc, Mutex};

    use crate::{establish_connection, model::task::Task, schema::task};

    use super::*;
    use rstest::*;

    #[fixture]
    #[once]
    pub fn database_connection_fixture() -> Arc<Mutex<SqliteConnection>> {
        let connection = Arc::new(Mutex::new(establish_connection()));
        diesel::delete(task_performed::table)
            .execute(&mut *connection.lock().unwrap())
            .expect("Failed to delete all records from table `task_preformed`");
        diesel::delete(task::table)
            .execute(&mut *connection.lock().unwrap())
            .expect("Failed to delete all records from table `task`");
        connection
    }

    #[rstest]
    fn get_task_by_task_id_and_date(database_connection_fixture: &Arc<Mutex<SqliteConnection>>) {
        let mut database_connection_fixture = database_connection_fixture.lock().unwrap();

        let task = Task::create_task("task_performed", &mut database_connection_fixture).unwrap();

        let task_inserted = TaskPerformed::insert_task_performed(
            &TaskPerformed {
                date: String::from("2000-08-14"),
                task_id: task.id,
                time_spent: 21,
            },
            &mut database_connection_fixture,
        )
        .unwrap();

        let task = TaskPerformed::get_task_by_task_id_and_date(
            task.id,
            "2000-08-14",
            &mut database_connection_fixture,
        )
        .unwrap();

        assert_eq!(task, task_inserted);
    }

    #[rstest]
    fn get_all_tasks_by_task_id(database_connection_fixture: &Arc<Mutex<SqliteConnection>>) {
        let mut database_connection_fixture = database_connection_fixture.lock().unwrap();

        let task1 = Task::create_task("task_performed_with_id_1", &mut database_connection_fixture)
            .unwrap();
        let task2 = Task::create_task("task_performed_with_id_2", &mut database_connection_fixture)
            .unwrap();

        let task_inserted1 = TaskPerformed::insert_task_performed(
            &TaskPerformed {
                date: String::from("2000-08-14"),
                task_id: task1.id,
                time_spent: 21,
            },
            &mut database_connection_fixture,
        )
        .unwrap();

        let task_inserted2 = TaskPerformed::insert_task_performed(
            &TaskPerformed {
                date: String::from("2000-08-15"),
                task_id: task1.id,
                time_spent: 21,
            },
            &mut database_connection_fixture,
        )
        .unwrap();

        let task_inserted3 = TaskPerformed::insert_task_performed(
            &TaskPerformed {
                date: String::from("2000-08-16"),
                task_id: task2.id,
                time_spent: 21,
            },
            &mut database_connection_fixture,
        )
        .unwrap();

        let tasks =
            TaskPerformed::get_all_tasks_by_task_id(task1.id, &mut database_connection_fixture);

        assert_eq!(tasks, vec![task_inserted1, task_inserted2]);

        let tasks =
            TaskPerformed::get_all_tasks_by_task_id(task2.id, &mut database_connection_fixture);

        assert_eq!(tasks, vec![task_inserted3]);
    }

    #[rstest]
    fn get_all_tasks_by_date(database_connection_fixture: &Arc<Mutex<SqliteConnection>>) {
        let mut database_connection_fixture = database_connection_fixture.lock().unwrap();

        let task1 = Task::create_task(
            "task_performed_with_date_1",
            &mut database_connection_fixture,
        )
        .unwrap();
        let task2 = Task::create_task(
            "task_performed_with_date_2",
            &mut database_connection_fixture,
        )
        .unwrap();

        let task_inserted1 = TaskPerformed::insert_task_performed(
            &TaskPerformed {
                date: String::from("2000-11-05"),
                task_id: task1.id,
                time_spent: 21,
            },
            &mut database_connection_fixture,
        )
        .unwrap();

        let task_inserted2 = TaskPerformed::insert_task_performed(
            &TaskPerformed {
                date: String::from("1999-09-05"),
                task_id: task1.id,
                time_spent: 21,
            },
            &mut database_connection_fixture,
        )
        .unwrap();

        let task_inserted3 = TaskPerformed::insert_task_performed(
            &TaskPerformed {
                date: String::from("2000-11-05"),
                task_id: task2.id,
                time_spent: 21,
            },
            &mut database_connection_fixture,
        )
        .unwrap();

        let tasks =
            TaskPerformed::get_all_tasks_by_date("2000-11-05", &mut database_connection_fixture);

        assert_eq!(tasks, vec![task_inserted1, task_inserted3]);
        let tasks =
            TaskPerformed::get_all_tasks_by_date("1999-09-05", &mut database_connection_fixture);

        assert_eq!(tasks, vec![task_inserted2]);
    }

    #[rstest]
    fn update_task_performed(database_connection_fixture: &Arc<Mutex<SqliteConnection>>) {
        let mut database_connection_fixture = database_connection_fixture.lock().unwrap();

        let task =
            Task::create_task("task_performed_update", &mut database_connection_fixture).unwrap();

        let _task_inserted = TaskPerformed::insert_task_performed(
            &TaskPerformed {
                date: String::from("2000-08-14"),
                task_id: task.id,
                time_spent: 21,
            },
            &mut database_connection_fixture,
        )
        .unwrap();

        let updated_task = TaskPerformed {
            date: String::from("2000-08-14"),
            task_id: task.id,
            time_spent: 27,
        };

        let updated_task =
            TaskPerformed::update_task_performed(updated_task, &mut database_connection_fixture)
                .unwrap();

        assert_eq!(updated_task.time_spent, 27);
    }

    #[rstest]
    fn insert_task_performed(database_connection_fixture: &Arc<Mutex<SqliteConnection>>) {
        let mut database_connection_fixture = database_connection_fixture.lock().unwrap();

        let task =
            Task::create_task("task_performed_insert", &mut database_connection_fixture).unwrap();

        let task_to_insert = TaskPerformed {
            date: String::from("2000-08-14"),
            task_id: task.id,
            time_spent: 21,
        };

        let task_inserted =
            TaskPerformed::insert_task_performed(&task_to_insert, &mut database_connection_fixture)
                .unwrap();

        assert_eq!(task_inserted, task_to_insert);

        let task_inserted = TaskPerformed::get_task_by_task_id_and_date(
            task_to_insert.task_id,
            &task_inserted.date,
            &mut database_connection_fixture,
        )
        .unwrap();

        assert_eq!(task_inserted, task_to_insert);
    }

    #[rstest]
    fn delete_task_performed(database_connection_fixture: &Arc<Mutex<SqliteConnection>>) {
        let mut database_connection_fixture = database_connection_fixture.lock().unwrap();

        let task =
            Task::create_task("task_performed_delete", &mut database_connection_fixture).unwrap();

        let task_to_delete = TaskPerformed {
            date: String::from("2000-08-14"),
            task_id: task.id,
            time_spent: 21,
        };

        TaskPerformed::insert_task_performed(&task_to_delete, &mut database_connection_fixture)
            .unwrap();

        let delete_task_performed = TaskPerformed::delete_task_performed(
            task_to_delete.task_id,
            &task_to_delete.date,
            &mut database_connection_fixture,
        )
        .unwrap();

        assert_eq!(delete_task_performed, task_to_delete);

        let task_deleted = TaskPerformed::get_task_by_task_id_and_date(
            task_to_delete.task_id,
            &task_to_delete.date,
            &mut database_connection_fixture,
        );

        assert!(task_deleted.is_none());
    }

    #[rstest]
    fn delete_task_performed_no_such_task(
        database_connection_fixture: &Arc<Mutex<SqliteConnection>>,
    ) {
        let mut database_connection_fixture: std::sync::MutexGuard<'_, SqliteConnection> =
            database_connection_fixture.lock().unwrap();

        let delete_err = TaskPerformed::delete_task_performed(
            -1,
            &"2000-08-14",
            &mut database_connection_fixture,
        )
        .unwrap_err();

        assert_eq!(delete_err, Error::NotFound);
    }
}
