use egui::{Id, ScrollArea};

use crate::{model::task::Task, task_prompt::TaskPrompt};

pub struct TaskPromptWidget {
    task_prompt: TaskPrompt,
}

impl TaskPromptWidget {
    pub fn new(task_prompt: TaskPrompt) -> Self {
        TaskPromptWidget { task_prompt }
    }
}

impl egui::Widget for &mut TaskPromptWidget {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let task_name_text_edit = ui.text_edit_singleline(&mut self.task_prompt.task_name_option);

        let popup_id = Id::new("popup");

        egui::popup_below_widget(
            ui,
            popup_id,
            &task_name_text_edit,
            egui::PopupCloseBehavior::CloseOnClick,
            |ui| {
                ui.set_max_height(100.0);

                ScrollArea::vertical()
                    .max_height(f32::INFINITY)
                    .show(ui, |ui| {
                        for task_option in &self.task_prompt.available_task_options {
                            if ui.button(task_option).clicked() {
                                println!("Clicked: {}", task_option);
                                self.task_prompt.task_name_option = task_option.to_string();
                                ui.memory_mut(|m| m.close_popup());
                            }
                        }
                    });
            },
        );

        if task_name_text_edit.gained_focus() {
            ui.memory_mut(|m| m.open_popup(popup_id));
        }

        // if task_name_text_edit.has_focus() {
        //     for task_option in &self.task_prompt.available_task_options {
        //         let button = ui.button(task_option);
        //         if button.clicked() {
        //             self.task_prompt.task_name_option = task_option.to_string();
        //             return ui.response();
        //             // task_name_text_edit.surrender_focus();
        //         }
        //     }
        // }
        // let response = ui.add(egui::TextEdit::singleline(&mut self.task_name));
        if task_name_text_edit.changed() {
            self.task_prompt.available_task_options = Task::filter_all_matching_tasks(
                &self.task_prompt.task_options,
                &self.task_prompt.task_name_option,
            )
            .iter()
            .map(|&s| s.name.clone())
            .collect();
        }

        if task_name_text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            println!("selected: {}", self.task_prompt.task_name_option);
            self.task_prompt.update_task(10);
            ui.close_menu();
        }

        ui.response()
    }
}
