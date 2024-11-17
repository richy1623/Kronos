use std::sync::{Arc, Mutex};

use crate::model::{task::Task, task_performed::TaskPerformed};
use chrono::NaiveDate;
use diesel::SqliteConnection;
pub struct TaskListItem {
    pub task_performed: TaskPerformed,
    pub task_name: String,
}
pub struct TaskList {
    db_connection: Arc<Mutex<SqliteConnection>>,
    date: NaiveDate,
    task_for_date: Vec<TaskListItem>,
}

impl TaskList {
    pub fn new(db_connection: Arc<Mutex<SqliteConnection>>, date: NaiveDate) -> Self {
        let mut task_list = TaskList {
            db_connection,
            date,
            task_for_date: Vec::new(),
        };
        task_list.change_date(date);
        task_list
    }

    pub fn change_date(&mut self, date: NaiveDate) {
        let get_all_tasks_by_date = TaskPerformed::get_all_tasks_by_date(
            &date.to_string(),
            &mut self.db_connection.lock().unwrap(),
        );
        let task_for_date = get_all_tasks_by_date
            .into_iter()
            .map(|task_performed| TaskListItem {
                task_name: Task::get_task_by_id(
                    task_performed.task_id.clone(),
                    &mut self.db_connection.lock().unwrap(),
                )
                .unwrap()
                .name,
                task_performed,
            })
            .collect();

        self.date = date;
        self.task_for_date = task_for_date;
    }

    pub fn list_all_tasks_performed(&self) -> &Vec<TaskListItem> {
        &self.task_for_date
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
