use std::{error::Error, thread::Thread, time::Duration};

use tokio::{sync::mpsc, time::sleep};

#[derive(PartialEq, Debug, Clone)]
pub enum KronosState {
    UiOpen,
    PendingPrompt,
    AwaitingPrompt,
    Closed,
}

pub struct KronosManager {
    pub state: KronosState,
    tx: mpsc::Sender<KronosState>,
    pub rx: mpsc::Receiver<KronosState>,
    // taskPromptWait: Thread
}

impl KronosManager {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<KronosState>(32);
        KronosManager {
            state: KronosState::PendingPrompt,
            tx,
            rx,
        }
    }

    pub fn change_state(&mut self, state: KronosState) {
        self.state = state.clone();

        let sender = self.tx.clone();
        tokio::spawn(async move {
            sender.send(state).await.unwrap();
        });
    }

    // pub fn get_receiver(&self) -> mpsc::Receiver<KronosState> {
    //     self.rx.clone()
    // }

    pub async fn start(&mut self) {
        while self.state != KronosState::Closed {
            tokio::select! {Some(state) = self.rx.recv() => {
                println!("{:?}", state);
            }
            _ = sleep(Duration::from_millis(1000)) => {
                println!("Slept")
            }}
        }
    }
}
