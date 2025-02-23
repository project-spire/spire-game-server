use crate::core::session::Session;
use std::error::Error;
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

    pub async fn run(&mut self, addr: &str) -> Result<(), Box<dyn Error>> {
        let listener = TcpListener::bind(addr).await?;
        println!("Listening on {}", listener.local_addr()?);

        let mut shutdown_rx = self.shutdown_tx.subscribe();

        loop {
            tokio::select! {
                result = listener.accept() => match result {
                    Ok((socket, addr)) => {
                        println!("Accepted connection from {}", addr);
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
        let (mut session, out_message_rx) = Session::new();

        tokio::spawn(async move {
            if let Err(e) = session
                .run(stream, out_message_rx, self.shutdown_tx.subscribe())
                .await
            {
                eprintln!("Error client running: {}", e);
            }
        });
    }
}
