use kronos::ui::task_prompt_widget::TaskPromptWidget;

fn main() -> iced::Result {
    iced::run(
        "A cool counter",
        TaskPromptWidget::update,
        TaskPromptWidget::view,
    )
}
