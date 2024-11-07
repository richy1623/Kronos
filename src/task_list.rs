use std::sync::{Arc, Mutex};

use crate::model::{task::Task, task_performed::TaskPerformed};
use chrono::NaiveDate;
use diesel::SqliteConnection;

pub struct TaskList {
    db_connection: Arc<Mutex<SqliteConnection>>,
}

impl TaskList {
    pub fn new(db_connection: Arc<Mutex<SqliteConnection>>) -> Self {
        TaskList { db_connection }
    }

    pub fn list_all_tasks_performed(self, date: &NaiveDate) -> Vec<TaskPerformed> {
        TaskPerformed::get_all_tasks_by_date(
            &date.to_string(),
            &mut self.db_connection.lock().unwrap(),
        )
    }

    pub fn add_task(self, task_name: &str, date: &NaiveDate, time_spent: i32) {
        let mut connection = self.db_connection.lock().unwrap();

        let task = Task::get_or_create_task(task_name, &mut connection)
            .expect("Failed to get or create task");

        TaskPerformed::insert_or_update_task_performed(
            &TaskPerformed {
                date: date.to_string(),
                task_id: task.id,
                time_spent,
            },
            &mut connection,
        )
        .expect("todo");
    }

    pub fn delete_task(self, task_name: &str, date: &NaiveDate) {
        let mut connection = self.db_connection.lock().unwrap();

        let task = Task::get_task_by_name(task_name, &mut connection);

        let task = match task {
            Some(task) => task,
            None => return,
        };

        TaskPerformed::delete_task_performed(task.id, &date.to_string(), &mut connection).unwrap();
    }
}
