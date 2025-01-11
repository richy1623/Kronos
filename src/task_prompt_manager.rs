use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

use tokio::{sync::broadcast, sync::mpsc, time::sleep};

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TaskPromptManagerState {
    UiOpen,
    PendingPrompt,
    AwaitingPrompt,
    Stopped,
}
const SLEEP_DURATION_MILLIS: u64 = 3000;

pub struct TaskPromptManager {
    state: Arc<Mutex<TaskPromptManagerState>>,
    pub tx: mpsc::Sender<TaskPromptManagerState>,
    rx: Arc<Mutex<mpsc::Receiver<TaskPromptManagerState>>>,
    broadcast_tx: broadcast::Sender<TaskPromptManagerState>,
    task_prompt_manager_join_handle: Option<JoinHandle<()>>,
}

impl TaskPromptManager {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<TaskPromptManagerState>(8);
        let (broadcast_tx, _) = broadcast::channel::<TaskPromptManagerState>(8);
        TaskPromptManager {
            state: Arc::new(Mutex::new(TaskPromptManagerState::Stopped)),
            tx,
            rx: Arc::new(Mutex::new(rx)),
            broadcast_tx,
            task_prompt_manager_join_handle: None,
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

        let task_prompt_manager_run_handle = thread::spawn(move || {
            let mut rx = rx.lock().unwrap(); // Lock the receiver
            loop {
                let current_state = state.lock().unwrap();

                match *current_state {
                    TaskPromptManagerState::Stopped => break,
                    TaskPromptManagerState::PendingPrompt => {
                        std::mem::drop(current_state);
                        rt.block_on(async {
                            tokio::select! {
                                Some(new_state) = rx.recv() => {
                                    *state.lock().unwrap() = new_state;
                                },
                                _ = sleep(Duration::from_millis(SLEEP_DURATION_MILLIS)) => {
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
    use super::*;
    use rstest::rstest;

    async fn change_state(
        task_prompt_manager: &mut TaskPromptManager,
        state: TaskPromptManagerState,
    ) {
        *task_prompt_manager.state.lock().unwrap() = state.clone();

        let sender = task_prompt_manager.tx.clone();
        sender.send(state).await.unwrap();

        let _ = task_prompt_manager.broadcast_tx.send(state);
    }

    #[rstest]
    #[tokio::test]
    async fn test_initial_state() {
        let manager = TaskPromptManager::new();
        assert_eq!(
            manager.get_state(),
            TaskPromptManagerState::Stopped,
            "Initial state should be Stopped"
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_start_changes_state_to_pending_prompt() {
        let mut manager = TaskPromptManager::new();
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
    async fn test_start_then_drop() {
        let mut manager = TaskPromptManager::new();
        manager.start();

        std::mem::drop(manager);
    }

    #[rstest]
    #[tokio::test]
    async fn test_drop_immediate() {
        let manager = TaskPromptManager::new();

        std::mem::drop(manager);
    }

    #[rstest]
    #[tokio::test]
    async fn test_stop_changes_state_to_stopped() {
        let mut manager = TaskPromptManager::new();
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
    async fn test_state_transition_via_mpsc_channel() {
        let mut manager = TaskPromptManager::new();
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
    async fn test_broadcasting_state_changes() {
        let mut manager = TaskPromptManager::new();
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
    async fn test_broadcast_no_more_receivers() {
        let mut manager = TaskPromptManager::new();
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
    async fn test_state_resets_after_sleep_duration() {
        let mut manager = TaskPromptManager::new();
        manager.start();

        tokio::time::sleep(Duration::from_millis(SLEEP_DURATION_MILLIS + 500)).await; // Wait for sleep duration
        let state = manager.get_state();

        assert_eq!(
            state,
            TaskPromptManagerState::AwaitingPrompt,
            "State should transition to AwaitingPrompt after sleep duration"
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_state_change_resets_sleep_duration() {
        let mut manager = TaskPromptManager::new();
        manager.start();

        tokio::time::sleep(Duration::from_millis(SLEEP_DURATION_MILLIS / 2)).await;

        // Reset the prompt timer
        manager
            .tx
            .clone()
            .send(TaskPromptManagerState::PendingPrompt)
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(SLEEP_DURATION_MILLIS / 2 + 500)).await; // Wait for sleep duration
        let state = manager.get_state();
        assert_eq!(
            state,
            TaskPromptManagerState::PendingPrompt,
            "State should still be PendingPrompt after reset"
        );

        tokio::time::sleep(Duration::from_millis(SLEEP_DURATION_MILLIS / 2)).await;
        let state = manager.get_state();
        assert_eq!(
            state,
            TaskPromptManagerState::AwaitingPrompt,
            "State should transition to AwaitingPrompt after sleep duration"
        );
    }
}
