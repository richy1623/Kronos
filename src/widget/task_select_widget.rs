use egui::{Id, ScrollArea};
use rand::Rng;

use crate::model::task::Task;

pub struct TaskSelectWidget {
    task_options: Vec<String>,
    available_task_options: Vec<String>,
    input_text: String,
    widget_id: Id,
    pub did_select_option: bool,
    did_click_option: bool,
    pub desired_width: Option<f32>,
    pub max_height: Option<f32>,
}

impl TaskSelectWidget {
    pub fn new(initial_task_name: String, task_options: Vec<String>) -> Self {
        let available_task_options = task_options.clone();

        TaskSelectWidget {
            task_options,
            available_task_options,
            input_text: initial_task_name,
            widget_id: Id::new(rand::thread_rng().gen::<u64>()),
            did_select_option: false,
            did_click_option: false,
            desired_width: None,
            max_height: None,
        }
    }

    pub fn get_input_text(&self) -> &str {
        &self.input_text
    }

    fn update_available_options(&mut self) {
        self.available_task_options =
            Task::filter_all_matching_tasks(&self.task_options, &self.input_text)
                .iter()
                .map(|&s| s.clone())
                .collect();
    }
}

impl egui::Widget for &mut TaskSelectWidget {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let mut text_edit = egui::TextEdit::singleline(&mut self.input_text);

        if let Some(desired_width) = self.desired_width {
            text_edit = text_edit.desired_width(desired_width);
        }

        let task_name_text_edit = ui.add(text_edit);

        if task_name_text_edit.changed() || self.did_click_option {
            self.update_available_options();
        }

        self.did_click_option = false;

        let popup_id = self.widget_id;

        egui::popup_below_widget(
            ui,
            popup_id,
            &task_name_text_edit,
            egui::PopupCloseBehavior::CloseOnClick,
            |ui| {
                if let Some(max_height) = self.max_height {
                    ui.set_max_height(max_height);
                }

                ScrollArea::vertical()
                    .max_height(f32::INFINITY)
                    .show(ui, |ui| {
                        for task_option in &self.available_task_options {
                            if ui.button(task_option).clicked() {
                                println!("Clicked: {}", task_option);
                                self.input_text = task_option.to_string();
                                self.did_click_option = true;
                                ui.memory_mut(|m| m.close_popup());
                            }
                        }
                    });
            },
        );

        if task_name_text_edit.gained_focus() {
            ui.memory_mut(|m| m.open_popup(popup_id));
        }

        self.did_select_option =
            task_name_text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

        ui.response()
    }
}
