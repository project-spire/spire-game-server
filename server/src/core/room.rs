use crate::core::session::{InMessage, OutMessage};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use crate::core::server::ServerContext;

pub enum RoomMessage {
    Broadcast(OutMessage),
    RoomTransferBegin {}
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

pub async fn handle_room_message(ctx: &Arc<RoomContext>, message: RoomMessage) {

}