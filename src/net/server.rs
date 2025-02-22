use std::error::Error;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;

struct Server {
    listener: TcpListener,

    shutdown_tx: broadcast::Sender<()>
}

impl Server {
    pub async fn new(addr: &str) -> Result<Self, Box<dyn Error>> {
        let listener = TcpListener::bind(addr).await?;
        let (shutdown_tx, _) = broadcast::channel(1);

        Ok(Server {
            listener,
            shutdown_tx
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        println!("Running server on {}", self.listener.local_addr()?);

        let mut shutdown_rx = self.shutdown_tx.subscribe();

        loop {
            tokio::select! {
                result = self.listener.accept() => self.on_accept(result)?,
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

    async fn on_accept(&self, result: tokio::io::Result<(TcpStream, SocketAddr)>) -> Result<(), Box<dyn Error>> {
        match result {
            Ok((socket, addr)) => {
                println!("New connection from {}", addr);

                let mut client = Client::new(socket, self.shutdown_tx.subscribe());
                tokio::spawn(async move {
                    if let Err(e) = client.run().await {
                        eprintln!("Error handling client {}: {}", client, e);
                    }
                })
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}