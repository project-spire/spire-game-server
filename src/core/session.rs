use crate::core::role::Role;
use bytes::{Bytes, BytesMut};
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinSet;

pub type InMessage = (SessionContext, Bytes);
pub type InMessageTx = mpsc::Sender<InMessage>;
pub type InMessageRx = mpsc::Receiver<InMessage>;

pub struct SessionContext {
    pub role: Arc<dyn Role>,
    pub send_tx: mpsc::Sender<Bytes>,
    pub transfer_tx: broadcast::Sender<InMessageTx>,
}

impl SessionContext {
    pub fn new(
        role: Arc<dyn Role>,
        send_tx: mpsc::Sender<Bytes>,
        transfer_tx: broadcast::Sender<InMessageTx>,
    ) -> SessionContext {
        SessionContext { role, send_tx, transfer_tx }
    }
}

enum Recv {
    Buf(Bytes),
    EOF,
}

pub async fn run_session(
    stream: TcpStream,
    recv_tx: InMessageTx,
    role: Arc<dyn Role>,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    let peer_addr = stream.peer_addr().unwrap_or(SocketAddr::from(([0, 0, 0, 0], 0)));
    let (reader, writer) = tokio::io::split(stream);
    let (send_tx, send_rx) = mpsc::channel(64);

    println!("Session {} has started", peer_addr);

    let mut tasks = JoinSet::new();
    tasks.spawn(async move {
        recv(reader, recv_tx, send_tx, role).await;
    });
    tasks.spawn(async move {
        send(writer, send_rx).await;
    });

    tokio::select! {
        _ = tasks.join_next() => {}
        _ = shutdown_rx.recv() => {}
    }
    // Reaching this section means that the session has been shutdown or had errored.
    // So abort the tasks.
    tasks.shutdown().await;

    println!("Session {} has ended", peer_addr);
}

async fn recv(
    mut reader: ReadHalf<TcpStream>,
    mut recv_tx: InMessageTx,
    send_tx: mpsc::Sender<Bytes>,
    role: Arc<dyn Role>,
) {
    let (transfer_tx, mut transfer_rx) = broadcast::channel(1);

    loop {
        match recv_internal(&mut reader).await {
            Ok(Recv::Buf(buf)) => {
                if let Ok(tx) = transfer_rx.try_recv() {
                    recv_tx = tx;
                }

                let ctx = SessionContext::new(role.clone(), send_tx.clone(), transfer_tx.clone());
                _ = recv_tx.send((ctx, buf)).await;
            }
            Ok(Recv::EOF) => {
                break;
            }
            Err(e) => {
                eprintln!("Error receiving: {}", e);
                break;
            }
        }
    }
}

async fn recv_internal(reader: &mut ReadHalf<TcpStream>) -> Result<Recv, Box<dyn Error + Send + Sync>> {
    let mut header_buf = [0u8; 4];
    let n = reader.read_exact(&mut header_buf).await?;
    if n == 0 {
        return Ok(Recv::EOF);
    }

    let body_len = u32::from_ne_bytes(header_buf) as usize;
    let mut body_buf = BytesMut::with_capacity(body_len);

    reader.read_exact(&mut body_buf).await?;
    if n == 0 {
        return Ok(Recv::EOF);
    }

    Ok(Recv::Buf(Bytes::from(body_buf)))
}

async fn send(
    mut writer: WriteHalf<TcpStream>,
    mut send_rx: mpsc::Receiver<Bytes>,
) {
    loop {
        match send_rx.recv().await {
            Some(buf) => {
                if let Err(e) = writer.write_all(buf.iter().as_slice()).await {
                    eprintln!("Error sending: {}", e);
                    break;
                }
            }
            None => {
                break;
            }
        }
    }
}