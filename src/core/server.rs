use crate::core::room::Room;
use crate::core::session::run_session;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc};

pub struct Server {
    shutdown_tx: broadcast::Sender<()>,
}

impl Server {
    pub fn new() -> Self {
        let (shutdown_tx, shutdown_rx) = broadcast::channel(1);

        Server { shutdown_tx }
    }

    pub async fn run(&mut self, listen_port: u16) -> Result<(), Box<dyn Error>> {
        let listen_addr = SocketAddr::from(([0, 0, 0, 0], listen_port));
        let listener = TcpListener::bind(listen_addr).await?;
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        println!("Listening on {}", listener.local_addr()?);

        let mut waiting_room = Room::new(0, self.shutdown_tx.subscribe());
        let in_message_tx = waiting_room.in_message_tx.clone();

        tokio::spawn(async move {
            waiting_room.run().await;
        });

        loop {
            tokio::select! {
                result = listener.accept() => match result {
                    Ok((socket, _)) => {
                        self.on_accept(socket, in_message_tx.clone());
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

        Ok(())
    }

    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
    }

    fn on_accept(&self, stream: TcpStream, in_message_tx: mpsc::Sender<Vec<u8>>) {
        let shutdown_rx = self.shutdown_tx.subscribe();

        tokio::spawn(async move {
            let (transfer_tx, transfer_rx) = broadcast::channel(1);
            _ = run_session(stream, in_message_tx, transfer_rx, shutdown_rx).await
        });
    }
}
