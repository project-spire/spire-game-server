use tokio::sync::{broadcast, mpsc};

pub struct Room {
    id: u32,
    pub in_message_tx: mpsc::Sender<Vec<u8>>,
    in_message_rx: mpsc::Receiver<Vec<u8>>,
    shutdown_rx: broadcast::Receiver<()>,
}

impl Room {
    pub fn new(id: u32, shutdown_rx: broadcast::Receiver<()>) -> Room {
        let (in_message_tx, in_message_rx) = mpsc::channel::<Vec<u8>>(32);

        Room { id, in_message_tx, in_message_rx, shutdown_rx }
    }

    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                message = self.in_message_rx.recv() => match message {
                    Some(message) => {
                        println!("Received {} bytes of message", message.len());
                    }
                    None => {
                        break;
                    }
                },
                _ = self.shutdown_rx.recv() => {
                    break;
                }
            }
        }
    }
}
