use tokio::sync::{broadcast, mpsc};

pub async fn run_room(
    mut in_message_rx: mpsc::Receiver<Vec<u8>>,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    loop {
        tokio::select! {
            _ = update() => {}
            result = in_message_rx.recv() => match result {
                Some(message) => {
                    println!("Received {} bytes of message", message.len());
                }
                None => { break; }
            },
            _ = shutdown_rx.recv() => { break; },
        }
    }
}

async fn update() {
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    println!("Room update");
}
