use iced::{
    overlay::{
        self,
        menu::{self, Menu},
    },
    widget::{self, Button, Column, PickList, Row, Text, TextInput},
    Point,
};

#[derive(Debug, Clone)]
pub enum TaskPromptWidgetMessage {
    Confirm,
    Cancel,
    ContentChanged(String),
    SuggestionSelected(String),
}

// #[derive(Default)]
pub struct TaskPromptWidget {
    task_name_input: String,
    suggestions: Vec<String>,
    available_suggestions: Vec<String>,
}

impl Default for TaskPromptWidget {
    fn default() -> Self {
        Self {
            task_name_input: String::new(),
            // TODO
            suggestions: vec![
                "apple".to_string(),
                "apple2".to_string(),
                "ale".to_string(),
                "ales".to_string(),
                "axe".to_string(),
                "bee".to_string(),
                "code".to_string(),
            ],
            available_suggestions: Vec::new(),
        }
    }
}

impl TaskPromptWidget {
    pub fn new() -> Self {
        let task_prompt_widget = Self {
            // TODO
            suggestions: vec![
                "apple".into(),
                "ale".into(),
                "axe".into(),
                "bee".into(),
                "code".into(),
            ],
            ..Self::default()
        };

        println!(">: {:?}", task_prompt_widget.suggestions);
        task_prompt_widget
    }

    pub fn update(&mut self, message: TaskPromptWidgetMessage) {
        match message {
            TaskPromptWidgetMessage::Confirm => {
                println!("Confirmed: {}", self.task_name_input);
                // TODO update db then close
            }
            TaskPromptWidgetMessage::Cancel => {
                println!("Exited");
                std::process::exit(0);
            }
            TaskPromptWidgetMessage::ContentChanged(value) => {
                println!("typed: {value}");
                self.task_name_input = value.clone();
                self.available_suggestions = self
                    .suggestions
                    .iter()
                    .filter(|suggestion| suggestion.starts_with(&value))
                    .cloned()
                    .collect();
                println!("available_suggestions: {:?}", self.available_suggestions);
                println!("suggestions: {:?}", self.suggestions);
            }
            TaskPromptWidgetMessage::SuggestionSelected(suggestion) => {
                self.task_name_input = suggestion;
                self.available_suggestions.clear();
                print!("chosen")
            }
        }
    }

    pub fn view(&self) -> Column<TaskPromptWidgetMessage> {
        // Input for the task name
        let input = TextInput::new("Task Name", &self.task_name_input)
            .on_input(TaskPromptWidgetMessage::ContentChanged);

        // Suggestion list wrapped in buttons
        let suggestions = widget::scrollable(
            self.available_suggestions
                .iter()
                .fold(Column::new().spacing(5), |column, suggestion| {
                    column.push(Button::new(Text::new(suggestion)).on_press(
                        TaskPromptWidgetMessage::SuggestionSelected(suggestion.clone()),
                    ))
                })
                .height(120),
        );

        // Confirmation and cancellation buttons
        let buttons = Row::new()
            .spacing(10)
            .push(Button::new(Text::new("Confirm")).on_press(TaskPromptWidgetMessage::Confirm))
            .push(Button::new(Text::new("Cancel")).on_press(TaskPromptWidgetMessage::Cancel));

        // Main layout
        Column::new()
            .spacing(10)
            .push(Text::new("What are you currently doing?"))
            .push(input)
            // .push(pick_list)
            .push(suggestions)
            .push(buttons)
    }
}
