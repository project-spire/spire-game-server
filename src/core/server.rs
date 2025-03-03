use crate::core::role::CharacterRole;
use crate::core::room::RoomContext;
use crate::core::session::{run_session, InMessageTx};
use crate::rooms::waiting_room;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinSet;

pub struct ServerContext {
    waiting_room_ctx: RoomContext,
}
unsafe impl Send for ServerContext {}
unsafe impl Sync for ServerContext {}

impl ServerContext {
    pub fn new(waiting_room_ctx: RoomContext) -> ServerContext {
        ServerContext { waiting_room_ctx }
    }
}

pub async fn run_server(listen_port: u16) -> Result<(), Box<dyn Error>> {
    let (shutdown_tx, _) = broadcast::channel(1);
    let waiting_room_tx = waiting_room::run(shutdown_tx.subscribe());
    
    let mut tasks = JoinSet::new();
    
    let shutdown_rx_listen = shutdown_tx.subscribe();
    let ctx = ServerContext::new(waiting_room_tx.clone());
    tasks.spawn(async move {
        listen(listen_port, ctx, shutdown_rx_listen).await;
    });

    while let Some(_) = tasks.join_next().await {}
    shutdown_tx.send(())?;
    
    Ok(())
}

async fn listen(
    listen_port: u16,
    ctx: ServerContext,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    let listen_addr = SocketAddr::from(([0, 0, 0, 0], listen_port));
    let listener = TcpListener::bind(listen_addr).await.unwrap();
    println!("Server listening at {}", listen_addr);

    loop {
        tokio::select! {
            result = listener.accept() => match result {
                Ok((socket, _)) => {
                    let in_message_tx = ctx.waiting_room_ctx.in_message_tx.clone();
                    accept(socket, in_message_tx, shutdown_rx.resubscribe());
                },
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                }
            },
            _ = shutdown_rx.recv() => {
                println!("Server shutting down...");
                break;
            }
        }
    }
}

fn accept(
    stream: TcpStream,
    in_message_tx: InMessageTx,
    shutdown_rx: broadcast::Receiver<()>,
) {
    let role = Arc::new(CharacterRole::new());

    tokio::spawn(async move {
        _ = run_session(stream, in_message_tx, role, shutdown_rx).await
    });
}
