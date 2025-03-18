use bevy_ecs::prelude::*;
use crate::character;
use crate::core::room::RoomMessage;
use crate::core::session::InMessage;
use crate::world::time::WorldTime;
use tokio::sync::{broadcast, mpsc};
use tokio::time::{self, Duration};

pub async fn run(
    mut in_message_rx: mpsc::Receiver<InMessage>,
    mut room_message_rx: mpsc::Receiver<RoomMessage>,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    let mut in_message_buffer = Vec::with_capacity(64);
    let mut room_message_buffer =  Vec::with_capacity(16);
    let mut update_interval = time::interval(Duration::from_millis(100));
    
    let mut world = World::default();
    world.insert_resource(WorldTime::default());

    loop {
        tokio::select! {
            _ = update_interval.tick() => {
                update(&mut world);
            },
            n = in_message_rx.recv_many(&mut in_message_buffer, 64) => {
                if n == 0 {
                    break;
                }

                for message in in_message_buffer.drain(0..n) {
                    handle_in_message(&mut world, message);
                }
            },
            n = room_message_rx.recv_many(&mut room_message_buffer, 16) => {
                if n == 0 {
                    break;
                }

                for message in room_message_buffer.drain(0..n) {
                    handle_room_message(&mut world, message);
                }
            },
            _ = shutdown_rx.recv() => break,
        }
    }
}

fn update(world: &mut World) {
    let mut time = world.get_resource_mut::<WorldTime>().unwrap();
    let now = std::time::Instant::now();
    let dt = now - time.now;

    time.now = now;
    time.dt = dt;

    let mut schedule = Schedule::default();
    schedule.add_systems((
        character::movement::update,
        character::movement::sync,
    ));

    schedule.run(world);
}

fn handle_in_message(world: &mut World, message: InMessage) {
    let (session_ctx, protocol, data) = message;
}

fn handle_room_message(world: &mut World, message: RoomMessage) {
    
}