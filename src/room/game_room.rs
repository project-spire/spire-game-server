use bevy_ecs::prelude::*;
use tokio::sync::{broadcast, mpsc};

pub async fn run_room(
    mut in_message_rx: mpsc::Receiver<Vec<u8>>,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    let mut world = World::default();

    loop {
        tokio::select! {
            _ = update(&mut world) => {}
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

async fn update(world: &mut World) {
    println!("Room update");
    let mut schedule = Schedule::default();
    //TODO: Add systems
    // schedule.run(&mut world);

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
}
