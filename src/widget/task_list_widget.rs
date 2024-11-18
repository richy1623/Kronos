use chrono::{Local, NaiveDate};
use egui::Button;

use crate::task_list::{TaskList, TaskListItem};

struct TaskPerformedEdit {
    task_list_item: TaskListItem,
    new_task_name: String,
    new_task_time_minutes: String,
}

impl TaskPerformedEdit {
    fn new(task_list_item: &TaskListItem) -> Self {
        TaskPerformedEdit {
            task_list_item: task_list_item.clone(),
            new_task_name: task_list_item.task_name.clone(),
            new_task_time_minutes: task_list_item.task_performed.time_spent.to_string(),
        }
    }
}

pub struct TaskListWidget {
    task_list: TaskList,
    date: NaiveDate,
    editable_task_id: Option<TaskPerformedEdit>,
}

impl TaskListWidget {
    pub fn new(task_list: TaskList) -> Self {
        TaskListWidget {
            task_list,
            date: Local::now().naive_local().date(),
            editable_task_id: None,
        }
    }
}
impl egui::Widget for &mut TaskListWidget {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.label(format!("Listing all tasks for {}", self.date));

        // create a static char width
        let char_width = ui.fonts(|fonts| fonts.glyph_width(&egui::FontId::default(), 'a'));
        println!("{}", char_width);
        let char_width = ui.fonts(|fonts| fonts.glyph_width(&egui::FontId::default(), 'a'));
        ui.label(char_width.to_string());

        for task in self.task_list.list_all_tasks_performed() {
            ui.horizontal(|ui| match &mut self.editable_task_id {
                Some(x)
                    if x.task_list_item.task_performed.task_id == task.task_performed.task_id =>
                {
                    // let task_name_length = task.task_name.len();
                    ui.add(egui::TextEdit::singleline(&mut x.new_task_name).desired_width(300.0));

                    ui.add(
                        egui::TextEdit::singleline(&mut x.new_task_time_minutes)
                            .desired_width(30.0),
                    );
                    if ui
                        .add_enabled(
                            x.new_task_name != ""
                                && x.new_task_time_minutes.parse::<u32>().is_ok()
                                && (x.new_task_name != x.task_list_item.task_name
                                    || x.new_task_time_minutes
                                        != x.task_list_item.task_performed.time_spent.to_string()),
                            Button::new("Accept"),
                        )
                        .clicked()
                    {
                        todo!("update task");
                    }
                    if ui.button("x").clicked() {
                        self.editable_task_id = None;
                    }
                }
                _ => {
                    ui.label(&task.task_name);
                    ui.label(format!("{}", task.task_performed.time_spent));
                    if ui.button("edit").clicked() {
                        self.editable_task_id = Some(TaskPerformedEdit::new(&task));
                    }
                }
            });
        }
        ui.response()
    }
}
