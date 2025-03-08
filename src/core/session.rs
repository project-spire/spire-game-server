use crate::core::role::Role;
use bytes::{Bytes, BytesMut};
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinSet;

pub type InMessage = (Arc<SessionContext>, Bytes);
pub type InMessageTx = mpsc::Sender<InMessage>;
pub type InMessageRx = mpsc::Receiver<InMessage>;

pub struct SessionContext {
    pub role: Role,
    pub send_tx: mpsc::Sender<Bytes>,
    pub transfer_tx: mpsc::Sender<InMessageTx>,
    pub close_tx: mpsc::Sender<()>,
}

impl SessionContext {
    pub fn new(
        role: Role,
        send_tx: mpsc::Sender<Bytes>,
        transfer_tx: mpsc::Sender<InMessageTx>,
        close_tx: mpsc::Sender<()>,
    ) -> SessionContext {
        SessionContext { role, send_tx, transfer_tx, close_tx }
    }
}

enum Recv {
    Buf(Bytes),
    EOF,
}

pub async fn run_session(
    stream: TcpStream,
    recv_tx: InMessageTx,
    role: Role,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    let peer_addr = stream.peer_addr().unwrap_or(SocketAddr::from(([0, 0, 0, 0], 0)));
    let (reader, writer) = tokio::io::split(stream);
    let (send_tx, send_rx) = mpsc::channel(64);
    let (close_tx, mut close_rx) = mpsc::channel(1);

    println!("Session {} has started", peer_addr);

    let mut tasks = JoinSet::new();
    tasks.spawn(async move {
        recv(reader, recv_tx, send_tx, close_tx, role).await;
    });
    tasks.spawn(async move {
        send(writer, send_rx).await;
    });

    tokio::select! {
        _ = tasks.join_next() => {}
        _ = close_rx.recv() => {}
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
    close_tx: mpsc::Sender<()>,
    role: Role,
) {
    let (transfer_tx, mut transfer_rx) = mpsc::channel(1);
    let ctx = Arc::new(SessionContext::new(role, send_tx, transfer_tx, close_tx));

    loop {
        match recv_internal(&mut reader).await {
            Ok(Recv::Buf(data)) => {
                if let Ok(tx) = transfer_rx.try_recv() {
                    recv_tx = tx;
                }
                _ = recv_tx.send((ctx.clone(), data)).await;
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
    let mut header_buf = [0u8; 2];
    let n = reader.read_exact(&mut header_buf).await?;
    if n == 0 {
        return Ok(Recv::EOF);
    }

    let body_len = u16::from_be_bytes(header_buf) as usize;
    let mut body_buf = vec![0u8; body_len];

    let n = reader.read_exact(&mut body_buf).await?;
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