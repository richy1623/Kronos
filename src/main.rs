use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use chrono::Local;
use kronos::{
    task_list::TaskList,
    task_prompt::TaskPrompt,
    task_prompt_manager::{self, TaskPromptManager},
    widget::{task_list_widget::TaskListWidget, task_prompt_widget::TaskPromptWidget},
};

#[derive(Debug)]
enum Command {
    // UpdateSettings,
    OpenTaskList,
    CloseTaskList,
    // SpawnTaskPrompt,
    // CloseTaskPrompt,
    Exit,
}

#[tokio::main]
async fn main() {
    let mut task_prompt_manager = TaskPromptManager::new();

    task_prompt_manager.start().await;

    let mut task_prompt_manager_rx = task_prompt_manager.subscribe();
    while let Ok(cmd) = task_prompt_manager_rx.recv().await {
        match cmd {
            task_prompt_manager::TaskPromptManagerState::UiOpen => (),
            task_prompt_manager::TaskPromptManagerState::PendingPrompt => (),
            task_prompt_manager::TaskPromptManagerState::AwaitingPrompt => {
                task_prompt_manager
                    .change_state(task_prompt_manager::TaskPromptManagerState::UiOpen);
                spawn_task_prompt();
                // spawn_task_list(true);
                task_prompt_manager
                    .change_state(task_prompt_manager::TaskPromptManagerState::PendingPrompt);
                spawn_fake_window();
            }
            task_prompt_manager::TaskPromptManagerState::Stopped => break,
        }
    }

    task_prompt_manager.stop().await;
}

fn spawn_task_list(spawn_active: bool) {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 600.0])
            .with_active(spawn_active)
            .with_visible(spawn_active),
        ..Default::default()
    };

    let mut task_list_widget = TaskListWidget::new(TaskList::new(
        Arc::new(Mutex::new(kronos::establish_connection())),
        Local::now().date_naive(),
    ));

    eframe::run_simple_native("Task List", native_options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(&mut task_list_widget);
            // ui.ctx()
            //     .send_viewport_cmd(egui::ViewportCommand::Minimized(true));
            // println!("{}", ui.is_visible());
            // ui.vis
            // thread::sleep(Duration::from_secs(2));
        });
    })
    .unwrap();
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

fn spawn_fake_window() {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_active(false)
            .with_inner_size([0.0, 0.0])
            .with_visible(false),
        ..Default::default()
    };
    eframe::run_simple_native("Kronos", native_options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        });
    })
    .unwrap();
}
