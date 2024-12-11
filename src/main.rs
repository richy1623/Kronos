use std::sync::{Arc, Mutex};

use chrono::Local;
use kronos::{task_list::TaskList, widget::task_list_widget::TaskListWidget};
use tokio::sync::mpsc::{self, Sender};

#[derive(Debug)]
enum Command {
    // UpdateSettings,
    OpenTaskList,
    CloseTaskList,
    // SpawnTaskPrompt,
    // CloseTaskPrompt,
}

#[tokio::main]
async fn main() {
    // let connection = Arc::new(Mutex::new(kronos::establish_connection()));

    let (tx, mut rx) = mpsc::channel::<Command>(32);
    let tx2 = tx.clone();
    let manager = tokio::spawn(async move {
        // Establish a connection to the server

        // Start receiving messages
        while let Some(cmd) = rx.recv().await {
            use Command::*;

            match cmd {
                // UpdateSettings => todo!(),
                OpenTaskList => spawn_task_list(tx.clone()).await,
                CloseTaskList => todo!(),
                // CloseTaskPrompt => todo!(),
                // SpawnTaskPrompt => todo!(),
            }
        }
    });

    tx2.clone().send(Command::OpenTaskList).await.unwrap();
    manager.is_finished();
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
