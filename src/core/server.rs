use crate::core::room::Room;
use crate::core::session::run_session;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinSet;

pub async fn run_server(listen_port: u16) -> Result<(), Box<dyn Error>> {
    let (_shutdown_tx, shutdown_rx) = broadcast::channel(1);

    let mut waiting_room = Room::new(0, shutdown_rx.resubscribe());
    let in_message_tx = waiting_room.in_message_tx.clone();

    let mut tasks = JoinSet::new();
    tasks.spawn(async move {
        listen(listen_port, in_message_tx, shutdown_rx).await;
    });
    tasks.spawn(async move {
        waiting_room.run().await;
    });

    while let Some(_) = tasks.join_next().await {}
    Ok(())
}

async fn listen(
    listen_port: u16,
    in_message_tx: mpsc::Sender<Vec<u8>>,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    let listen_addr = SocketAddr::from(([0, 0, 0, 0], listen_port));
    let listener = TcpListener::bind(listen_addr).await.unwrap();
    println!("Server listening at {}", listen_addr);

    loop {
        tokio::select! {
            result = listener.accept() => match result {
                Ok((socket, _)) => {
                    accept(socket, in_message_tx.clone(), shutdown_rx.resubscribe());
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
    in_message_tx: mpsc::Sender<Vec<u8>>,
    shutdown_rx: broadcast::Receiver<()>,
) {
    tokio::spawn(async move {
        let (transfer_tx, transfer_rx) = broadcast::channel(1);
        _ = run_session(stream, in_message_tx, transfer_rx, shutdown_rx).await
    });
}
