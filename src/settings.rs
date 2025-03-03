use std::{fs, path::PathBuf, time::Duration};

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

pub const APPLICATION_NAME: &str = "Kronos";
pub const DATA_DIRECTORY_NAME: &str = "data";
pub const DATABASE_FILE_NAME: &str = "database.db";
pub const SETTINGS_DIRECTORY_NAME: &str = "settings";
pub const SETTINGS_FILE_NAME: &str = "settings.json";

pub const DEFAULT_TASK_PROMPT_DELAY_SECONDS: u64 = 15 * 60;

pub const APPLICATION_STORAGE_PATH: Lazy<PathBuf> = Lazy::new(|| {
    let mut application_storage_path =
        dirs::data_dir().expect("Could not find the user data directory");
    application_storage_path.push(APPLICATION_NAME);
    application_storage_path
});

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct Settings {
    task_prompt_delay: Duration,
    data_storage_path: PathBuf,
    database_file_path: PathBuf,
    user_settings_file_path: PathBuf,
}

impl Settings {
    pub fn from_dir(application_storage_path: PathBuf) -> Self {
        log::trace!(
            "Creating Settings object with application_storage_path '{}'.",
            application_storage_path
                .to_str()
                .unwrap_or("<unable to print path>")
        );
        // Set file locations
        let user_settings_directory = application_storage_path
            .clone()
            .join(SETTINGS_DIRECTORY_NAME);
        let user_settings_file_path = application_storage_path
            .clone()
            .join(SETTINGS_DIRECTORY_NAME)
            .join(SETTINGS_FILE_NAME);
        let data_storage_path = application_storage_path.clone().join(DATA_DIRECTORY_NAME);
        let database_file_path = application_storage_path
            .clone()
            .join(DATA_DIRECTORY_NAME)
            .join(DATABASE_FILE_NAME);
        // Create directories if needed
        if fs::metadata(&application_storage_path.as_path()).is_err() {
            log::info!(
                "APPLICATION_STORAGE_PATH directory does not exist. Creating directory '{}'.",
                application_storage_path
                    .to_str()
                    .unwrap_or("<unable to print path>")
            );
            std::fs::create_dir_all(application_storage_path.as_path()).expect(&format!(
                "Failed to create application save directory '{}'",
                APPLICATION_STORAGE_PATH
                    .to_str()
                    .unwrap_or("<unable to print path>")
            ));
        }
        if fs::metadata(&user_settings_directory.as_path()).is_err() {
            log::info!(
                "user_settings_directory directory does not exist. Creating directory '{}'.",
                user_settings_directory
                    .to_str()
                    .unwrap_or("<unable to print path>")
            );
            std::fs::create_dir(user_settings_directory.as_path()).expect(&format!(
                "Failed to create user settings save directory '{}'",
                user_settings_directory
                    .to_str()
                    .unwrap_or("<unable to print path>")
            ));
        }
        if fs::metadata(&data_storage_path.as_path()).is_err() {
            log::info!(
                "data_storage_path directory does not exist. Creating directory '{}'.",
                data_storage_path
                    .to_str()
                    .unwrap_or("<unable to print path>")
            );
            std::fs::create_dir(data_storage_path.as_path()).expect(&format!(
                "Failed to create user data save directory '{}'",
                APPLICATION_STORAGE_PATH
                    .to_str()
                    .unwrap_or("<unable to print path>")
            ));
        }

        if user_settings_file_path.as_path().exists() {
            log::debug!(
                "user_settings_file_path file exists. Loading settings from file '{}'.",
                user_settings_file_path
                    .to_str()
                    .unwrap_or("<unable to print path>")
            );
            if let Ok(settings_file_data) = fs::read_to_string(&user_settings_file_path.as_path()) {
                match serde_json::from_str::<Self>(&settings_file_data) {
                    Ok(settings) => {
                        return Settings {
                            data_storage_path,
                            database_file_path,
                            user_settings_file_path,
                            ..settings
                        };
                    }
                    Err(e) => {
                        log::error!("User settings file cannot be read. {}", e);
                    }
                };
            };
        }

        log::info!(
            "Failed to load existing settings file. Creating new settings with default values."
        );
        Settings {
            task_prompt_delay: Duration::from_secs(DEFAULT_TASK_PROMPT_DELAY_SECONDS),
            data_storage_path,
            database_file_path,
            user_settings_file_path,
        }
    }

