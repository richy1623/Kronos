use std::{
    cmp::Reverse,
    sync::{Arc, Mutex},
};

use crate::model::{task::Task, task_performed::TaskPerformed};
use chrono::NaiveDate;
use diesel::SqliteConnection;

#[derive(Clone)]
pub struct TaskListItem {
    pub task_performed: TaskPerformed,
    pub task_name: String,
}
pub struct TaskList {
    db_connection: Arc<Mutex<SqliteConnection>>,
    date: NaiveDate,
    tasks_for_date: Vec<TaskListItem>,
}

impl TaskList {
    pub fn new(db_connection: Arc<Mutex<SqliteConnection>>, date: NaiveDate) -> Self {
        let mut task_list = TaskList {
            db_connection,
            date,
            tasks_for_date: Vec::new(),
        };
        task_list.change_date(date);
        task_list
    }

    pub fn change_date(&mut self, date: NaiveDate) {
        let get_all_tasks_by_date = TaskPerformed::get_all_tasks_by_date(
            &date.to_string(),
            &mut self.db_connection.lock().unwrap(),
        );
        let mut task_for_date = get_all_tasks_by_date
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
            .collect::<Vec<TaskListItem>>();
        task_for_date.sort_by_key(|task| Reverse(task.task_performed.time_spent));

        // task_for_date.sort_by(compare);

        self.date = date;
        self.tasks_for_date = task_for_date;
    }

    pub fn list_all_tasks_performed(&self) -> &Vec<TaskListItem> {
        &self.tasks_for_date
    }

    pub fn add_task(&mut self, task_name: &str, time_spent: i32) {
        //TODO: this does not handle duplicate task names
        let mut connection = self.db_connection.lock().unwrap();

        let task = Task::get_or_create_task(task_name, &mut connection)
            .expect("Failed to get or create task");

        let task_performed = TaskPerformed::insert_or_overwrite_task_performed(
            &TaskPerformed {
                date: self.date.to_string(),
                task_id: task.id,
                time_spent,
            },
            &mut connection,
        )
        .expect("todo");

        self.tasks_for_date.push(TaskListItem {
            task_name: task_name.to_string(),
            task_performed: task_performed,
        });
    }

    pub fn delete_task_performed(self, task_name: &str, date: &NaiveDate) {
        let mut connection = self.db_connection.lock().unwrap();

        let task = Task::get_task_by_name(task_name, &mut connection);

        let task = match task {
            Some(task) => task,
            None => return,
        };

        TaskPerformed::delete_task_performed(task.id, &date.to_string(), &mut connection).unwrap();
    }

    pub fn update_task_performed(&mut self, task_id: i32, task_name: &str, time_spent: i32) {
        let mut connection = self.db_connection.lock().unwrap();

        // Check to see if a task exists with that name for the day

        let task_to_update_index = self
            .tasks_for_date
            .iter()
            .position(|task_list_item| task_list_item.task_performed.task_id == task_id)
            .expect("Updating a task should always have a valid task_id");

        let task_to_update = self.tasks_for_date.swap_remove(task_to_update_index);

        let mut new_task = TaskPerformed {
            task_id,
            time_spent,
            date: self.date.to_string(),
        };

        if task_to_update.task_name != task_name {
            // Check if we need to update an existing task
            let task_item_with_same_name = self
                .tasks_for_date
                .iter()
                .filter(|task_list_item| {
                    task_list_item.task_name == task_name
                        && task_list_item.task_performed.task_id != task_id
                })
                .next();

            match task_item_with_same_name {
                Some(task) => {
                    new_task.time_spent = new_task.time_spent + task.task_performed.time_spent;
                    new_task.task_id = task.task_performed.task_id;
                }
                None => {
                    // Fetch the correct new task id
                    new_task.task_id = Task::get_or_create_task(task_name, &mut connection)
                        .unwrap()
                        .id
                }
            };

            // Remove the old task_performed from the db
            TaskPerformed::delete_task_performed(task_id, &self.date.to_string(), &mut connection)
                .unwrap();
        }

        // Insert or update the new task
        TaskPerformed::insert_or_overwrite_task_performed(&new_task, &mut connection).unwrap();
        self.tasks_for_date.push(TaskListItem {
            task_performed: new_task,
            task_name: task_name.to_string(),
        });
        self.tasks_for_date
            .sort_by_key(|task| Reverse(task.task_performed.time_spent));
    }

    pub fn fetch_most_recent_task_names(&self, max_tasks: i32) -> Vec<String> {
        let mut connection = self.db_connection.lock().unwrap();
        Task::fetch_most_recent_tasks(max_tasks, &mut connection)
            .into_iter()
            .map(|task| task.name)
            .collect()
    }
}
