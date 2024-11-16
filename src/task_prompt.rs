use std::sync::{Arc, Mutex};

use crate::model::{task::Task, task_performed::TaskPerformed};
use chrono::Local;
use diesel::SqliteConnection;

pub struct TaskPrompt {
    pub task_name_option: String,
    pub task_options: Vec<Task>,
    pub available_task_options: Vec<String>,
    db_connection: Arc<Mutex<SqliteConnection>>,
}

impl TaskPrompt {
    pub fn new(db_connection: Arc<Mutex<SqliteConnection>>) -> Self {
        let task_options = Task::fetch_most_recent_tasks(1000, &mut db_connection.lock().unwrap());
        let available_task_options = task_options.iter().map(|task| task.name.clone()).collect();
        TaskPrompt {
            task_name_option: String::new(),
            task_options,
            available_task_options,
            db_connection,
        }
    }

    pub fn update_task(&mut self, time_spent_minutes: i32) {
        let mut connection = &mut self.db_connection.lock().unwrap();

        let task = Task::get_or_create_task(&self.task_name_option, &mut connection)
            .expect("Get or Create Failed");

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
                    time_spent: time_spent_minutes,
                };
                TaskPerformed::insert_task_performed(&task_performed, &mut connection)
                    .expect("Update Failed");
            }
        }
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
