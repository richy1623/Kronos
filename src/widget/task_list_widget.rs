use chrono::{Local, NaiveDate};

use crate::task_list::TaskList;

pub struct TaskListWidget {
    task_list: TaskList,
    date: NaiveDate,
}

impl TaskListWidget {
    pub fn new(task_list: TaskList) -> Self {
        TaskListWidget {
            task_list,
            date: Local::now().naive_local().date(),
        }
    }
}
impl egui::Widget for &mut TaskListWidget {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.label(format!("Listing all tasks for {}", self.date));
        for task in self.task_list.list_all_tasks_performed() {
            ui.horizontal(|ui| {
                ui.label(task.task_name.clone());
                ui.label(format!("{}", task.task_performed.time_spent));
            });
        }
        ui.response()
    }
}
