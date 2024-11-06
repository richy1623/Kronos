use std::{fs::{self, File}, path::Path};
use serde::{Deserialize, Serialize};
use std::thread;
use notify::{Watcher, RecommendedWatcher, RecursiveMode, Result};

#[derive(Deserialize, Serialize, Debug)]
struct Settings_File{
    username: String,
    api_key: String,
    setting_3: u32,
    setting_4: f64
}

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
        app_settings.start_watcher();
        app_settings
    }

    pub fn load(&mut self) {
        print!("Reading settings . . .\n");
        let mut settings: Settings_File = {
            let data = fs::read_to_string("./user_settings/user_settings.json").expect("LogRocket: error reading file");
            serde_json::from_str(&data).unwrap()
        };

        self.username = settings.username;
        self.api_key = settings.api_key;
    }

    fn start_watcher(&mut self){
        // thread::spawn(|| {
        //     for i in 1..10 {

        //         let mut watcher = notify::recommended_watcher(|res| {
        //             match res {
        //                Ok(event) => println!("event: {:?}", event),
        //                Err(e) => println!("watch error: {:?}", e),
        //             }
        //         }).expect("Error setting up a watcher");

                
        //         watcher.watch(Path::new("./user_settings/user_settings.json"), RecursiveMode::Recursive).expect("Ran into error while watching");
        //         print!("Files changed {}\n",i);
        //     }
        // }).join();
        print!("\nThreading to be done\n");

    }

    // Add these getter methods to allow access to the private fields
    // Thanks Chatgpt
    pub fn get_username(&self) -> &str {
        &self.username
    }

    pub fn get_api_key(&self) -> &str {
        &self.api_key
    }
}
