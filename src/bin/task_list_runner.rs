#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)]
use std::sync::{Arc, Mutex};

use chrono::Local;
// it's an example
use eframe::egui;
use kronos::{settings::Settings, task_list::TaskList, widget::task_list_widget::TaskListWidget};

fn main() {
    let settings = Settings::new();
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_always_on_top()
            .with_inner_size([600.0, 600.0]),
        ..Default::default()
    };

    let task_list_widget = TaskListWidget::new(TaskList::new(
        Arc::new(Mutex::new(kronos::establish_connection(
            settings.get_database_file_path().to_str().unwrap(),
        ))),
        Local::now().date_naive(),
    ));
    eframe::run_native(
        "Task List",
        native_options,
        Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc, task_list_widget)))),
    )
    .unwrap();
}

struct MyEguiApp {
    task_list_widget: TaskListWidget,
}

impl MyEguiApp {
    fn new(_cc: &eframe::CreationContext<'_>, task_list_widget: TaskListWidget) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self { task_list_widget }
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(&mut self.task_list_widget);
        });
    }
}
