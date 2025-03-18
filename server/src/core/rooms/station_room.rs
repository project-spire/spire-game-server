use bytes::Bytes;
use crate::core::room::RoomContext;
use crate::core::server::ServerContext;
use crate::core::session::SessionContext;
use crate::protocol::*;
use crate::protocol::net::{
    NetProtocol,
    net_protocol::Protocol::*,
};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};

pub fn run(
    server_ctx: Arc<ServerContext>,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> Arc<RoomContext> {
    let (in_message_tx, mut in_message_rx) = mpsc::channel(64);

    let ctx = Arc::new(RoomContext::new(in_message_tx));

    tokio::spawn(async move {
        let mut message_buffer = Vec::with_capacity(64);

        loop {
            tokio::select! {
                n = in_message_rx.recv_many(&mut message_buffer, 64) => {
                    if n == 0 {
                        break;
                    }

                    for (session_ctx, protocol, data) in message_buffer.drain(0..n) {
                        if protocol != ProtocolCategory::Net {
                            eprintln!("Protocol not net: {:?}", protocol);
                            continue;
                        }

                        handle(&server_ctx, session_ctx, data).await;
                    }
                },
                _ = shutdown_rx.recv() => break,
            }
        }
    });

    ctx
}

async fn handle(
    server_ctx: &Arc<ServerContext>,
    session_ctx: Arc<SessionContext>,
    data: Bytes,
) {
    let protocol = NetProtocol::decode(data);
    if let Err(e) = protocol {
        eprintln!("Failed to decode net protocol: {}", e);
        _ = session_ctx.close_tx.send(());
        return;
    }

    match protocol.unwrap().protocol {
        Some(RoomTransferReady(ready)) => {
            handle_room_transfer_ready(&server_ctx, session_ctx, ready).await;
        }
        None => {
            _ = session_ctx.close_tx.send(());
        }
    }
}

async fn handle_room_transfer_ready(
    server_ctx: &Arc<ServerContext>,
    session_ctx: Arc<SessionContext>,
    ready: RoomTransferReady,
) {

}