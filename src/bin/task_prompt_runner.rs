#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)]
use std::sync::{Arc, Mutex};

// it's an example
use eframe::egui;
use kronos::{task_prompt::TaskPrompt, widget::task_prompt_widget::TaskPromptWidget};

fn main() {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    let task_prompt_widget = TaskPromptWidget::new(TaskPrompt::new(Arc::new(Mutex::new(
        kronos::establish_connection(),
    ))));
    eframe::run_native(
        "My egui App",
        native_options,
        Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc, task_prompt_widget)))),
    )
    .unwrap();
}

struct MyEguiApp {
    task_prompt_widget: TaskPromptWidget,
}

impl MyEguiApp {
    fn new(_cc: &eframe::CreationContext<'_>, task_prompt_widget: TaskPromptWidget) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self { task_prompt_widget }
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(&mut self.task_prompt_widget);
        });
    }
}
