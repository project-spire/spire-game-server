use bevy_ecs::prelude::*;
use tokio::sync::{broadcast, mpsc};
use tokio::time::{self, Duration, Instant};
use crate::core::session::InMessage;

pub async fn run(
    mut in_message_rx: mpsc::Receiver<InMessage>,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    let mut world = World::default();
    let mut interval = time::interval(Duration::from_millis(100));
    let mut last_tick = Instant::now();;

    let mut message_buffer = Vec::with_capacity(64);

    loop {
        tokio::select! {
            _ = interval.tick() => {
                let now = Instant::now();
                update(&mut world, now - last_tick);
                last_tick = now;
            },
            n = in_message_rx.recv_many(&mut message_buffer, 64) => {
                if n == 0 {
                    break;
                }

                for message in message_buffer.drain(0..n) {
                    handle(message);
                }
            },
            _ = shutdown_rx.recv() => break,
        }
    }
}

fn update(world: &mut World, dt: Duration) {
    println!("Room update; dt={}", dt.as_secs());
    // let mut schedule = Schedule::new();
    //TODO: Add systems
    // schedule.run(world);
}

fn handle(message: InMessage) {
    let (session_ctx, protocol, data) = message;
}