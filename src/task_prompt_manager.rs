use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use tokio::{sync::broadcast, sync::mpsc, time::sleep};

use crate::settings::Settings;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TaskPromptManagerState {
    UiOpen,
    PendingPrompt,
    AwaitingPrompt,
    Stopped,
}

pub struct TaskPromptManager {
    state: Arc<Mutex<TaskPromptManagerState>>,
    pub tx: mpsc::Sender<TaskPromptManagerState>,
    rx: Arc<Mutex<mpsc::Receiver<TaskPromptManagerState>>>,
    broadcast_tx: broadcast::Sender<TaskPromptManagerState>,
    task_prompt_manager_join_handle: Option<JoinHandle<()>>,
    settings: Arc<Mutex<Settings>>,
}

impl TaskPromptManager {
    pub fn new(settings: Arc<Mutex<Settings>>) -> Self {
        let (tx, rx) = mpsc::channel::<TaskPromptManagerState>(8);
        let (broadcast_tx, _) = broadcast::channel::<TaskPromptManagerState>(8);
        TaskPromptManager {
            state: Arc::new(Mutex::new(TaskPromptManagerState::Stopped)),
            tx,
            rx: Arc::new(Mutex::new(rx)),
            broadcast_tx,
            task_prompt_manager_join_handle: None,
            settings,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<TaskPromptManagerState> {
        self.broadcast_tx.subscribe()
    }

    pub fn get_state(&self) -> TaskPromptManagerState {
        return self.state.lock().unwrap().clone();
    }

    pub fn start(&mut self) {
        if let Some(handle) = &self.task_prompt_manager_join_handle {
            if !handle.is_finished() {
                return;
            }
        }

        let rx = Arc::clone(&self.rx);
        let rt = tokio::runtime::Runtime::new().unwrap();
        let broadcast_tx = self.broadcast_tx.clone();

        *self.state.lock().unwrap() = TaskPromptManagerState::PendingPrompt;
        let state = Arc::clone(&self.state);

        let _ = broadcast_tx.send(TaskPromptManagerState::PendingPrompt);

        let settings_handle = self.settings.clone();

        let task_prompt_manager_run_handle = thread::spawn(move || {
            let mut rx = rx.lock().unwrap(); // Lock the receiver
            loop {
                let current_state = state.lock().unwrap();

                match *current_state {
                    TaskPromptManagerState::Stopped => break,
                    TaskPromptManagerState::PendingPrompt => {
                        std::mem::drop(current_state);
                        rt.block_on(async {
                            let sleep_duration = {
                                settings_handle
                                    .lock()
                                    .unwrap()
                                    .get_task_prompt_delay()
                                    .clone()
                            };
                            tokio::select! {
                                Some(new_state) = rx.recv() => {
                                    *state.lock().unwrap() = new_state;
                                },
                                _ = sleep(sleep_duration) => {
                                    *state.lock().unwrap() = TaskPromptManagerState::AwaitingPrompt;
                                }
                            }
                        });
                    }
                    _other => {
                        std::mem::drop(current_state);
                        rt.block_on(async {
                            match rx.recv().await {
                                Some(new_state) => {
                                    *state.lock().unwrap() = new_state.clone();
                                }
                                None => todo!(),
                            }
                        });
                    }
                }

                let _ = broadcast_tx.send(state.lock().unwrap().clone());
            }
        });
        self.task_prompt_manager_join_handle = Some(task_prompt_manager_run_handle);
    }
}

impl Drop for TaskPromptManager {
    fn drop(&mut self) {
        // TODO refactor
        *self.state.lock().unwrap() = TaskPromptManagerState::Stopped;

        let sender = self.tx.clone();

        if let Some(handle) = self.task_prompt_manager_join_handle.take() {
            thread::spawn(|| {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .build()
                    .unwrap();

                runtime.spawn_blocking(move || {
                    sender
                        .blocking_send(TaskPromptManagerState::Stopped)
                        .unwrap()
                });
            })
            .join()
            .unwrap();

            handle.join().unwrap();
        }

        let _ = self.broadcast_tx.send(TaskPromptManagerState::Stopped);
    }
}

#[cfg(test)]
mod tests {
    use std::{ops::Add, sync::Arc, time::Duration};

    use super::*;
    use rstest::{fixture, rstest};
    use tempfile::TempDir;

    async fn change_state(
        task_prompt_manager: &mut TaskPromptManager,
        state: TaskPromptManagerState,
    ) {
        *task_prompt_manager.state.lock().unwrap() = state.clone();

        let sender = task_prompt_manager.tx.clone();
        sender.send(state).await.unwrap();

        let _ = task_prompt_manager.broadcast_tx.send(state);
    }

    #[fixture]
    pub fn settings() -> (Arc<Mutex<Settings>>, TempDir) {
        let temp_dir = TempDir::new().unwrap();

        (
            Arc::new(Mutex::new(Settings::from_dir(
                temp_dir.path().to_path_buf(),
            ))),
            temp_dir,
        )
    }

    #[rstest]
    #[tokio::test]
    async fn test_initial_state(settings: (Arc<Mutex<Settings>>, TempDir)) {
        let (settings, _temp_dir) = settings;
        let manager = TaskPromptManager::new(settings);
        assert_eq!(
            manager.get_state(),
            TaskPromptManagerState::Stopped,
            "Initial state should be Stopped"
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_start_changes_state_to_pending_prompt(settings: (Arc<Mutex<Settings>>, TempDir)) {
        let (settings, _temp_dir) = settings;
        let mut manager = TaskPromptManager::new(settings);
        manager.start();
        let state = manager.get_state();

        assert_eq!(
            state,
            TaskPromptManagerState::PendingPrompt,
            "State should transition to PendingPrompt after start"
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_start_then_drop(settings: (Arc<Mutex<Settings>>, TempDir)) {
        let (settings, _temp_dir) = settings;
        let mut manager = TaskPromptManager::new(settings);
        manager.start();

        std::mem::drop(manager);
    }

    #[rstest]
    #[tokio::test]
    async fn test_drop_immediate(settings: (Arc<Mutex<Settings>>, TempDir)) {
        let (settings, _temp_dir) = settings;
        let manager = TaskPromptManager::new(settings);

        std::mem::drop(manager);
    }

    #[rstest]
    #[tokio::test]
    async fn test_stop_changes_state_to_stopped(settings: (Arc<Mutex<Settings>>, TempDir)) {
        let (settings, _temp_dir) = settings;
        let mut manager = TaskPromptManager::new(settings);
        manager.start();

        change_state(&mut manager, TaskPromptManagerState::Stopped).await;

        let state = manager.get_state();

        assert_eq!(
            state,
            TaskPromptManagerState::Stopped,
            "State should transition to Stopped after stop"
        );

        tokio::time::sleep(Duration::from_millis(100)).await; // TODO rust awaitility

        assert!(
            manager
                .task_prompt_manager_join_handle
                .as_ref()
                .unwrap()
                .is_finished(),
            "Stop did not stop the thread."
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_state_transition_via_mpsc_channel(settings: (Arc<Mutex<Settings>>, TempDir)) {
        let (settings, _temp_dir) = settings;
        let mut manager = TaskPromptManager::new(settings);
        manager.start();

        let tx = manager.tx.clone();
        tx.send(TaskPromptManagerState::UiOpen).await.unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await; // TODO rust awaitility
        let state = manager.get_state();

        assert_eq!(
            state,
            TaskPromptManagerState::UiOpen,
            "State should transition to UiOpen when sent via mpsc channel"
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_broadcasting_state_changes(settings: (Arc<Mutex<Settings>>, TempDir)) {
        let (settings, _temp_dir) = settings;
        let mut manager = TaskPromptManager::new(settings);
        manager.start();

        let mut subscriber = manager.subscribe();

        change_state(&mut manager, TaskPromptManagerState::AwaitingPrompt).await;

        let received_state = subscriber.recv().await.unwrap();

        assert_eq!(
            received_state,
            TaskPromptManagerState::AwaitingPrompt,
            "Broadcast receiver should receive AwaitingPrompt state"
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_broadcast_no_more_receivers(settings: (Arc<Mutex<Settings>>, TempDir)) {
        let (settings, _temp_dir) = settings;
        let mut manager = TaskPromptManager::new(settings);
        manager.start();

        let mut subscriber = manager.subscribe();
        change_state(&mut manager, TaskPromptManagerState::AwaitingPrompt).await;

        let received_state = subscriber.recv().await.unwrap();

        assert_eq!(
            received_state,
            TaskPromptManagerState::AwaitingPrompt,
            "Broadcast receiver should receive AwaitingPrompt state"
        );

        std::mem::drop(subscriber);

        change_state(&mut manager, TaskPromptManagerState::PendingPrompt).await;
    }

    #[rstest]
    #[tokio::test]
    async fn test_state_resets_after_sleep_duration(settings: (Arc<Mutex<Settings>>, TempDir)) {
        let (settings, _temp_dir) = settings;
        {
            settings
                .lock()
                .unwrap()
                .update_task_prompt_delay(Duration::from_secs(1));
        }
        let task_prompt_delay = { settings.lock().unwrap().get_task_prompt_delay().clone() };

        let mut manager = TaskPromptManager::new(settings);
        manager.start();

        tokio::time::sleep(task_prompt_delay.add(Duration::from_millis(500))).await; // Wait for sleep duration
        let state = manager.get_state();

        assert_eq!(
            state,
            TaskPromptManagerState::AwaitingPrompt,
            "State should transition to AwaitingPrompt after sleep duration"
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_state_change_resets_sleep_duration(settings: (Arc<Mutex<Settings>>, TempDir)) {
        let (settings, _temp_dir) = settings;
        {
            settings
                .lock()
                .unwrap()
                .update_task_prompt_delay(Duration::from_secs(1));
        }
        let task_prompt_delay = { settings.lock().unwrap().get_task_prompt_delay().clone() };

        let mut manager = TaskPromptManager::new(settings);
        manager.start();

        tokio::time::sleep(
            task_prompt_delay
                .div_f32(2.0)
                .add(Duration::from_millis(250)),
        )
        .await;

        // Reset the prompt timer
        manager
            .tx
            .clone()
            .send(TaskPromptManagerState::PendingPrompt)
            .await
            .unwrap();

        tokio::time::sleep(
            task_prompt_delay
                .div_f32(2.0)
                .add(Duration::from_millis(400)),
        )
        .await; // Wait for sleep duration
        let state = manager.get_state();
        assert_eq!(
            state,
            TaskPromptManagerState::PendingPrompt,
            "State should still be PendingPrompt after reset"
        );

        tokio::time::sleep(task_prompt_delay.div_f32(2.0)).await;
        let state = manager.get_state();
        assert_eq!(
            state,
            TaskPromptManagerState::AwaitingPrompt,
            "State should transition to AwaitingPrompt after sleep duration"
        );
    }
}
