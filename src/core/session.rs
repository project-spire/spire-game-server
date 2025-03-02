use std::error::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinSet;

enum In {
    Message(Vec<u8>),
    Disconnected,
}

enum Out {
    Message(Vec<u8>),
    Disconnected,
}

pub async fn run_session(
    stream: TcpStream,
    server_shutdown_rx: broadcast::Receiver<()>,
) -> Result<(), Box<dyn Error>> {
    let peer_addr = stream.peer_addr()?;
    println!("Session {} has started", peer_addr);

    let (reader, writer) = tokio::io::split(stream);
    let (out_message_tx, out_message_rx) = mpsc::channel::<String>(32);

    let server_shutdown_rx_recv = server_shutdown_rx.resubscribe();
    let server_shutdown_rx_send = server_shutdown_rx.resubscribe();

    let mut tasks = JoinSet::new();

    let recv_handle = tasks.spawn(async move {
        recv(reader, server_shutdown_rx_recv).await;
    });

    let send_handle = tasks.spawn(async move {
        send(writer, out_message_rx, server_shutdown_rx_send).await;
    });

    while let Some(_) = tasks.join_next().await {
        // Returning from any of recv/send means that session has ended or errored.
        // So abort them.
        recv_handle.abort();
        send_handle.abort();
    }

    println!("Session {} has ended", peer_addr);
    Ok(())
}

async fn recv(mut reader: ReadHalf<TcpStream>, mut shutdown_rx: broadcast::Receiver<()>) {
    loop {
        tokio::select! {
            result = recv_internal(&mut reader) => {
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
}

async fn recv_internal(reader: &mut ReadHalf<TcpStream>) -> Result<In, Box<dyn Error>> {
    let mut header_buf = [0u8; 4];

    let n = reader.read_exact(&mut header_buf).await?;
    if n == 0 {
        return Ok(In::Disconnected);
    }

    let body_len = u32::from_ne_bytes(header_buf) as usize;
    let mut body_buf = vec![0u8; body_len];
    let n = reader.read_exact(&mut body_buf).await?;
    if n == 0 {
        return Ok(In::Disconnected);
    }

    Ok(In::Message(body_buf))
}

async fn send(
    mut writer: WriteHalf<TcpStream>,
    mut out_message_rx: mpsc::Receiver<String>,
    mut server_shutdown_rx: broadcast::Receiver<()>,
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
                    println!("WHY?");
                    break;
                }
            },
            _ = server_shutdown_rx.recv() => {
                break;
            }
        }
    }
}