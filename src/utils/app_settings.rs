use std::{fs::{self, File}, path::Path};

pub struct AppSettings {
    username: String,
    api_key: String,
}

impl AppSettings {
    pub fn new() -> AppSettings {
        let mut app_settings = AppSettings {
            username: String::from("[PLACEHOLDER]"),
            api_key: String::from("[PLACEHOLDER]"),
        };
        app_settings.load();
        app_settings
    }

    pub fn load(&mut self) {
        print!("Reading settings . . .\n");
        let json_file_path = Path::new("./user_settings.json");
        let mut settings_string = fs::read_to_string("user_settings.json").expect("Should have been able to read the file");
        print!("{}\n", settings_string);

    }
}
