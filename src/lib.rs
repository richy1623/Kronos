pub mod model;
pub mod schema;
pub mod settings;
pub mod task_list;
pub mod task_prompt;
pub mod task_prompt_manager;
pub mod widget;

use diesel::prelude::*;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};
use dotenvy::dotenv;
use std::env;

pub const DATA_STORAGE_PATH: &str = "data";

pub const MIGRATIONS: EmbeddedMigrations = diesel_migrations::embed_migrations!();

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let mut connection = SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    connection.run_pending_migrations(MIGRATIONS).unwrap();

    connection
}
