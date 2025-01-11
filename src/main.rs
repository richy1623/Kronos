use std::sync::{Arc, Mutex};

use chrono::Local;
use kronos::{
    task_list::TaskList,
    task_prompt::TaskPrompt,
    task_prompt_manager::{self, TaskPromptManager, TaskPromptManagerState},
    widget::{task_list_widget::TaskListWidget, task_prompt_widget::TaskPromptWidget},
};

#[tokio::main]
async fn main() {
    let mut task_prompt_manager = TaskPromptManager::new();

    task_prompt_manager.start();

    let mut task_prompt_manager_rx = task_prompt_manager.subscribe();
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
                spawn_task_prompt();
                // spawn_fake_window();
                spawn_task_list(false, &mut task_prompt_manager);
            }
            task_prompt_manager::TaskPromptManagerState::Stopped => break,
        }
    }
}

fn spawn_task_list(spawn_active: bool, task_prompt_manager: &TaskPromptManager) {
    let mut task_prompt_manager_rx = task_prompt_manager.subscribe();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 600.0])
            .with_visible(spawn_active)
            .with_active(spawn_active),
        ..Default::default()
    };

    let mut task_list_widget = TaskListWidget::new(TaskList::new(
        Arc::new(Mutex::new(kronos::establish_connection())),
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
                ui.ctx().request_repaint_after_secs(1.0); // Force repaint to keep updates flowing
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

            // let close_requested = ui.ctx().input(|i| i.viewport().close_requested());

            open_task_prompt = open_task_prompt
                || task_prompt_manager_rx
                    .try_recv()
                    .map(|state| state == TaskPromptManagerState::AwaitingPrompt)
                    .unwrap_or(false);

            // dbg!(is_minimized);
            // ui.ctx()
            //     .send_viewport_cmd(egui::ViewportCommand::Minimized(true));
            // println!("{}", ui.is_visible());
            // ui.vis
            // thread::sleep(Duration::from_secs(2));

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
    // println!(was_minimized);
    println!("exits");
    // tx.send(Command::CloseTaskList).await.unwrap();
}

fn spawn_task_prompt() {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_always_on_top()
            .with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    let mut task_prompt_widget = TaskPromptWidget::new(TaskPrompt::new(Arc::new(Mutex::new(
        kronos::establish_connection(),
    ))));

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
            .with_inner_size([0.0, 0.0])
            .with_visible(false),
        ..Default::default()
    };
    eframe::run_simple_native("Kronos", native_options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |_ui| {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        });
    })
    .unwrap();
}
