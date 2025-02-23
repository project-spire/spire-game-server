use std::error::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinSet;

pub struct Session {
    pub out_message_tx: mpsc::Sender<String>,
}

impl Session {
    pub fn new() -> (Session, mpsc::Receiver<String>) {
        let (out_message_tx, out_message_rx) = mpsc::channel::<String>(32);

        (Session { out_message_tx }, out_message_rx)
    }

    pub async fn run(
        &mut self,
        stream: TcpStream,
        out_message_rx: mpsc::Receiver<String>,
        shutdown_rx: broadcast::Receiver<()>,
    ) -> Result<(), Box<dyn Error>> {
        let (reader, writer) = tokio::io::split(stream);
        let shutdown_rx_recv = shutdown_rx.resubscribe();
        let shutdown_rx_send = shutdown_rx.resubscribe();

        let mut tasks = JoinSet::new();

        tasks.spawn(async move {
            Self::recv(reader, shutdown_rx_recv).await;
        });

        tasks.spawn(async move {
            Self::send_internal(writer, out_message_rx, shutdown_rx_send).await;
        });

        while let Some(result) = tasks.join_next().await {
            if let Err(e) = result {
                eprintln!("Client task error: {}", e);
            }
        }

        Ok(())
    }

    async fn recv(mut reader: ReadHalf<TcpStream>, mut shutdown_rx: broadcast::Receiver<()>) {
        loop {
            tokio::select! {
                result = Self::recv_internal(&mut reader) => {
                    if let Err(e) = result {
                        eprintln!("Error receiving: {}", e);
                        break;
                    }
                }
                _ = shutdown_rx.recv() => {
                    break;
                }
            }
        }

        //TODO: Stop session
    }

    async fn recv_internal(reader: &mut ReadHalf<TcpStream>) -> Result<(), Box<dyn Error>> {
        let mut header_buf = [0u8; 4];

        let n = reader.read_exact(&mut header_buf).await?;
        if n == 0 {
            // Client disconnected
            return Ok(());
        }

        let body_len = u32::from_ne_bytes(header_buf) as usize;
        let mut body_buf = vec![0u8; body_len];
        let n = reader.read_exact(&mut body_buf).await?;
        if n == 0 {
            return Ok(());
        }

        Ok(())
    }

    async fn send_internal(
        mut writer: WriteHalf<TcpStream>,
        mut out_message_rx: mpsc::Receiver<String>,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) {
        async fn write(writer: &mut WriteHalf<TcpStream>, message: Option<String>) -> bool {
            if let None = message {
                return false;
            }

            let message = message.unwrap();
            if let Err(e) = writer.write_all(message.as_bytes()).await {
                eprintln!("Error sending message: {}", e);
                return false;
            }

            true
        }

        loop {
            tokio::select! {
                message = out_message_rx.recv() =>  {
                    if !write(&mut writer, message).await {
                        break;
                    }
                },
                _ = shutdown_rx.recv() => {
                    break;
                }
            }
        }
    }
}
