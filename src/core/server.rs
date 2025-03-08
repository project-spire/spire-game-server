use crate::core::role::Role;
use crate::core::room::RoomContext;
use crate::core::session::{run_session, InMessageTx};
use crate::rooms::auth_room;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio::task::JoinSet;

pub struct ServerContext {}
unsafe impl Send for ServerContext {}
unsafe impl Sync for ServerContext {}

impl ServerContext {
    pub fn new() -> ServerContext {
        ServerContext {}
    }
}

pub async fn run_server(listen_port: u16) -> Result<(), Box<dyn Error>> {
    let (shutdown_tx, _) = broadcast::channel(1);

    let mut tasks = JoinSet::new();

    let auth_room_ctx = auth_room::run(shutdown_tx.subscribe());
    let shutdown_rx_listen = shutdown_tx.subscribe();
    tasks.spawn(async move {
        listen(listen_port, auth_room_ctx, shutdown_rx_listen).await;
    });

    while let Some(_) = tasks.join_next().await {}
    shutdown_tx.send(())?;

    Ok(())
}

async fn listen(
    listen_port: u16,
    auth_room_ctx: RoomContext,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    let listen_addr = SocketAddr::from(([0, 0, 0, 0], listen_port));
    let listener = TcpListener::bind(listen_addr).await.unwrap();
    println!("Server listening at {}", listen_addr);

    loop {
        tokio::select! {
            result = listener.accept() => match result {
                Ok((socket, _)) => {
                    let in_message_tx = auth_room_ctx.in_message_tx.clone();
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
    if let Err(e) = stream.set_nodelay(true) {
        eprintln!("Error setting nodelay: {}", e);
        return;
    }

    tokio::spawn(async move {
        _ = run_session(stream, in_message_tx, shutdown_rx).await
    });
}
