use std::error::Error;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::sync::broadcast;

struct Client {
    socket: TcpStream,
    shutdown_rx: broadcast::Receiver<()>,
}

impl Client {
    pub fn new(socket: TcpStream, shutdown_rx: broadcast::Receiver<()> ) -> Self {
        Client {
            socket,
            shutdown_rx,
        }
    }

    pub async fn run(&mut self) {
        let mut buf = [0; 1024];

        loop {
            tokio::select! {
                result = self.recv() => {
                    if let Err(e) = result {
                        eprintln!("Error receiving from client {}: {}", self, e);
                        break;
                    }
                }
                _ = self.shutdown_rx.recv() => {
                    break;
                }
            }
        }
    }

    pub fn send(&mut self, data: &[u8]) -> Result<(), Box<dyn Error>> {

    }

    async fn recv(&mut self) -> Result<(), Box<dyn Error>> {
        let mut header_buf = [0u8; 4];

        let n = self.socket.read_exact(&mut header_buf).await?;
        if n == 0 {
            // Client disconnected
            return Ok(());
        }

        let body_len = u32::from_ne_bytes(header_buf) as usize;
        let mut body_buf = vec![0u8; body_len];
        let n = self.socket.read_exact(&mut body_buf).await?;
        if n == 0 {
            return Ok(());
        }

        Ok(())
    }
}