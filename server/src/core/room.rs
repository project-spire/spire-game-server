use crate::core::session::InMessage;
use tokio::sync::mpsc;

pub enum RoomMessage {
    
}

pub struct RoomContext {
    pub in_message_tx: mpsc::Sender<InMessage>,
}

impl RoomContext {
    pub fn new(in_message_tx: mpsc::Sender<InMessage>) -> RoomContext {
        RoomContext { in_message_tx }
    }
}