    pub fn new() -> Self {
        Settings::from_dir(APPLICATION_STORAGE_PATH.clone())
    }

    pub fn get_task_prompt_delay(&self) -> Duration {
        self.task_prompt_delay
    }

    pub fn update_task_prompt_delay(&mut self, task_prompt_delay: Duration) {
        self.task_prompt_delay = task_prompt_delay;
        self.save_settings_to_file();
    }

    fn save_settings_to_file(&self) {
        fs::write(
            self.user_settings_file_path.as_path(),
            serde_json::to_string(&self).expect("Failed to serialize"),
        )
        .expect(&format!(
            "Failed to save file: \"{}\"",
            self.user_settings_file_path
                .to_str()
                .unwrap_or("<unable to print path>")
        ));
    }

    pub fn get_data_storage_path(&self) -> &PathBuf {
        &self.data_storage_path
    }

    pub fn get_database_file_path(&self) -> &PathBuf {
        &self.database_file_path
    }

    pub fn get_user_settings_file_path(&self) -> &PathBuf {
        &self.user_settings_file_path
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_create_settings() {
        let temp_dir = TempDir::new().unwrap();

        let settings = Settings::from_dir(temp_dir.path().to_path_buf());

        assert_eq!(
            settings.get_task_prompt_delay(),
            Duration::from_secs(DEFAULT_TASK_PROMPT_DELAY_SECONDS)
        );
        assert!(temp_dir.path().join(DATA_DIRECTORY_NAME).exists());
        assert!(temp_dir.path().join(SETTINGS_DIRECTORY_NAME).exists());
    }

    #[test]
    fn test_update_settings_and_save() {
        let temp_dir = TempDir::new().unwrap();

        let new_task_delay = Duration::from_secs(5);

        {
            let mut settings = Settings::from_dir(temp_dir.path().to_path_buf());

            assert_eq!(
                settings.get_task_prompt_delay(),
                Duration::from_secs(DEFAULT_TASK_PROMPT_DELAY_SECONDS)
            );

            settings.update_task_prompt_delay(new_task_delay);

            assert_eq!(settings.task_prompt_delay, new_task_delay);
        }

        let settings = Settings::from_dir(temp_dir.path().to_path_buf());
        assert_eq!(settings.task_prompt_delay, new_task_delay);
    }

    #[test]
    fn test_read_settings_from_file() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::create_dir_all(
            temp_dir
                .path()
                .join(APPLICATION_NAME)
                .join(SETTINGS_DIRECTORY_NAME),
        )
        .unwrap();

        let path_to_test_resource = PathBuf::from("test")
            .join("res")
            .join("test_read_settings.json");
        let path_to_settings_file_location = temp_dir
            .path()
            .join(APPLICATION_NAME)
            .join(SETTINGS_DIRECTORY_NAME)
            .join(SETTINGS_FILE_NAME);
        fs::copy(
            &path_to_test_resource,
            &path_to_settings_file_location,
        )
        .expect(&format!("Failed to copy file '{:?}' to '{:?}'.\nResult of checking for file: {:?}\nResult of checking for dir: {:?}", 
            &path_to_test_resource, 
            &path_to_settings_file_location, 
            fs::metadata(&path_to_test_resource), 
            fs::metadata(&path_to_settings_file_location)));

        let settings = Settings::from_dir(temp_dir.path().join(APPLICATION_NAME));

        assert_eq!(settings.get_task_prompt_delay(), Duration::from_secs(1));
    }
}
