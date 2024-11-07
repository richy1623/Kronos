use std::sync::{Arc, Mutex};

use crate::model::{task::Task, task_performed::TaskPerformed};
use chrono::Local;
use diesel::SqliteConnection;
use regex::Regex;

pub struct TaskPrompt {
    db_connection: Arc<Mutex<SqliteConnection>>,
}

impl TaskPrompt {
    pub fn new(db_connection: Arc<Mutex<SqliteConnection>>) -> Self {
        TaskPrompt { db_connection }
    }

    pub fn get_all_matching_tasks(self, search_string: &str) -> Vec<Task> {
        // self.db_connection
        let regex = Regex::new(&format!(
            "{}{}{}",
            ".*",
            search_string
                .chars()
                .map(|character| format!("{}{}", character, ".*"))
                .collect::<String>(),
            ".*"
        ))
        .unwrap();

        let most_recent_tasks =
            Task::fetch_most_recent_tasks(1000, &mut self.db_connection.lock().unwrap());

        most_recent_tasks
            .into_iter()
            .filter(|task| regex.is_match(&task.name))
            .take(10)
            .collect()
    }

    pub fn update_task(self, task_name: &str, time_spent_minutes: i32) {
        let mut connection = self.db_connection.lock().unwrap();

        let task =
            Task::get_or_create_task(task_name, &mut connection).expect("Get or Create Failed");

        let current_date = Local::now().date_naive().to_string();

        let task_performed =
            TaskPerformed::get_task_by_task_id_and_date(task.id, &current_date, &mut connection);

        match task_performed {
            Some(mut task_performed) => {
                task_performed.time_spent += time_spent_minutes;
                TaskPerformed::update_task_performed(task_performed, &mut connection)
                    .expect("Insert Failed");
            }
            None => {
                let task_performed = TaskPerformed {
                    date: current_date,
                    task_id: task.id,
                    time_spent: 0,
                };
                TaskPerformed::insert_task_performed(&task_performed, &mut connection)
                    .expect("Update Failed");
            }
        }
    }

    // TODO decide if we want to use a file to track last logged time (this means that we will be able to track time over the application crashing/shutting down) -> we just store the timestamp of the last update/startup if it was a previous day
}
