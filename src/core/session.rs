use std::error::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinSet;

enum In {
    Message(Vec<u8>),
    EOF,
}

pub async fn run_session(
    stream: TcpStream,
    in_message_tx: mpsc::Sender<Vec<u8>>,
    transfer_rx: broadcast::Receiver<mpsc::Sender<Vec<u8>>>,
    shutdown_rx: broadcast::Receiver<()>,
) -> Result<(), Box<dyn Error>> {
    let peer_addr = stream.peer_addr()?;
    let (reader, writer) = tokio::io::split(stream);
    let (out_message_tx, out_message_rx) = mpsc::channel::<String>(32);
    let shutdown_rx_recv = shutdown_rx.resubscribe();
    let shutdown_rx_send = shutdown_rx.resubscribe();

    println!("Session {} has started", peer_addr);

    let mut tasks = JoinSet::new();
    let recv_handle = tasks.spawn(async move {
        recv(reader, in_message_tx, transfer_rx, shutdown_rx_recv).await;
    });
    let send_handle = tasks.spawn(async move {
        send(writer, out_message_rx, shutdown_rx_send).await;
    });

    while let Some(_) = tasks.join_next().await {
        // Returning from any of recv/send task means that the session has ended or errored.
        // So abort the tasks.
        recv_handle.abort();
        send_handle.abort();
    }

    println!("Session {} has ended", peer_addr);
    Ok(())
}

async fn recv(
    mut reader: ReadHalf<TcpStream>,
    mut in_message_tx: mpsc::Sender<Vec<u8>>,
    mut transfer_rx: broadcast::Receiver<mpsc::Sender<Vec<u8>>>,
    mut shutdown_rx: broadcast::Receiver<()>) {
    loop {
        tokio::select! {
            result = transfer_rx.recv() => match result {
                Ok(tx) => {
                    in_message_tx = tx;
                }
                Err(e) => {
                    eprintln!("Error receiving receiver: {}", e);
                    break;
                }
            },
            result = recv_internal(&mut reader) => match result {
                Ok(i) => match i {
                    In::Message(buf) => {
                        _ = in_message_tx.send(buf).await;
                    }
                    In::EOF => {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Error receiving message: {}", e);
                    break;
                }
            },
            _ = shutdown_rx.recv() => {
                break;
            }
        }
    }
}

async fn recv_internal(reader: &mut ReadHalf<TcpStream>) -> Result<In, Box<dyn Error + Send + Sync>> {
    let mut header_buf = [0u8; 4];
    let n = reader.read_exact(&mut header_buf).await?;
    if n == 0 {
        return Ok(In::EOF);
    }

    let body_len = u32::from_ne_bytes(header_buf) as usize;
    let mut body_buf = vec![0u8; body_len];

    let n = reader.read_exact(&mut body_buf).await?;
    if n == 0 {
        return Ok(In::EOF);
    }

    Ok(In::Message(body_buf))
}

async fn send(
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
            message = out_message_rx.recv() => {
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