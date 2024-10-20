use std::{fs::File, path::Path};
use serde::{Deserialize, Serialize};

#[derive(serde::Deserialize)]
pub struct AppSettings {
    setting_1: Option<String>,
    setting_2: Option<String>,
}

impl AppSettings {
    pub fn new() -> AppSettings {
        let mut app_settings = AppSettings {
            setting_1: None,
            setting_2: None,
        };
        app_settings.load();
        app_settings
    }

    fn load(&mut self) {
        let json_file_path = Path::new("default_settings.json");
        let settings_file = File::open(json_file_path);
        let users:Vec<AppSettings> = serde_json::from_reader(settings_file).expect("error while reading or parsing");
    }
}
