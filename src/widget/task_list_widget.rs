use chrono::{Local, NaiveDate};
use egui::Button;

use crate::task_list::{TaskList, TaskListItem};

use super::task_select_widget::TaskSelectWidget;

struct TaskPerformedEdit {
    task_list_item: TaskListItem,
    task_name_select_widget: TaskSelectWidget,
    new_task_time_minutes: String,
}

impl TaskPerformedEdit {
    fn new(
        initial_task_name: String,
        task_list_item: &TaskListItem,
        task_options: Vec<String>,
    ) -> Self {
        let mut task_name_select_widget = TaskSelectWidget::new(initial_task_name, task_options);
        task_name_select_widget.desired_width = Some(300.0);
        task_name_select_widget.max_height = Some(100.0);
        TaskPerformedEdit {
            task_list_item: task_list_item.clone(),
            task_name_select_widget,
            new_task_time_minutes: task_list_item.task_performed.time_spent.to_string(),
        }
    }
}

struct TaskPerformedToAdd {
    task_name_select_widget: TaskSelectWidget,
    new_task_time_minutes: String,
}

impl TaskPerformedToAdd {
    fn new(task_options: Vec<String>) -> Self {
        TaskPerformedToAdd {
            task_name_select_widget: TaskSelectWidget::new(String::new(), task_options),
            new_task_time_minutes: String::new(),
        }
    }
}

pub struct TaskListWidget {
    task_list: TaskList,
    date: NaiveDate,
    tasks_to_display: Vec<TaskListItem>,
    tasks_to_display_require_reload: bool,
    editable_task_id: Option<TaskPerformedEdit>,
    editable_task_to_add: Option<TaskPerformedToAdd>,
}

impl TaskListWidget {
    pub fn new(task_list: TaskList) -> Self {
        TaskListWidget {
            task_list,
            date: Local::now().naive_local().date(),
            tasks_to_display: Vec::new(),
            tasks_to_display_require_reload: true,
            editable_task_id: None,
            editable_task_to_add: None,
        }
    }
}
impl egui::Widget for &mut TaskListWidget {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.label(format!("Listing all tasks for {}", self.date));

        // Collect tasks into a temporary vector to avoid borrowing conflicts
        if self.tasks_to_display_require_reload {
            self.tasks_to_display = self.task_list.list_all_tasks_performed().to_vec();
            self.tasks_to_display_require_reload = false;
        }

        for task in &self.tasks_to_display {
            ui.horizontal(|ui| match &mut self.editable_task_id {
                Some(task_performed_edit)
                    if task_performed_edit.task_list_item.task_performed.task_id
                        == task.task_performed.task_id =>
                {
                    // let task_name_length = task.task_name.len();
                    ui.add(&mut task_performed_edit.task_name_select_widget);

                    ui.add(
                        egui::TextEdit::singleline(&mut task_performed_edit.new_task_time_minutes)
                            .desired_width(30.0),
                    );
                    if ui
                        .add_enabled(
                            task_performed_edit.task_name_select_widget.get_input_text() != ""
                                && task_performed_edit
                                    .new_task_time_minutes
                                    .parse::<u32>()
                                    .is_ok()
                                && (task_performed_edit.task_name_select_widget.get_input_text()
                                    != task_performed_edit.task_list_item.task_name
                                    || task_performed_edit.new_task_time_minutes
                                        != task_performed_edit
                                            .task_list_item
                                            .task_performed
                                            .time_spent
                                            .to_string()),
                            Button::new("Accept"),
                        )
                        .clicked()
                    {
                        self.task_list.update_task_performed(
                            task_performed_edit.task_list_item.task_performed.task_id,
                            &task_performed_edit.task_name_select_widget.get_input_text(),
                            task_performed_edit
                                .new_task_time_minutes
                                .parse::<i32>()
                                .expect("Checked by UI before allowing update"),
                        );
                        self.editable_task_id = None;
                        self.tasks_to_display_require_reload = true;
                    }
                    if ui.button("x").clicked() {
                        self.editable_task_id = None;
                    }
                }
                _ => {
                    ui.label(&task.task_name);
                    ui.label(format!("{}", task.task_performed.time_spent));
                    if ui.button("edit").clicked() {
                        self.editable_task_id = Some(TaskPerformedEdit::new(
                            task.task_name.to_string(),
                            &task,
                            self.task_list.fetch_most_recent_task_names(1000),
                        ));
                    }
                }
            });
        }

        ui.horizontal(|ui| match &mut self.editable_task_to_add {
            Some(task_to_add) => {
                ui.add(&mut task_to_add.task_name_select_widget);

                ui.add(
                    egui::TextEdit::singleline(&mut task_to_add.new_task_time_minutes)
                        .desired_width(30.0),
                );
                if ui
                    .add_enabled(
                        task_to_add.task_name_select_widget.get_input_text() != ""
                            && task_to_add.new_task_time_minutes.parse::<u32>().is_ok(),
                        Button::new("Accept"),
                    )
                    .clicked()
                {
                    self.task_list.add_task(
                        task_to_add.task_name_select_widget.get_input_text(),
                        task_to_add
                            .new_task_time_minutes
                            .parse::<i32>()
                            .expect("Checked by UI before allowing update"),
                    );
                    self.editable_task_to_add = None;
                    self.tasks_to_display_require_reload = true;
                }
                if ui.button("x").clicked() {
                    self.editable_task_to_add = None;
                }
            }
            _ => {
                if ui.button("Add Task").clicked() {
                    self.editable_task_to_add = Some(TaskPerformedToAdd::new(
                        self.task_list.fetch_most_recent_task_names(1000),
                    ))
                }
            }
        });
        ui.response()
    }
}
