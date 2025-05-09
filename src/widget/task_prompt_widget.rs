use egui::Button;

use crate::task_prompt::TaskPrompt;

use super::task_select_widget::TaskSelectWidget;

pub struct TaskPromptWidget {
    task_prompt: TaskPrompt,
    task_select_widget: TaskSelectWidget,
}

impl TaskPromptWidget {
    pub fn new(task_prompt: TaskPrompt) -> Self {
        let mut task_select_widget = TaskSelectWidget::new(
            String::new(),
            task_prompt
                .task_options
                .iter()
                .map(|task| task.name.clone())
                .collect(),
        );
        task_select_widget.max_height = Some(100.0);
        TaskPromptWidget {
            task_prompt,
            task_select_widget,
        }
    }

    fn update_task_performed(&mut self, ui: &mut egui::Ui) {
        self.task_prompt.task_name_option = self.task_select_widget.get_input_text().to_string();
        self.task_prompt.update_task();
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
    }
}

impl egui::Widget for &mut TaskPromptWidget {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.vertical_centered(|ui| {
            ui.label(format!(
                "What have you been doing for the past '{}' minutes?",
                self.task_prompt.get_time_spent_minutes()
            ));

            ui.add(&mut self.task_select_widget);

            if self.task_select_widget.did_select_option {
                self.update_task_performed(ui);
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    // Make sure there is space for the second button by using with_layout
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
                        // "Accept" button on the right
                        if ui
                            .add_enabled(
                                self.task_select_widget.get_input_text() != "",
                                Button::new("Accept"),
                            )
                            .clicked()
                        {
                            self.update_task_performed(ui);
                        }
                    });
                })
            });
        });

        ui.response()
    }
}
