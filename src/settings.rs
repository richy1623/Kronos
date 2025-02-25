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

pub const USER_DATA_STORAGE_PATH: Lazy<PathBuf> = Lazy::new(|| {
    let mut user_data_storage_path = APPLICATION_STORAGE_PATH.clone();
    user_data_storage_path.push(DATA_DIRECTORY_NAME);
    user_data_storage_path
});
pub const USER_DATA_DATABASE_FILE_PATH: Lazy<PathBuf> = Lazy::new(|| {
    let mut user_data_file_path = USER_DATA_STORAGE_PATH.clone();
    user_data_file_path.push(DATABASE_FILE_NAME);
    user_data_file_path
});

pub const USER_SETTINGS_STORAGE_PATH: Lazy<PathBuf> = Lazy::new(|| {
    let mut user_settings_storage_path = APPLICATION_STORAGE_PATH.clone();
    user_settings_storage_path.push(SETTINGS_DIRECTORY_NAME);
    user_settings_storage_path
});

pub const USER_SETTINGS_FILE_PATH: Lazy<PathBuf> = Lazy::new(|| {
    let mut user_settings_file_path = USER_SETTINGS_STORAGE_PATH.clone();
    user_settings_file_path.push(SETTINGS_FILE_NAME);
    user_settings_file_path
});

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct Settings {
    task_prompt_delay: Duration,
}

impl Settings {
    pub fn new() -> Self {
        if fs::metadata(&APPLICATION_STORAGE_PATH.as_path()).is_err() {
            log::info!(
                "APPLICATION_STORAGE_PATH directory does not exist. Creating directory '{}'.",
                APPLICATION_STORAGE_PATH
                    .to_str()
                    .unwrap_or("<unable to print path>")
            );
            std::fs::create_dir(APPLICATION_STORAGE_PATH.as_path()).expect(&format!(
                "Failed to create application save directory '{}'",
                APPLICATION_STORAGE_PATH
                    .to_str()
                    .unwrap_or("<unable to print path>")
            ));
        }
        if fs::metadata(&USER_SETTINGS_STORAGE_PATH.as_path()).is_err() {
            log::info!(
                "USER_SETTINGS_STORAGE_PATH directory does not exist. Creating directory '{}'.",
                USER_SETTINGS_STORAGE_PATH
                    .to_str()
                    .unwrap_or("<unable to print path>")
            );
            std::fs::create_dir(USER_SETTINGS_STORAGE_PATH.as_path()).expect(&format!(
                "Failed to create user settings save directory '{}'",
                APPLICATION_STORAGE_PATH
                    .to_str()
                    .unwrap_or("<unable to print path>")
            ));
        }
        if fs::metadata(&USER_DATA_STORAGE_PATH.as_path()).is_err() {
            log::info!(
                "USER_DATA_STORAGE_PATH directory does not exist. Creating directory '{}'.",
                USER_DATA_STORAGE_PATH
                    .to_str()
                    .unwrap_or("<unable to print path>")
            );
            std::fs::create_dir(USER_DATA_STORAGE_PATH.as_path()).expect(&format!(
                "Failed to create user data save directory '{}'",
                APPLICATION_STORAGE_PATH
                    .to_str()
                    .unwrap_or("<unable to print path>")
            ));
        }

        if USER_SETTINGS_FILE_PATH.as_path().exists() {
            log::debug!(
                "USER_SETTINGS_FILE_PATH file exists. Loading settings from file '{}'.",
                USER_SETTINGS_FILE_PATH
                    .to_str()
                    .unwrap_or("<unable to print path>")
            );
            if let Ok(settings_file_data) = fs::read_to_string(&USER_SETTINGS_FILE_PATH.as_path()) {
                match serde_json::from_str::<Self>(&settings_file_data) {
                    Ok(settings) => {
                        return settings;
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
            // task_prompt_delay_seconds: DEFAULT_TASK_PROMPT_DELAY_SECONDS,
        }
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
            USER_SETTINGS_FILE_PATH.as_path(),
            serde_json::to_string(&self).expect("Failed to serialize"),
        )
        .expect(&format!(
            "Failed to save file: \"{}\"",
            USER_SETTINGS_FILE_PATH
                .to_str()
                .unwrap_or("<unable to print path>")
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn testsettings_create_settings() {
        env_logger::init();
        let settings = Settings::new();
        assert_eq!(
            settings.get_task_prompt_delay(),
            Duration::from_secs(DEFAULT_TASK_PROMPT_DELAY_SECONDS)
        );
    }

    #[test]
    fn testsettings_update_task_prompt_delay() {
        let mut settings = Settings::new();
        let new_delay = Duration::from_secs(30 * 60); // 30 minutes

        settings.update_task_prompt_delay(new_delay);

        assert_eq!(settings.get_task_prompt_delay(), new_delay);
    }

    #[test]
    fn testsettings_read_settings_from_file() {
        env_logger::init();
        let settings = Settings::new();
        assert_eq!(settings.get_task_prompt_delay(), Duration::from_secs(1800));
        // 30 minutes
    }
}
