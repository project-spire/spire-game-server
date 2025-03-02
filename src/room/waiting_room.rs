use tokio::sync::{broadcast, mpsc};

pub async fn run(
    mut in_message_rx: mpsc::Receiver<Vec<u8>>,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    loop {
        tokio::select! {
            result = in_message_rx.recv() => match result {
                Some(message) => {
                    handle(message);
                }
                None => { break; }
            },
            _ = shutdown_rx.recv() => { break; },
        }
    }
}

fn handle(message: Vec<u8>) {}