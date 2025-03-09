use crate::core::config::config;
use crate::core::resource::Resource;
use crate::core::role::Role;
use crate::core::room::RoomContext;
use crate::core::session::{InMessage, OutMessage, SessionContext, run_session};
use crate::rooms::auth_room;
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinSet;

pub enum ServerMessage {
    BroadcastPlayer(OutMessage),
    SessionAuthenticated(Arc<SessionContext>),
    SessionClosed(Arc<SessionContext>),
    RoomTransfer { session: Arc<SessionContext>, target: u64 },
}

pub struct ServerContext {
    pub message_tx: mpsc::Sender<ServerMessage>,
}

impl ServerContext {
    pub fn new(message_tx: mpsc::Sender<ServerMessage>) -> ServerContext {
        ServerContext { message_tx }
    }
}

pub async fn run_server() -> Result<(), Box<dyn Error>> {
    let (shutdown_tx, _) = broadcast::channel(1);
    let shutdown_rx_listen = shutdown_tx.subscribe();
    let shutdown_rx_handle = shutdown_tx.subscribe();
    let (message_tx, message_rx) = mpsc::channel(64);

    let ctx = Arc::new(ServerContext::new(message_tx));
    let ctx_listen = ctx.clone();
    let auth_room_ctx = auth_room::run(ctx.clone(), shutdown_tx.subscribe());
    
    let resource = Resource::new().await;

    let mut tasks = JoinSet::new();
    tasks.spawn(async move {
        listen_game(ctx_listen, auth_room_ctx, shutdown_rx_listen).await;
    });
    tasks.spawn(async move {
        handle(message_rx, shutdown_rx_handle).await;
    });

    while let Some(_) = tasks.join_next().await {}
    shutdown_tx.send(())?;

    Ok(())
}

async fn listen_game(
    ctx: Arc<ServerContext>,
    auth_room_ctx: Arc<RoomContext>,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    let listen_addr = SocketAddr::from(([0, 0, 0, 0], config().game_listen_port));
    let listener = TcpListener::bind(listen_addr).await.unwrap();
    println!("Server listening game at {}", listen_addr);

    loop {
        tokio::select! {
            result = listener.accept() => match result {
                Ok((socket, _)) => {
                    let in_message_tx = auth_room_ctx.in_message_tx.clone();
                    accept_game(ctx.clone(), socket, in_message_tx, shutdown_rx.resubscribe());
                },
                Err(e) => {
                    eprintln!("Error accepting game connection: {}", e);
                }
            },
            _ = shutdown_rx.recv() => break,
        }
    }
}

fn accept_game(
    ctx: Arc<ServerContext>,
    stream: TcpStream,
    in_message_tx: mpsc::Sender<InMessage>,
    shutdown_rx: broadcast::Receiver<()>,
) {
    if let Err(e) = stream.set_nodelay(true) {
        eprintln!("Error setting nodelay: {}", e);
        return;
    }

    tokio::spawn(async move {
        let session_ctx =
            run_session(stream, in_message_tx, shutdown_rx).await;
        _ = ctx.message_tx.send(ServerMessage::SessionClosed(session_ctx)).await;
    });
}

async fn handle(
    mut message_rx: mpsc::Receiver<ServerMessage>,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    let mut player_sessions = HashMap::new();
    let mut rooms = HashMap::new();
    let mut message_buffer = Vec::with_capacity(64);

    loop {
        tokio::select! {
            n = message_rx.recv_many(&mut message_buffer, 64) => {
                if n == 0 {
                    break;
                }

                for message in message_buffer.drain(0..n) {
                    handle_internal(&mut player_sessions, &mut rooms, message).await;
                }
            },
            _ = shutdown_rx.recv() => break,
        }
    }
}

async fn handle_internal(
    player_sessions: &mut HashMap<u64, Arc<SessionContext>>,
    rooms: &mut HashMap<u64, Arc<RoomContext>>,
    message: ServerMessage
) {
    match message {
        ServerMessage::BroadcastPlayer(message) => handle_broadcast_player(&player_sessions, message).await,
        ServerMessage::SessionAuthenticated(session) => handle_session_authenticated(player_sessions, session),
        ServerMessage::SessionClosed(session) => handle_session_closed(player_sessions, session),
        ServerMessage::RoomTransfer{session, target} => handle_room_transfer(&rooms, session, target).await,
    }
}

async fn handle_broadcast_player(
    player_sessions: &HashMap<u64, Arc<SessionContext>>,
    message: OutMessage
) {
    for (_, session) in player_sessions {
        _ = session.out_message_tx.send(message.clone()).await;
    }
}

fn handle_session_authenticated(
    player_sessions: &mut HashMap<u64, Arc<SessionContext>>,
    session: Arc<SessionContext>
) {
    //TODO: Handle exceptions that session is already closed

    match session.role.get() {
        Some(Role::Player {account_id, character_id}) => {
            player_sessions.insert(*character_id, session);
        },
        Some(Role::CheatPlayer {account_id, character_id}) => {
            player_sessions.insert(*character_id, session);
        },
        _ => {}
    }
    
    println!("Players count on authenticated: {}", player_sessions.len());
}

fn handle_session_closed(
    player_sessions: &mut HashMap<u64, Arc<SessionContext>>,
    session: Arc<SessionContext>
) {
    match session.role.get() {
        Some(Role::Player {account_id, character_id}) => {
            player_sessions.remove(character_id);
        },
        Some(Role::CheatPlayer {account_id, character_id}) => {
            player_sessions.remove(character_id);
        },
        _ => {}
    }

    println!("Players count on closed: {}", player_sessions.len());
}

async fn handle_room_transfer(
    rooms: &HashMap<u64, Arc<RoomContext>>,
    session: Arc<SessionContext>,
    target: u64,
) {
    if !rooms.contains_key(&target) {
        eprintln!("Invalid room transfer: {}, target={}", "TODO", target);
        return;
    }

    {
        let mut in_message_tx = session.in_message_tx.write().await;
        *in_message_tx = rooms.get(&target).unwrap().in_message_tx.clone();
    }
}
