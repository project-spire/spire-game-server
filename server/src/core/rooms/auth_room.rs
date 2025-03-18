use bytes::Bytes;
use crate::core::config::config;
use crate::core::room::RoomContext;
use crate::core::server::{ServerContext, ServerMessage};
use crate::core::session::{Account, Privilege, SessionContext};
use crate::protocol::*;
use crate::protocol::auth::*;
use jsonwebtoken::{Algorithm, Validation, DecodingKey, decode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    aid: String, // account_id
    cid: String, // character_id
    prv: String, // privilege
}

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
                        if protocol != ProtocolCategory::Auth {
                            eprintln!("Protocol not auth: {:?}", protocol);
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
    let protocol = AuthProtocol::decode(data);
    if let Err(e) = protocol {
        eprintln!("Failed to decode auth protocol: {}", e);
        _ = session_ctx.close_tx.send(());
        return;
    }

    match protocol.unwrap().protocol {
        Some(auth_protocol::Protocol::Login(login)) => {
            handle_login(&server_ctx, session_ctx, login).await;
        }
        None => {
            _ = session_ctx.close_tx.send(());
        }
    }
}

async fn handle_login(
    server_ctx: &Arc<ServerContext>,
    session_ctx: Arc<SessionContext>,
    login: Login,
) {
    let token_data = decode::<Claims>(
        &login.token,
        &DecodingKey::from_secret(config().auth_key.as_bytes()),
        &Validation::new(Algorithm::HS256),
    );
    if let Err(e) = token_data {
        eprintln!("Error validating token: {}", e);
        _ = session_ctx.close_tx.send(()).await;
        return;
    }

    let claims = token_data.unwrap().claims;
    let account_id = match claims.aid.parse() {
        Ok(id) => id,
        _ => {
            eprintln!("Invalid account id: {}", claims.aid);
            return;
        }
    };
    let character_id = match claims.cid.parse() {
        Ok(id) => id,
        _ => {
            eprintln!("Invalid character id: {}", claims.cid);
            return;
        }
    };
    let privilege = match claims.prv.as_str() {
        "None" => Privilege::None,
        "Manager" => Privilege::Manager,
        _ => {
            eprintln!("Invalid privilege: {}", claims.prv);
            return;
        }
    };

    _ = session_ctx.account.set(Account{ account_id, character_id, privilege });

    println!("Authenticated: {:?}", session_ctx.account());
    _ = server_ctx.message_tx.send(ServerMessage::SessionAuthenticated(session_ctx)).await;
}
