use std::sync::{Arc, Mutex};

use diesel::SqliteConnection;
use log::{error, warn};
use reqwest::{
    Method, StatusCode,
    blocking::{Client, Request, RequestBuilder},
};

use crate::{model::task::Task, settings::Settings};

#[derive(PartialEq, Debug)]
pub enum SyncStatus {
    Synced,
    OutOfSync,
    Disconnected,
    Disabled,
}
pub struct SyncManager {
    settings: Arc<Mutex<Settings>>,
    sync_status: SyncStatus,
    db_connection: Arc<Mutex<SqliteConnection>>,
}

impl SyncManager {
    pub fn new(
        settings: Arc<Mutex<Settings>>,
        db_connection: Arc<Mutex<SqliteConnection>>,
    ) -> Self {
        let sync_status = if settings.lock().unwrap().get_sync_server_url().is_some() {
            SyncStatus::OutOfSync
        } else {
            SyncStatus::Disabled
        };
        SyncManager {
            settings,
            sync_status,
            db_connection,
        }
    }

    pub fn start(&mut self) {}

    pub fn get_sync_status(&self) -> &SyncStatus {
        &self.sync_status
    }

    pub fn sync_to_server(&mut self) {
        let sync_server_url = {
            let settings = self.settings.lock().unwrap();
            match settings.get_sync_server_url() {
                Some(sync_server_url) => sync_server_url.clone(),
                None => {
                    self.sync_status = SyncStatus::Disabled;
                    return;
                }
            }
        };

        self.sync_status = SyncStatus::OutOfSync;
        let mut new_sync_status = SyncStatus::Synced;

        let mut connection = match self.db_connection.lock() {
            Ok(connection) => connection,
            Err(error) => {
                warn!(
                    "Failed to sync_to_server due to failure to obtain the DB connection. {}",
                    error
                );
                return;
            }
        };

        let client = Client::new();

        // Sync Tasks
        let unsynced_tasks = Task::get_all_unsynced_tasks(&mut connection);

        for unsynced_task in unsynced_tasks {
            let request = Request::new(Method::POST, sync_server_url.clone());
            let request = RequestBuilder::from_parts(client.clone(), request)
                .body(serde_json::to_string(&unsynced_task).expect("Failed to serialize"))
                .build()
                .unwrap();
            match client.execute(request) {
                Ok(response) => match response.status() {
                    StatusCode::ACCEPTED => {
                        if let Err(e) = Task::update_task_is_synced_to_server(
                            unsynced_task.id,
                            true,
                            &mut connection,
                        ) {
                            new_sync_status = SyncStatus::OutOfSync;
                            error!(
                                "Failed to update sync state in local DB for task '{:?}' which was synced to server. {}",
                                unsynced_task, e
                            );
                        }
                    }
                    status_code => {
                        warn!(
                            "Failed to sync_to_server. Expected response code 'ACCEPTED', received '{}'",
                            status_code
                        );
                        new_sync_status = SyncStatus::OutOfSync;
                    }
                },
                Err(_) => {
                    self.sync_status = SyncStatus::Disconnected;
                    return;
                }
            }
        }

        // Sync Tasks Performed
        // TODO

        // Update Sync Status
        self.sync_status = new_sync_status;
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::MIGRATIONS;

    use super::*;
    use diesel::Connection;
    use diesel_migrations::MigrationHarness;
    use httpmock::MockServer;
    use rstest::{fixture, rstest};
    use tempfile::TempDir;

    #[fixture]
    pub fn test_parameters() -> (Arc<Mutex<Settings>>, Arc<Mutex<SqliteConnection>>, TempDir) {
        let temp_dir = TempDir::new().unwrap();

        let path_to_db = temp_dir.path().join("test_sync_manager.db");
        let path_to_db = path_to_db.to_str().unwrap();

        let db_connection = Arc::new(Mutex::new(
            SqliteConnection::establish(path_to_db)
                .unwrap_or_else(|_| panic!("Error connecting to {}", path_to_db)),
        ));
        db_connection
            .lock()
            .unwrap()
            .run_pending_migrations(MIGRATIONS)
            .unwrap();
        (
            Arc::new(Mutex::new(Settings::from_dir(
                temp_dir.path().to_path_buf(),
            ))),
            db_connection,
            temp_dir,
        )
    }

    #[rstest]
    pub fn test_sync_manager_tasks(
        test_parameters: (Arc<Mutex<Settings>>, Arc<Mutex<SqliteConnection>>, TempDir),
    ) {
        let (settings, db_connection, _temp_dir) = test_parameters;

        let server = MockServer::start();
        {
            settings
                .clone()
                .lock()
                .unwrap()
                .update_sync_server_url(Some(server.base_url().parse().unwrap()));
        }

        let expectation = server.mock(|when, then| {
            when.path("/");
            then.status(202);
        });

        // Create new sync manager
        let mut sync_manager = SyncManager::new(settings.clone(), db_connection.clone());
        assert_eq!(sync_manager.get_sync_status(), &SyncStatus::OutOfSync);

        // Sync Nothing to server
        sync_manager.sync_to_server();
        assert_eq!(sync_manager.get_sync_status(), &SyncStatus::Synced);

        // Create Tasks
        let task_1 = {
            let mut connection = db_connection.lock().unwrap();

            Task::create_task("task 1", &mut connection).unwrap()
        };
        let task_2 = {
            let mut connection = db_connection.lock().unwrap();

            Task::create_task("task 2", &mut connection).unwrap()
        };

        // Sync tasks to server
        sync_manager.sync_to_server();

        assert_eq!(sync_manager.get_sync_status(), &SyncStatus::Synced);
        expectation.assert_hits(2);
        {
            let mut connection = db_connection.lock().unwrap();

            assert!(
                Task::get_task_by_id(task_1.id, &mut connection)
                    .unwrap()
                    .is_synced_to_server
            );
            assert!(
                Task::get_task_by_id(task_2.id, &mut connection)
                    .unwrap()
                    .is_synced_to_server
            );
        }
    }
}
