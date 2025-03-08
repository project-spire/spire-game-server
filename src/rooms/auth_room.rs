use crate::core::config::config;
use crate::core::room::RoomContext;
use crate::core::session::SessionContext;
use crate::protocol::auth::{auth_protocol::Protocol, AuthProtocol, Login, Role};
use bytes::Bytes;
use jsonwebtoken::{encode, Header, Algorithm, EncodingKey};
use prost::Message;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use tokio::sync::{broadcast, mpsc};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    aid: String, // account_id
    cid: String, // character_id
    role: String,
}

pub fn run(mut shutdown_rx: broadcast::Receiver<()>) -> RoomContext {
    let (in_message_tx, mut in_message_rx) = mpsc::channel(256);

    let ctx = RoomContext::new(in_message_tx);

    tokio::spawn(async move {
        loop {
            tokio::select! {
                result = in_message_rx.recv() => match result {
                    Some((session_ctx, data)) => {
                        handle(session_ctx, data).await;
                    }
                    None => { break; }
                },
                _ = shutdown_rx.recv() => { break; },
            }
        }
    });

    ctx
}

async fn handle(ctx: Arc<SessionContext>, data: Bytes) {
    let protocol = AuthProtocol::decode(data);
    if let Err(e) = protocol {
        eprintln!("Failed to decode auth protocol: {}", e);
        _ = ctx.close_tx.send(());
        return;
    }

    match protocol.unwrap().protocol {
        Some(Protocol::Login(login)) => { handle_login(ctx, login).await; }
        None => { _ = ctx.close_tx.send(()); }
    }
}

async fn handle_login(ctx: Arc<SessionContext>, login: Login) {
    let header = Header::new(Algorithm::HS256);
    let claims = Claims {
        aid: login.account_id.to_string(),
        cid: login.character_id.to_string(),
        role: String::from(match login.role {
            r if r == Role::Player as i32 => "Player",
            r if r == Role::CheatPlayer as i32 => "CheatPlayer",
            r if r == Role::Admin as i32 => "Admin",
            _ => {
                _ = ctx.close_tx.send(()).await;
                return;
            }
        }),
    };

    if let Err(e) = encode(&header, &claims, &EncodingKey::from_secret(config().auth_key.as_bytes())) {
        eprintln!("Error validating token: {}", e);
        return;
    }
    
    println!("Authenticated!");
}