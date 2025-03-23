use crate::character::player::{Account, PlayerBundle};
use crate::core::config::config;
use crate::core::resource::Resource;
use crate::core::room::{RoomContext, RoomMessage};
use crate::core::rooms::{auth_room, station_room};
use crate::core::session::{InMessage, OutMessage, SessionContext, run_session, Session};
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinSet;

pub enum ServerMessage {
    Broadcast(OutMessage),
    SessionAuthenticated(Arc<SessionContext>, Account),
    SessionClosed(Arc<SessionContext>),
    RoomTransferBegin { player: Arc<PlayerBundle>, target: u64 },
    RoomTransferCommit { player: Arc<PlayerBundle>, target: u64 },
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
    let ctx_handle = ctx.clone();
    let auth_room_ctx = auth_room::run(ctx.clone(), shutdown_tx.subscribe());
    let station_room_ctx = station_room::run(ctx.clone(), shutdown_tx.subscribe());
    
    let resource = Resource::new().await;

    let mut tasks = JoinSet::new();
    tasks.spawn(async move {
        listen(ctx_listen, auth_room_ctx, shutdown_rx_listen).await;
    });
    tasks.spawn(async move {
        handle(ctx_handle, message_rx, shutdown_rx_handle).await;
    });

    while let Some(_) = tasks.join_next().await {}
    shutdown_tx.send(())?;

    Ok(())
}

async fn listen(
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
                    accept(ctx.clone(), socket, in_message_tx, shutdown_rx.resubscribe());
                },
                Err(e) => {
                    eprintln!("Error accepting game connection: {}", e);
                }
            },
            _ = shutdown_rx.recv() => break,
        }
    }
}

fn accept(
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
        let session_ctx = run_session(stream, in_message_tx, shutdown_rx).await;
        _ = ctx.message_tx.send(ServerMessage::SessionClosed(session_ctx)).await;
    });
}

async fn handle(
    ctx: Arc<ServerContext>,
    mut message_rx: mpsc::Receiver<ServerMessage>,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    let mut rooms = HashMap::new();
    let mut message_buffer = Vec::with_capacity(64);

    loop {
        tokio::select! {
            n = message_rx.recv_many(&mut message_buffer, 64) => {
                if n == 0 {
                    break;
                }

                for message in message_buffer.drain(0..n) {
                    handle_internal(message, &ctx, &mut rooms).await;
                }
            },
            _ = shutdown_rx.recv() => break,
        }
    }
}

async fn handle_internal(
    message: ServerMessage,
    ctx: &Arc<ServerContext>,
    rooms: &mut HashMap<u64, Arc<RoomContext>>,
) {
    match message {
        ServerMessage::Broadcast(message) => handle_broadcast(rooms, message).await,
        ServerMessage::SessionAuthenticated(session, account) => handle_session_authenticated(ctx, session, account).await,
        ServerMessage::SessionClosed(session) => handle_session_closed(session).await,
        ServerMessage::RoomTransferBegin { player, target} => handle_room_transfer_begin(&rooms, player, target).await,
        ServerMessage::RoomTransferCommit { player, target} => {},
    }
}

async fn handle_broadcast(
    rooms: &mut HashMap<u64, Arc<RoomContext>>,
    message: OutMessage
) {
    for room in rooms.values() {
        _ = room.message_tx.send(RoomMessage::Broadcast(message.clone())).await;
    }
}

async fn handle_session_authenticated(
    ctx: &Arc<ServerContext>,
    session_ctx: Arc<SessionContext>,
    account: Account
) {
    if !session_ctx.is_open().await {
        return
    }

    let server_ctx = ctx.clone();

    tokio::spawn(async move {
        let session = Session::new(session_ctx);
        let (player, last_room) = PlayerBundle::load(account, session).await;

        _ = server_ctx.message_tx.send(
            ServerMessage::RoomTransferBegin {player, target: last_room});
    });
}

async fn handle_session_closed(session: Arc<SessionContext>) {


    //TODO: Remove from the current room
}

async fn handle_room_transfer_begin(
    rooms: &HashMap<u64, Arc<RoomContext>>,
    player_bundle: Arc<PlayerBundle>,
    target: u64,
) {
    if !player_bundle.session.ctx.is_open().await {
        return
    }

    if !rooms.contains_key(&target) {
        eprintln!("Invalid room transfer: {}, target={}", player_bundle.session.ctx, target);
        return;
    }

    {
        let mut in_message_tx = player_bundle.session.ctx.in_message_tx.write().await;
        *in_message_tx = rooms.get(&target).unwrap().in_message_tx.clone();
    }
}
