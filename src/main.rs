use std::sync::{Arc, Mutex, RwLock};

use chrono::Local;
use diesel::SqliteConnection;
use kronos::{
    model::latest_task::LatestTaskManager,
    settings::Settings,
    task_list::TaskList,
    task_prompt::TaskPrompt,
    task_prompt_manager::{self, TaskPromptManager, TaskPromptManagerState},
    widget::{task_list_widget::TaskListWidget, task_prompt_widget::TaskPromptWidget},
};

#[tokio::main]
async fn main() {
    env_logger::init();

    let settings = Arc::new(Mutex::new(Settings::new()));

    let database_connection = {
        Arc::new(Mutex::new(kronos::establish_connection(
            settings
                .lock()
                .unwrap()
                .get_database_file_path()
                .to_str()
                .expect("Failed to read database file path"),
        )))
    };

    let mut task_prompt_manager = TaskPromptManager::new(settings.clone());

    task_prompt_manager.start();
    let con = database_connection.clone();
    let con2 = database_connection.clone();

    let mut task_prompt_manager_rx = task_prompt_manager.subscribe();
    let settings_handle = settings.clone();
    while let Ok(cmd) = task_prompt_manager_rx.recv().await {
        match cmd {
            task_prompt_manager::TaskPromptManagerState::UiOpen => (),
            task_prompt_manager::TaskPromptManagerState::PendingPrompt => (),
            task_prompt_manager::TaskPromptManagerState::AwaitingPrompt => {
                task_prompt_manager
                    .tx
                    .send(TaskPromptManagerState::UiOpen)
                    .await
                    .unwrap();
                spawn_task_prompt(con.clone(), settings_handle.clone());
                // spawn_fake_window();
                spawn_task_list(false, &mut task_prompt_manager, con2.clone());
            }
            task_prompt_manager::TaskPromptManagerState::Stopped => break,
        }
    }
}

fn spawn_task_list(
    spawn_active: bool,
    task_prompt_manager: &TaskPromptManager,
    database_connection: Arc<Mutex<SqliteConnection>>,
) {
    let mut task_prompt_manager_rx = task_prompt_manager.subscribe();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 600.0])
            .with_visible(spawn_active)
            .with_active(spawn_active),
        ..Default::default()
    };

    let mut task_list_widget = TaskListWidget::new(TaskList::new(
        database_connection,
        Local::now().date_naive(),
    ));

    let mut open_task_prompt = false;
    let mut was_minimized = false;

    let tx = task_prompt_manager.tx.clone();

    let mut minimize_on_spawn = !spawn_active;

    eframe::run_simple_native("Task List", native_options, move |ctx, _frame| {
        if minimize_on_spawn {
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
            minimize_on_spawn = false;
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(&mut task_list_widget);

            let is_minimized = ui.ctx().input(|i| i.viewport().minimized).unwrap();
            if is_minimized {
                // Perform updates while minimized
                // Force repaint to keep updates flowing
                ui.ctx().request_repaint_after_secs(1.0);
            }
            if is_minimized != was_minimized {
                let tx = tx.clone();
                let tokio_runtime = tokio::runtime::Builder::new_current_thread()
                    .build()
                    .unwrap();

                let state = if is_minimized {
                    TaskPromptManagerState::PendingPrompt
                } else {
                    TaskPromptManagerState::UiOpen
                };

                tokio_runtime.spawn_blocking(move || {
                    tx.blocking_send(state).unwrap();
                });
                tokio_runtime.shutdown_background();
            }

            was_minimized = is_minimized;

            open_task_prompt = open_task_prompt
                || task_prompt_manager_rx
                    .try_recv()
                    .map(|state| state == TaskPromptManagerState::AwaitingPrompt)
                    .unwrap_or(false);

            if open_task_prompt {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
    })
    .unwrap();

    if task_prompt_manager.get_state() != TaskPromptManagerState::AwaitingPrompt {
        task_prompt_manager
            .tx
            .clone()
            .try_send(TaskPromptManagerState::Stopped)
            .unwrap();
    }
}

fn spawn_task_prompt(
    database_connection: Arc<Mutex<SqliteConnection>>,
    settings: Arc<Mutex<Settings>>,
) {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_always_on_top()
            .with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    let mut task_prompt_widget = TaskPromptWidget::new(TaskPrompt::new(
        database_connection,
        Arc::new(RwLock::new(LatestTaskManager::new(settings))),
    ));

    eframe::run_simple_native("Task Prompt", native_options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(&mut task_prompt_widget);
        });
    })
    .unwrap();
}

#[allow(unused)]
fn spawn_fake_window() {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_active(false)
            .with_inner_size([0.0, 0.0]),
        ..Default::default()
    };
    eframe::run_simple_native("Kronos", native_options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |_ui| {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        });
    })
    .unwrap();
}
