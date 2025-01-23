use std::{
    fs,
    path::{Path, PathBuf},
};

use chrono::Local;
use serde::{Deserialize, Serialize};

use crate::DATA_STORAGE_PATH;

pub const LATEST_TASK_FILE_NAME: &str = "latest_task.json";

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct LatestTask {
    pub task_id: Option<i32>,
    pub date_time_performed: chrono::DateTime<Local>,
}

pub struct LatestTaskManager {
    last_task_file_location: PathBuf,
}

impl LatestTaskManager {
    pub fn new() -> Self {
        LatestTaskManager {
            last_task_file_location: Path::new(DATA_STORAGE_PATH).to_path_buf(),
        }
    }

    pub fn from_file_location(last_task_file_location: PathBuf) -> Self {
        if fs::metadata(&last_task_file_location).is_err() {
            fs::create_dir_all(&last_task_file_location).unwrap();
        }
        LatestTaskManager {
            last_task_file_location,
        }
    }

    fn get_latest_task_file_path(&self) -> PathBuf {
        Path::new(&self.last_task_file_location).join(LATEST_TASK_FILE_NAME)
    }

    pub fn get_latest_task_file_location(&self) -> PathBuf {
        self.last_task_file_location.clone()
    }

    pub fn update_latest_task_file_location(&mut self, new_last_task_performed_file_path: PathBuf) {
        let old_file_path = self.get_latest_task_file_path();
        let latest_task_performed = self.get_latest_task_performed();
        self.last_task_file_location = new_last_task_performed_file_path;
        if fs::metadata(&self.last_task_file_location).is_err() {
            fs::create_dir_all(&self.last_task_file_location).unwrap();
        }
        fs::write(
            self.get_latest_task_file_path(),
            serde_json::to_string(&latest_task_performed).expect("Failed to serialize"),
        )
        .expect(&format!(
            "Failed to save file: \"{}\"",
            self.get_latest_task_file_path()
                .to_str()
                .unwrap_or("<unable to print path>")
        ));

        if fs::metadata(&old_file_path).is_ok() {
            if fs::remove_file(&old_file_path).is_err() {
                log::error!(
                    "Failed to delete file: '{}'",
                    old_file_path.to_str().unwrap_or("<unable to print path>")
                )
            }
        }
    }

    pub fn get_latest_task_performed(&self) -> LatestTask {
        {
            let data = match fs::read_to_string(self.get_latest_task_file_path()) {
                Ok(data) => data,
                Err(_) => {
                    return LatestTask {
                        task_id: None,
                        date_time_performed: Local::now(),
                    }
                }
            };
            serde_json::from_str(&data).unwrap()
        }
    }

    pub fn update_latest_task_performed(&mut self, task_id: Option<i32>) -> LatestTask {
        let latest_task = LatestTask {
            task_id,
            date_time_performed: Local::now(),
        };
        // TODO return Result
        fs::write(
            self.get_latest_task_file_path(),
            serde_json::to_string(&latest_task).expect("Failed to serialize"),
        )
        .expect(&format!(
            "Failed to save file: \"{}\"",
            self.get_latest_task_file_path()
                .to_str()
                .unwrap_or("<unable to print path>")
        ));
        latest_task
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Local};
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
    fn test_latest_task_manager_new() {
        let latest_task_manager = LatestTaskManager::new();
        assert_eq!(
            latest_task_manager
                .last_task_file_location
                .to_str()
                .unwrap(),
            DATA_STORAGE_PATH
        );
        assert_eq!(
            latest_task_manager
                .get_latest_task_file_location()
                .to_str()
                .unwrap(),
            DATA_STORAGE_PATH
        );
        assert_eq!(
            latest_task_manager.get_latest_task_file_path(),
            Path::new("data").join(LATEST_TASK_FILE_NAME)
        );
    }

    #[test]
    fn test_latest_task_manager_from_file_location() {
        let test_file_path = Path::new("test").join("latest_task_from_location");
        // Delete the path if it exists
        if fs::metadata(test_file_path.clone()).is_ok() {
            if fs::metadata(test_file_path.join(LATEST_TASK_FILE_NAME)).is_ok() {
                fs::remove_file(test_file_path.join(LATEST_TASK_FILE_NAME)).unwrap();
            }
            fs::remove_dir(test_file_path.clone()).unwrap();
        }
        // Create the latest_task_manager
        let latest_task_manager = LatestTaskManager::from_file_location(test_file_path.clone());
        assert_eq!(latest_task_manager.last_task_file_location, test_file_path);
        assert_eq!(
            latest_task_manager.get_latest_task_file_location(),
            test_file_path
        );
        assert!(fs::metadata(test_file_path.clone()).is_ok());
        assert_eq!(
            latest_task_manager.get_latest_task_file_path(),
            test_file_path.join(LATEST_TASK_FILE_NAME)
        );
    }

