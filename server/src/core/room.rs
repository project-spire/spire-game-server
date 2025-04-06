use bevy_ecs::world::World;
use crate::core::server::ServerContext;
use crate::core::session::{InMessage, OutMessage, SessionContext};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use tokio::time;

pub enum RoomMessage {
    Broadcast(OutMessage),
    RoomTransferCommit { session_ctx: SessionContext },
}

pub struct RoomContext {
    pub message_tx: mpsc::Sender<RoomMessage>,
    pub in_message_tx: mpsc::Sender<InMessage>,
}

impl RoomContext {
    pub fn new(message_tx: mpsc::Sender<RoomMessage>, in_message_tx: mpsc::Sender<InMessage>) -> RoomContext {
        RoomContext {
            message_tx,
            in_message_tx,
        }
    }
}

pub enum InMessageHandleResult {
    Break,
    Continue,
    Pass,
}

pub enum RoomMessageHandleResult {
    Break,
    Continue,
    Pass,
}

pub type InMessageHandler = fn(&InMessage) -> InMessageHandleResult;
pub type RoomMessageHandler = fn(&RoomMessage) -> RoomMessageHandleResult;

pub struct RoomBuilder {
    pub in_message_handlers: Vec<InMessageHandler>,
    pub in_message_buffer_size: usize,

    pub room_message_handlers: Vec<RoomMessageHandler>,
    pub room_message_buffer_size: usize,

    pub update_interval: Option<time::Duration>,
}

impl RoomBuilder {
    pub fn new() -> Self {
        RoomBuilder {
            in_message_handlers: Vec::new(),
            in_message_buffer_size: 0,

            room_message_handlers: Vec::new(),
            room_message_buffer_size: 0,

            update_interval: None,
        }
    }

    pub fn add_in_message_handler(mut self, handler: InMessageHandler) -> Self {
        self.in_message_handlers.push(handler);
        self
    }

    pub fn set_in_message_buffer_size(mut self, size: usize) -> Self {
        self.in_message_buffer_size = size;
        self
    }

    pub fn add_room_message_handler(mut self, handler: RoomMessageHandler) -> Self {
        self.room_message_handlers.push(handler);
        self
    }

    pub fn set_room_message_buffer_size(mut self, size: usize) -> Self {
        self.room_message_buffer_size = size;
        self
    }

    pub fn set_update_interval(mut self, interval: time::Duration) -> Self {
        self.update_interval = Some(interval);
        self
    }
}

impl Default for RoomBuilder {
    fn default() -> Self {
        let mut builder = RoomBuilder::new();

        builder.in_message_buffer_size = 256;
        builder.room_message_buffer_size = 64;

        builder
    }
}

pub fn run_room(
    builder: RoomBuilder,
    server_ctx: Arc<ServerContext>,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> Arc<RoomContext> {
    let (in_message_tx, mut in_message_rx) = mpsc::channel(builder.in_message_buffer_size);
    if builder.in_message_handlers.is_empty() {
        panic!("InMessage handlers must not be empty!");
    }

    let (room_message_tx, mut room_message_rx) = mpsc::channel(builder.room_message_buffer_size);
    if builder.room_message_handlers.is_empty() {
        panic!("RoomMessage handlers must not be empty!");
    }

    let ctx = Arc::new(RoomContext::new(room_message_tx, in_message_tx));
    let ctx_return = ctx.clone();

    tokio::spawn(async move {
        let mut in_message_buffer = Vec::with_capacity(builder.in_message_buffer_size);
        let mut room_message_buffer = Vec::with_capacity(builder.room_message_buffer_size);

        let mut world = World::default();
        let update_enabled = builder.update_interval.is_some();
        let mut update_timer = time::interval(
            if update_enabled {
                builder.update_interval.unwrap()
            } else {
                time::Duration::from_secs(1)
            }
        );

        loop {
            tokio::select! {
                n = in_message_rx.recv_many(&mut in_message_buffer, builder.in_message_buffer_size) => {
                    if n == 0 {
                        break;
                    }

                    for in_message in in_message_buffer.drain(0..n) {
                        handle_in_message(
                            &in_message,
                            &builder.in_message_handlers,
                            &ctx,
                            &server_ctx,
                        ).await;
                    }
                },
                n = room_message_rx.recv_many(&mut room_message_buffer, builder.room_message_buffer_size) => {
                    if n == 0 {
                        break;
                    }

                    for room_message in room_message_buffer.drain(0..n) {
                        handle_room_message(
                            &room_message,
                            &builder.room_message_handlers,
                            &ctx,
                            &server_ctx,
                        ).await;
                    }
                },
                _ = update_timer.tick(), if update_enabled => {
                    update();
                }
                _ = shutdown_rx.recv() => break,
            }
        }
    });

    ctx_return
}

async fn handle_in_message(
    message: &InMessage,
    handlers: &Vec<InMessageHandler>,
    ctx: &Arc<RoomContext>,
    server_ctx: &Arc<ServerContext>,
) {
    let mut handled = false;
    for handler in handlers {
        match handler(&message) {
            InMessageHandleResult::Break => {
                handled = true;
                break;
            },
            InMessageHandleResult::Continue => {
                handled = true;
                continue;
            },
            InMessageHandleResult::Pass => {
                continue;
            }
        }
    }

    if !handled {
        todo!("Print unhandled message error");
    }
}

async fn handle_room_message(
    message: &RoomMessage,
    handlers: &Vec<RoomMessageHandler>,
    ctx: &Arc<RoomContext>,
    server_ctx: &Arc<ServerContext>,
) {

}

fn update() {

}
