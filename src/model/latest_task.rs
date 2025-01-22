use std::{fs, path::Path};

use chrono::Local;
use serde::{Deserialize, Serialize};

use crate::DATA_STORAGE_PATH;

pub const LATEST_TASK_FILE_NAME: &str = "latest_task.json";

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct LatestTask {
    pub task_id: Option<i32>,
    pub date_time_performed: chrono::DateTime<Local>,
}

impl LatestTask {
    fn get_latest_task_file_path() -> String {
        Path::new(DATA_STORAGE_PATH)
            .join(LATEST_TASK_FILE_NAME)
            .as_path()
            .to_str()
            .unwrap()
            .to_string()
    }
    pub fn get_latest_task_performed() -> Self {
        {
            if fs::metadata(LatestTask::get_latest_task_file_path()).is_err() {
                return LatestTask {
                    task_id: None,
                    date_time_performed: Local::now(),
                };
            }
            let data =
                fs::read_to_string(LatestTask::get_latest_task_file_path()).expect(&format!(
                    "Failed to read file: \"{}\"",
                    LatestTask::get_latest_task_file_path()
                ));
            serde_json::from_str(&data).unwrap()
        }
    }

    pub fn update_latest_task_performed(task_id: Option<i32>) -> Self {
        if fs::metadata(crate::DATA_STORAGE_PATH).is_err() {
            fs::create_dir_all(crate::DATA_STORAGE_PATH).unwrap();
        }

        let latest_task = LatestTask {
            task_id,
            date_time_performed: Local::now(),
        };
        // TODO return Result
        fs::write(
            LatestTask::get_latest_task_file_path(),
            serde_json::to_string(&latest_task).expect("Failed to serialize"),
        )
        .expect(&format!(
            "Failed to save file: \"{}\"",
            LatestTask::get_latest_task_file_path()
        ));
        latest_task
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;
    use std::fs;

    /// Helper function to parse the `date_time_performed` and allow time comparison with tolerance
    fn assert_date_time_close(
        expected: &chrono::DateTime<Local>,
        actual: &chrono::DateTime<Local>,
    ) {
        let difference = (*expected - *actual).num_seconds().abs();
        assert!(
            difference <= 5,
            "Timestamps are too far apart: expected = {}, actual = {}, difference = {} seconds",
            expected,
            actual,
            difference
        );
    }

    #[test]
    fn test_task_latest() {
        // Arrange: Ensure no test file exists
        if fs::metadata(LatestTask::get_latest_task_file_path()).is_ok() {
            fs::remove_file(LatestTask::get_latest_task_file_path())
                .expect("Failed to remove test file");
        }
        // Test: Read no task
        let task = LatestTask::get_latest_task_performed();

        // Assert: Verify the task data
        assert_eq!(task.task_id, None);
        assert_date_time_close(&task.date_time_performed, &Local::now());

        // Test: Create a new task
        LatestTask::update_latest_task_performed(Some(1));

        // Assert: Verify the file was created and contains the correct data
        let data = fs::read_to_string(LatestTask::get_latest_task_file_path())
            .expect("Failed to read test file");
        let task: LatestTask = serde_json::from_str(&data).expect("Failed to parse JSON");
        assert_eq!(task.task_id.unwrap(), 1);
        assert_date_time_close(&task.date_time_performed, &Local::now());

        // Test: Update a task
        LatestTask::update_latest_task_performed(Some(2));

        // Assert: Verify the file was updated with new data
        let data = fs::read_to_string(LatestTask::get_latest_task_file_path())
            .expect("Failed to read test file");
        let task: LatestTask = serde_json::from_str(&data).expect("Failed to parse JSON");
        assert_eq!(task.task_id.unwrap(), 2);
        assert_date_time_close(&task.date_time_performed, &Local::now());

        // Test: Read the task
        let task = LatestTask::get_latest_task_performed();

        // Assert: Verify the task data
        assert_eq!(task.task_id.unwrap(), 2);
        assert_date_time_close(&task.date_time_performed, &Local::now());
    }
}
