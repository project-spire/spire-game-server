use crate::core::session::InMessageTx;

pub struct RoomContext {
    pub in_message_tx: InMessageTx
}
unsafe impl Send for RoomContext {}
unsafe impl Sync for RoomContext {}

impl RoomContext {
    pub fn new(in_message_tx: InMessageTx) -> RoomContext {
        RoomContext { in_message_tx }
    }
}

impl Clone for RoomContext {
    fn clone(&self) -> RoomContext {
        RoomContext {
            in_message_tx: self.in_message_tx.clone()
        }
    }
}