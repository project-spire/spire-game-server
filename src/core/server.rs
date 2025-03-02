use crate::core::session::run_session;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;

pub struct Server {
    shutdown_tx: broadcast::Sender<()>,
}

impl Server {
    pub fn new() -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);

        Server { shutdown_tx }
    }

    pub async fn run(&mut self, listen_port: u16) -> Result<(), Box<dyn Error>> {
        let listen_addr = SocketAddr::from(([0, 0, 0, 0], listen_port));
        let listener = TcpListener::bind(listen_addr).await?;
        println!("Listening on {}", listener.local_addr()?);

        let mut shutdown_rx = self.shutdown_tx.subscribe();

        loop {
            tokio::select! {
                result = listener.accept() => match result {
                    Ok((socket, addr)) => {
                        self.on_accept(socket);
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

    fn on_accept(&self, stream: TcpStream) {
        let shutdown_rx = self.shutdown_tx.subscribe();

        tokio::spawn(async move {
            _ = run_session(stream, shutdown_rx).await
        });
    }
}
