use crate::core::room::RoomContext;
use crate::core::session::SessionContext;
use crate::protocol::auth::{auth_protocol::Protocol, AuthProtocol, Login};
use bytes::Bytes;
use prost::Message;
use tokio::sync::{broadcast, mpsc};

pub fn run(mut shutdown_rx: broadcast::Receiver<()>) -> RoomContext {
    let (in_message_tx, mut in_message_rx) = mpsc::channel(256);

    let ctx = RoomContext::new(in_message_tx);

    tokio::spawn(async move {
        loop {
            tokio::select! {
                result = in_message_rx.recv() => match result {
                    Some((session_ctx, data)) => {
                        handle(session_ctx, data);
                    }
                    None => { break; }
                },
                _ = shutdown_rx.recv() => { break; },
            }
        }
    });

    ctx
}

fn handle(ctx: SessionContext, data: Bytes) {
    let protocol = AuthProtocol::decode(data);
    if let Err(e) = protocol {
        eprintln!("Failed to decode auth protocol: {}", e);
        //TODO: Stop session
        return;
    }

    match protocol.unwrap().protocol {
        Some(Protocol::Login(login)) => { handle_login(ctx, login); }
        None => {}
    }
}

fn handle_login(ctx: SessionContext, login: Login) {
}