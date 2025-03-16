use crate::core::config::config;
use crate::core::role::Role;
use crate::core::room::RoomContext;
use crate::core::server::{ServerContext, ServerMessage};
use crate::core::session::SessionContext;
use protocol::{
    Protocol,
    auth::{AuthProtocol, Login, LoginRole, auth_protocol}
};
use bytes::Bytes;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use prost::Message;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    aid: String, // account_id
    cid: String, // character_id
    role: String,
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
                        if protocol != Protocol::Auth {
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
    let header = Header::new(Algorithm::HS256);
    let claims = Claims {
        aid: login.account_id.to_string(),
        cid: login.character_id.to_string(),
        role: String::from(match login.role {
            r if r == LoginRole::Player as i32 => {
                _ = session_ctx.role.set(Role::Player {
                    account_id: login.account_id,
                    character_id: login.character_id,
                });
                "Player"
            }
            r if r == LoginRole::CheatPlayer as i32 => {
                _ = session_ctx.role.set(Role::Player {
                    account_id: login.account_id,
                    character_id: login.character_id,
                });
                "CheatPlayer"
            }
            r if r == LoginRole::Admin as i32 => {
                _ = session_ctx.role.set(Role::Admin);
                "Admin"
            }
            _ => {
                eprintln!("Invalid role string: {}", login.role);
                _ = session_ctx.close_tx.send(()).await;
                return;
            }
        }),
    };

    if let Err(e) = encode(
        &header,
        &claims,
        &EncodingKey::from_secret(config().auth_key.as_bytes()),
    ) {
        eprintln!("Error validating token: {}", e);
        _ = session_ctx.close_tx.send(()).await;
        return;
    }

    println!("Authenticated: {}", session_ctx.role.get().unwrap());
    
    tokio::spawn(async move {
        //TODO: 
        
        _ = server_ctx.message_tx.send(ServerMessage::SessionAuthenticated(session_ctx)).await;
    });
}
