use crate::core::room::RoomContext;
use crate::core::session::SessionContext;
use crate::protocol::auth::{auth_protocol::Protocol, AuthProtocol, Login, Role};
use bytes::Bytes;
use prost::Message;
use std::sync::Arc;
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

fn handle(ctx: Arc<SessionContext>, data: Bytes) {
    let protocol = AuthProtocol::decode(data);
    if let Err(e) = protocol {
        eprintln!("Failed to decode auth protocol: {}", e);
        _ = ctx.close_tx.send(());
        return;
    }

    match protocol.unwrap().protocol {
        Some(Protocol::Login(login)) => { handle_login(ctx, login); }
        None => { _ = ctx.close_tx.send(()); }
    }
}

fn handle_login(ctx: Arc<SessionContext>, login: Login) {
    match login.role {
        r if r == Role::Player as i32 => {}
        r if r == Role::CheatPlayer as i32 => {}
        r if r == Role::Admin as i32 => {}
        _ => { _ = ctx.close_tx.send(()); }
    }
}