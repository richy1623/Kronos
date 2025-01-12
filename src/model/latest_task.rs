use std::fs;

use chrono::Local;
use serde::{Deserialize, Serialize};

const LATEST_TASK_FILE_LOCATION: &str = "./data/latest_task.json";

#[derive(Deserialize, Serialize, Debug)]
pub struct LatestTask {
    pub task_id: Option<i32>,
    pub date_time_performed: chrono::DateTime<Local>,
}

impl LatestTask {
    pub fn get_latest_task_performed() -> Self {
        {
            if fs::metadata(LATEST_TASK_FILE_LOCATION).is_err() {
                return LatestTask {
                    task_id: None,
                    date_time_performed: Local::now(),
                };
            }
            let data = fs::read_to_string(LATEST_TASK_FILE_LOCATION).expect(&format!(
                "Failed to read file: \"{}\"",
                LATEST_TASK_FILE_LOCATION
            ));
            serde_json::from_str(&data).unwrap()
        }
    }

    pub fn update_latest_task_performed(task_id: i32) -> Self {
        // TODO: Handle directory not exists
        let latest_task = LatestTask {
            task_id: Some(task_id),
            date_time_performed: Local::now(),
        };
        fs::write(
            LATEST_TASK_FILE_LOCATION,
            serde_json::to_string(&latest_task).expect("Failed to serialize"),
        )
        .expect(&format!(
            "Failed to save file: \"{}\"",
            LATEST_TASK_FILE_LOCATION
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
        if fs::metadata(LATEST_TASK_FILE_LOCATION).is_ok() {
            fs::remove_file(LATEST_TASK_FILE_LOCATION).expect("Failed to remove test file");
        }
        // Test: Read no task
        let task = LatestTask::get_latest_task_performed();

        // Assert: Verify the task data
        assert_eq!(task.task_id, None);
        assert_date_time_close(&task.date_time_performed, &Local::now());

        // Test: Create a new task
        LatestTask::update_latest_task_performed(1);

        // Assert: Verify the file was created and contains the correct data
        let data = fs::read_to_string(LATEST_TASK_FILE_LOCATION).expect("Failed to read test file");
        let task: LatestTask = serde_json::from_str(&data).expect("Failed to parse JSON");
        assert_eq!(task.task_id.unwrap(), 1);
        assert_date_time_close(&task.date_time_performed, &Local::now());

        // Test: Update a task
        LatestTask::update_latest_task_performed(2);

        // Assert: Verify the file was updated with new data
        let data = fs::read_to_string(LATEST_TASK_FILE_LOCATION).expect("Failed to read test file");
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
