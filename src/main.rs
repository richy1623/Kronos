use std::{
    sync::{Arc, Mutex},
    thread,
};

use chrono::Local;
use kronos::{
    kronos_manager::{self, KronosManager},
    task_list::TaskList,
    widget::task_list_widget::TaskListWidget,
};
use tokio::sync::mpsc::{self, Sender};

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
    // let connection = Arc::new(Mutex::new(kronos::establish_connection()));

    let (tx, mut rx) = mpsc::channel::<Command>(32);

    // spawn_task_list(tx.clone()).await;
    // let manager = tokio::spawn(async move {
    //     // Establish a connection to the server

    //     // Start receiving messages
    let mut kronos_manager = KronosManager::new();
    let kronos_run_thread = thread::spawn(|| {
        // tokio::spawn(async {
        //     // &kronos_manager.start().await;
        // });
    });

    while let Some(cmd) = kronos_manager.rx.recv().await {
        match cmd {
            kronos_manager::KronosState::UiOpen => (),
            kronos_manager::KronosState::PendingPrompt => (),
            kronos_manager::KronosState::AwaitingPrompt => {
                kronos_manager.change_state(kronos_manager::KronosState::UiOpen);
                spawn_task_list(tx.clone()).await;
                kronos_manager.change_state(kronos_manager::KronosState::PendingPrompt);
            }
            kronos_manager::KronosState::Closed => break,
        }
    }
    // tx.send(Command::OpenTaskList).await.unwrap();
    // while let Some(cmd) = rx.recv().await {
    //     use Command::*;

    //     match cmd {
    //         // UpdateSettings => todo!(),
    //         OpenTaskList => spawn_task_list(tx.clone()).await,
    //         CloseTaskList => tx.send(Command::Exit).await.unwrap(),
    //         // CloseTaskPrompt => todo!(),
    //         // SpawnTaskPrompt => todo!(),
    //         Exit => break,
    //     }
    // }

    // });
    // manager.is_finished();
    kronos_run_thread.join().unwrap();
}

async fn spawn_task_list(tx: Sender<Command>) {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_always_on_top()
            .with_inner_size([600.0, 600.0]),
        ..Default::default()
    };

    let mut task_list_widget = TaskListWidget::new(TaskList::new(
        Arc::new(Mutex::new(kronos::establish_connection())),
        Local::now().date_naive(),
    ));

    eframe::run_simple_native("My egui App", native_options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(&mut task_list_widget);
        });
    })
    .unwrap();
    tx.send(Command::CloseTaskList).await.unwrap();
}