    #[test]
    fn test_get_latest_task_performed_no_such_task() {
        let test_file_path = Path::new("test").join("latest_task_performed_no_such_task");
        // Delete the path if it exists
        if fs::metadata(&test_file_path).is_ok() {
            if fs::metadata(test_file_path.join(LATEST_TASK_FILE_NAME)).is_ok() {
                fs::remove_file(test_file_path.join(LATEST_TASK_FILE_NAME)).unwrap();
            }
            fs::remove_dir(test_file_path.clone()).unwrap();
        }
        // Create the latest_task_manager
        let latest_task_manager = LatestTaskManager::from_file_location(test_file_path.clone());

        let latest_task = latest_task_manager.get_latest_task_performed();
        assert_eq!(latest_task.task_id, None);
        assert_date_time_close(&latest_task.date_time_performed, &Local::now());
    }

    #[test]
    fn test_get_latest_task_performed_with_task() {
        let test_file_path = Path::new("test").join("latest_task_performed_with_task");

        // Create the latest_task_manager
        let latest_task_manager = LatestTaskManager::from_file_location(test_file_path.clone());

        // Set the task
        let latest_task = LatestTask {
            task_id: Some(1),
            date_time_performed: DateTime::parse_from_rfc3339("2000-08-14T00:00:00+02:00")
                .unwrap()
                .into(),
        };
        fs::write(
            &latest_task_manager.get_latest_task_file_path(),
            serde_json::to_string(&latest_task).expect("Failed to serialize"),
        )
        .expect(&format!(
            "Failed to save file: \"{}\"",
            latest_task_manager
                .get_latest_task_file_path()
                .to_str()
                .unwrap_or("<unable to print path>")
        ));

        // Verify the task created
        let latest_task_found = latest_task_manager.get_latest_task_performed();
        assert_eq!(latest_task_found, latest_task);
    }

    #[test]
    fn test_update_latest_task_performed() {
        let test_file_path = Path::new("test").join("update_latest_task");

        // Create the latest_task_manager
        let mut latest_task_manager = LatestTaskManager::from_file_location(test_file_path.clone());

        // Set the task
        let latest_task = LatestTask {
            task_id: Some(1),
            date_time_performed: DateTime::parse_from_rfc3339("1999-11-05T00:00:00+02:00")
                .unwrap()
                .into(),
        };
        fs::write(
            &latest_task_manager.get_latest_task_file_path(),
            serde_json::to_string(&latest_task).expect("Failed to serialize"),
        )
        .expect(&format!(
            "Failed to save file: \"{}\"",
            latest_task_manager
                .get_latest_task_file_path()
                .to_str()
                .unwrap_or("<unable to print path>")
        ));

        // Update the task
        let latest_task_found = latest_task_manager.update_latest_task_performed(Some(7));

        // Verify the task created
        assert_eq!(latest_task_found.task_id.unwrap(), 7);
        assert_date_time_close(&latest_task_found.date_time_performed, &Local::now());
    }

    #[test]
    fn test_change_latest_task_dir() {
        let test_file_path_before = Path::new("test").join("change_dir_before");

        let test_file_path_after = Path::new("test").join("change_dir_after");
        // Delete the path if it exists
        if fs::metadata(&test_file_path_after).is_ok() {
            if fs::metadata(test_file_path_after.join(LATEST_TASK_FILE_NAME)).is_ok() {
                fs::remove_file(test_file_path_after.join(LATEST_TASK_FILE_NAME)).unwrap();
            }
            fs::remove_dir(test_file_path_after.clone()).unwrap();
        }

        // Create the latest_task_manager
        let mut latest_task_manager =
            LatestTaskManager::from_file_location(test_file_path_before.clone());

        let latest_task = latest_task_manager.update_latest_task_performed(Some(1));

        // Update the file location
        latest_task_manager.update_latest_task_file_location(test_file_path_after.clone());

        // Verify
        assert_eq!(
            &latest_task_manager.get_latest_task_file_location(),
            &test_file_path_after
        );
        assert_eq!(
            &latest_task_manager.get_latest_task_file_path(),
            &test_file_path_after.join(LATEST_TASK_FILE_NAME)
        );
        assert_eq!(latest_task_manager.get_latest_task_performed(), latest_task);

        assert!(fs::metadata(test_file_path_before.join(LATEST_TASK_FILE_NAME)).is_err());
    }
}
