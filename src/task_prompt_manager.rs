use std::{sync::Arc, time::Duration};

use tokio::{
    sync::{broadcast, mpsc, Mutex},
    task::JoinHandle,
    time::sleep,
};

#[derive(PartialEq, Debug, Clone)]
pub enum TaskPromptManagerState {
    UiOpen,
    PendingPrompt,
    AwaitingPrompt,
    Stopped,
}
const SLEEP_DURATION_MILLIS: u64 = 3000;

pub struct TaskPromptManager {
    last_known_state: TaskPromptManagerState,
    tx: mpsc::Sender<TaskPromptManagerState>,
    rx: Arc<Mutex<mpsc::Receiver<TaskPromptManagerState>>>,
    broadcast_tx: broadcast::Sender<TaskPromptManagerState>,
    task_prompt_wait: Option<JoinHandle<()>>, // TODO rename
}

impl TaskPromptManager {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<TaskPromptManagerState>(32);
        let (broadcast_tx, _) = broadcast::channel::<TaskPromptManagerState>(16);
        TaskPromptManager {
            last_known_state: TaskPromptManagerState::Stopped,
            tx,
            rx: Arc::new(Mutex::new(rx)),
            broadcast_tx,
            task_prompt_wait: None,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<TaskPromptManagerState> {
        self.broadcast_tx.subscribe()
    }

    pub fn change_state(&mut self, state: TaskPromptManagerState) {
        self.last_known_state = state.clone();

        let sender = self.tx.clone();
        let _ = self.broadcast_tx.send(state.clone());
        tokio::spawn(async move {
            sender.send(state).await.unwrap();
        });
    }

    pub async fn get_state(&mut self) -> TaskPromptManagerState {
        let rx = Arc::clone(&self.rx);
        if let Ok(received_state) = tokio::spawn(async move { rx.lock().await.try_recv() })
            .await
            .unwrap()
        {
            self.last_known_state = received_state;
        }

        return self.last_known_state.clone();
    }

    pub async fn stop(&mut self) {
        self.change_state(TaskPromptManagerState::Stopped);

        if let Some(handle) = self.task_prompt_wait.take() {
            handle.await.expect("todo");
        }
    }

    pub async fn start(&mut self) {
        let rx = Arc::clone(&self.rx);
        self.change_state(TaskPromptManagerState::PendingPrompt);
        let mut current_state = self.last_known_state.clone();

        let broadcast_tx = self.broadcast_tx.clone();
        let tx = self.tx.clone();

        let task_prompt_manager_run_handle = tokio::spawn(async move {
            let mut rx = rx.lock().await; // Lock the receiver
            'running_loop: loop {
                if current_state == TaskPromptManagerState::PendingPrompt {
                    tokio::select! {Some(state) = rx.recv() => {
                        println!("{:?}", state);
                        match state {
                            TaskPromptManagerState::PendingPrompt => (),
                            TaskPromptManagerState::Stopped => break,
                            _ => (), // TODO this should probably only be awaiting
                        }
                        current_state = state;
                    }
                    _ = sleep(Duration::from_millis(SLEEP_DURATION_MILLIS)) => {
                        let _ = tx.send(TaskPromptManagerState::AwaitingPrompt).await;
                        let _ = broadcast_tx.send(TaskPromptManagerState::AwaitingPrompt);
                        // self.change_state(TaskPromptManagerState::PendingPrompt);
                    }};
                } else {
                    loop {
                        match rx.recv().await {
                            Some(TaskPromptManagerState::PendingPrompt) => {
                                current_state = TaskPromptManagerState::PendingPrompt;
                                break;
                            }
                            Some(TaskPromptManagerState::Stopped) => {
                                break 'running_loop;
                            }
                            Some(_state) => {
                                continue;
                            }
                            None => {
                                log::error!("MPSC Channel has failed");
                                break 'running_loop;
                            }
                        }
                    }
                }
            }
        });
        self.task_prompt_wait = Some(task_prompt_manager_run_handle);
    }
}
