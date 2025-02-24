use std::{
    cmp::Reverse,
    sync::{Arc, Mutex},
};

use crate::model::{task::Task, task_performed::TaskPerformed};
use chrono::NaiveDate;
use diesel::SqliteConnection;

#[derive(Clone, PartialEq, Debug)]
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
        let mut connection = self.db_connection.lock().unwrap();

        let task = Task::get_or_create_task(task_name, &mut connection)
            .expect("Failed to get or create task");

        let task_performed_index = self
            .tasks_for_date
            .iter()
            .position(|task_list_item| task_list_item.task_name == task_name);

        let task_performed = match task_performed_index {
            Some(task_performed_index) => {
                // Remove and extract the old TaskListItem from the tasks_for_date
                let mut task_performed = self
                    .tasks_for_date
                    .swap_remove(task_performed_index)
                    .task_performed;
                // Update the total time spent
                task_performed.time_spent += time_spent;
                TaskPerformed::update_task_performed(&task_performed, &mut connection)
                    .expect("todo")
            }
            None => {
                TaskPerformed::insert_task_performed(
                    &TaskPerformed {
                        date: self.date.to_string(),
                        task_id: task.id,
                        time_spent,
                    },
                    &mut connection,
                )
                .expect("todo") // TODO this should probably return the error
            }
        };

        self.tasks_for_date.push(TaskListItem {
            task_name: task_name.to_string(),
            task_performed,
        });

        self.tasks_for_date
            .sort_by_key(|task| Reverse(task.task_performed.time_spent));
    }

    pub fn delete_task_performed(&mut self, task_name: &str, date: &NaiveDate) {
        let mut connection = self.db_connection.lock().unwrap();

        let task = Task::get_task_by_name(task_name, &mut connection);

        let task = match task {
            Some(task) => task,
            None => return,
        };

        TaskPerformed::delete_task_performed(task.id, &date.to_string(), &mut connection).unwrap();

        // Update the tasks_for_date if the task did exist there
        match self
            .tasks_for_date
            .iter()
            .position(|task_list_item| task_list_item.task_performed.task_id == task.id)
        {
            Some(index) => {
                self.tasks_for_date.swap_remove(index);
            }
            None => (),
        }
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
            let task_item_with_same_name_index =
                self.tasks_for_date.iter().position(|task_list_item| {
                    task_list_item.task_name == task_name
                        && task_list_item.task_performed.task_id != task_id // TODO remove this line
                });

            match task_item_with_same_name_index {
                Some(index) => {
                    let task = self.tasks_for_date.swap_remove(index);
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

#[cfg(test)]
mod tests {
    use std::{
        fs,
        sync::{Arc, Mutex},
    };

    use crate::{
        schema::{task, task_performed},
        MIGRATIONS,
    };

    use super::*;
    use diesel::{Connection, RunQueryDsl};
    use diesel_migrations::MigrationHarness;
    use rstest::*;
    use serial_test::serial;

    const DATABASE_URL: &str = "test/task_list_test_database.db";

    #[fixture]
    #[once]
    pub fn db_connection() -> Arc<Mutex<SqliteConnection>> {
        fs::create_dir_all("test").unwrap();
        let db_connection = Arc::new(Mutex::new(
            SqliteConnection::establish(&DATABASE_URL)
                .unwrap_or_else(|_| panic!("Error connecting to {}", DATABASE_URL)),
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

        db_connection
    }

    #[rstest]
    #[serial]
    fn test_create_new_task(db_connection: &Arc<Mutex<SqliteConnection>>) {
        let task_name_1 = "test_task_1";
        let task_name_2 = "test_task_2";
        let time_spent = 60;
        let date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();

        {
            let mut task_list = TaskList::new(db_connection.clone(), date);
            task_list.add_task(task_name_1, time_spent);
            task_list.add_task(task_name_2, time_spent);

            assert_eq!(task_list.date, date);
            assert_eq!(
                task_list.tasks_for_date.get(0).unwrap().task_name,
                task_name_1
            );
            assert_eq!(
                task_list.tasks_for_date.get(1).unwrap().task_name,
                task_name_2
            );
        }

        let connection = db_connection.clone();
        let mut connection = connection.lock().unwrap();

        let inserted_task_1 = Task::get_task_by_name(task_name_1, &mut connection).unwrap();
        assert_eq!(inserted_task_1.name, task_name_1);
        let inserted_task_2 = Task::get_task_by_name(task_name_2, &mut connection).unwrap();
        assert_eq!(inserted_task_2.name, task_name_2);

        let tasks_performed =
            TaskPerformed::get_all_tasks_by_date(&date.to_string(), &mut connection);
        assert_eq!(tasks_performed.len(), 2);

        let task_performed_1 = tasks_performed.get(0).unwrap();
        assert_eq!(task_performed_1.date, date.to_string());
        assert_eq!(task_performed_1.time_spent, time_spent);
        assert_eq!(task_performed_1.task_id, inserted_task_1.id);

        let task_performed_2 = tasks_performed.get(1).unwrap();
        assert_eq!(task_performed_2.date, date.to_string());
        assert_eq!(task_performed_2.time_spent, time_spent);
        assert_eq!(task_performed_2.task_id, inserted_task_2.id);
    }

    #[rstest]
    #[serial]
    fn test_change_date(db_connection: &Arc<Mutex<SqliteConnection>>) {
        let task_1 = {
            let connection = db_connection.clone();
            let mut connection = connection.lock().unwrap();
            Task::create_task("test_change_date_task_1", &mut connection).unwrap()
        };

        let task_2 = {
            let connection = db_connection.clone();
            let mut connection = connection.lock().unwrap();
            Task::create_task("test_change_date_task_2", &mut connection).unwrap()
        };

        let date_1 = NaiveDate::from_ymd_opt(2023, 2, 1).unwrap();
        let date_2 = NaiveDate::from_ymd_opt(2023, 2, 2).unwrap();

        let insert_task_performed_1 = {
            let connection = db_connection.clone();
            let mut connection = connection.lock().unwrap();
            let task_performed_1 = TaskPerformed {
                date: date_1.to_string(),
                task_id: task_1.id,
                time_spent: 5,
            };

            TaskPerformed::insert_task_performed(&task_performed_1, &mut connection).unwrap()
        };

        let insert_task_performed_2 = {
            let connection = db_connection.clone();
            let mut connection = connection.lock().unwrap();
            let task_performed_2 = TaskPerformed {
                date: date_2.to_string(),
                task_id: task_2.id,
                time_spent: 10,
            };

            TaskPerformed::insert_task_performed(&task_performed_2, &mut connection).unwrap()
        };

        let mut task_list = TaskList::new(db_connection.clone(), date_1);
        assert_eq!(task_list.date, date_1);
        assert_eq!(task_list.tasks_for_date.len(), 1);
        assert_eq!(
            task_list.tasks_for_date.get(0).unwrap().task_performed,
            insert_task_performed_1
        );

        // Change Date
        task_list.change_date(date_2);
        assert_eq!(task_list.date, date_2);
        assert_eq!(task_list.tasks_for_date.len(), 1);
        assert_eq!(
            task_list.tasks_for_date.get(0).unwrap().task_performed,
            insert_task_performed_2
        );
    }

    #[rstest]
    #[serial]
    fn test_delete_task_performed(db_connection: &Arc<Mutex<SqliteConnection>>) {
        let connection = db_connection.clone();
        let task_name = "test_delete_task_performed_task";
        let time_spent = 60;

        let date = NaiveDate::from_ymd_opt(2023, 3, 1).unwrap();

        {
            let mut task_list = TaskList::new(db_connection.clone(), date);
            task_list.add_task(task_name, time_spent);

            task_list.delete_task_performed(task_name, &date);

            assert!(task_list.tasks_for_date.is_empty());
        }

        assert!(TaskPerformed::get_all_tasks_by_date(
            &date.to_string(),
            &mut connection.lock().unwrap()
        )
        .is_empty());
    }

    #[rstest]
    #[serial]
    fn test_update_task_performed(db_connection: &Arc<Mutex<SqliteConnection>>) {
        let task_name_1 = "test_update_task_performed_task_1";
        let time_spent_1 = 60;
        let task_name_2 = "test_update_task_performed_task_2";
        let time_spent_2 = 30;
        let updated_task_name = "test_update_task_performed_task_updated";
        let updated_task_time_spent = 90;
        let date = NaiveDate::from_ymd_opt(2023, 4, 1).unwrap();

        let task_1_id = {
            let connection = db_connection.clone();
            let mut connection = connection.lock().unwrap();
            Task::create_task(task_name_1, &mut connection).unwrap().id
        };

        let task_2_id = {
            let connection = db_connection.clone();
            let mut connection = connection.lock().unwrap();
            Task::create_task(task_name_2, &mut connection).unwrap().id
        };

        {
            let mut task_list = TaskList::new(db_connection.clone(), date);
            task_list.add_task(task_name_1, time_spent_1);
            task_list.add_task(task_name_2, time_spent_2);

            let tasks_for_date = task_list.list_all_tasks_performed();
            assert_eq!(tasks_for_date.len(), 2);

            task_list.update_task_performed(task_1_id, updated_task_name, updated_task_time_spent);
            assert_eq!(task_list.tasks_for_date.len(), 2);
            let updated_task = task_list
                .tasks_for_date
                .iter()
                .filter(|task_list_item| &task_list_item.task_name == updated_task_name)
                .next()
                .unwrap();
            assert_eq!(updated_task.task_name, updated_task_name);
            assert_eq!(
                updated_task.task_performed.time_spent,
                updated_task_time_spent
            );
            assert_ne!(updated_task.task_performed.task_id, task_1_id);

            assert_eq!(
                task_list.fetch_most_recent_task_names(3),
                vec![
                    updated_task_name.to_string(),
                    task_name_2.to_string(),
                    task_name_1.to_string()
                ]
            );
        }

        {
            let connection = db_connection.clone();
            let mut connection = connection.lock().unwrap();
            assert!(TaskPerformed::get_task_by_task_id_and_date(
                task_1_id,
                &date.to_string(),
                &mut connection,
            )
            .is_none());
            assert!(TaskPerformed::get_task_by_task_id_and_date(
                task_2_id,
                &date.to_string(),
                &mut connection,
            )
            .is_some());
            let updated_task = Task::get_task_by_name(&updated_task_name, &mut connection).unwrap();
            assert_eq!(updated_task.name, updated_task_name);

            let updated_task_performed = TaskPerformed::get_task_by_task_id_and_date(
                updated_task.id,
                &date.to_string(),
                &mut connection,
            )
            .unwrap();
            assert_eq!(updated_task_performed.time_spent, updated_task_time_spent);

            assert_eq!(
                TaskPerformed::get_all_tasks_by_date(&date.to_string(), &mut connection).len(),
                2
            )
        };
    }

    #[rstest]
    #[serial]
    fn test_fetch_most_recent_tasks(db_connection: &Arc<Mutex<SqliteConnection>>) {
        let task_name_1 = "test_fetch_most_recent_tasks_1";
        let task_name_2 = "test_fetch_most_recent_tasks_2";
        let task_name_3 = "test_fetch_most_recent_tasks_3";
        let task_name_4 = "test_fetch_most_recent_tasks_4";
        let time_spent = 5;

        let date = NaiveDate::from_ymd_opt(2023, 5, 1).unwrap();

        let mut task_list = TaskList::new(db_connection.clone(), date);

        task_list.add_task(task_name_1, time_spent);
        task_list.add_task(task_name_2, time_spent);
        task_list.add_task(task_name_3, time_spent);
        assert_eq!(
            task_list.fetch_most_recent_task_names(3),
            vec![
                task_name_3.to_string(),
                task_name_2.to_string(),
                task_name_1.to_string()
            ]
        );
        assert_eq!(
            task_list.fetch_most_recent_task_names(1),
            vec![task_name_3.to_string()]
        );

        task_list.add_task(task_name_4, time_spent);
        assert_eq!(
            task_list.fetch_most_recent_task_names(3),
            vec![
                task_name_4.to_string(),
                task_name_3.to_string(),
                task_name_2.to_string()
            ]
        );
    }
}
