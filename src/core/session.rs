use crate::core::role::Role;
use bytes::Bytes;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::{Arc, OnceLock};
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{RwLock, broadcast, mpsc};
use tokio::task::JoinSet;

pub type InMessage = (Arc<SessionContext>, Bytes);
pub type OutMessage = Bytes;

pub struct SessionContext {
    pub role: OnceLock<Role>,
    pub in_message_tx: RwLock<mpsc::Sender<InMessage>>,
    pub out_message_tx: mpsc::Sender<OutMessage>,
    pub close_tx: mpsc::Sender<()>,
}

impl SessionContext {
    pub fn new(
        in_message_tx: mpsc::Sender<InMessage>,
        out_message_tx: mpsc::Sender<OutMessage>,
        close_tx: mpsc::Sender<()>,
    ) -> SessionContext {
        let role = OnceLock::new();
        let in_message_tx = RwLock::new(in_message_tx);
        SessionContext {
            role,
            in_message_tx,
            out_message_tx,
            close_tx,
        }
    }
}

enum Recv {
    Buf(Bytes),
    EOF,
}

pub async fn run_session(
    stream: TcpStream,
    in_message_tx: mpsc::Sender<InMessage>,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> Arc<SessionContext> {
    let peer_addr = stream
        .peer_addr()
        .unwrap_or(SocketAddr::from(([0, 0, 0, 0], 0)));
    let (reader, writer) = tokio::io::split(stream);

    let (out_message_tx, in_message_rx) = mpsc::channel(32);
    let (close_tx, mut close_rx) = mpsc::channel(1);

    let ctx = Arc::new(SessionContext::new(in_message_tx, out_message_tx, close_tx));
    let ctx_recv = ctx.clone();

    println!("Session {} has started", peer_addr);

    let mut tasks = JoinSet::new();
    tasks.spawn(async move {
        recv(reader, ctx_recv).await;
    });
    tasks.spawn(async move {
        send(writer, in_message_rx).await;
    });

    tokio::select! {
        _ = tasks.join_next() => {}
        _ = close_rx.recv() => { close_rx.close(); }
        _ = shutdown_rx.recv() => {}
    }
    // Reaching this section means that the session has been shutdown or had errored.
    // So abort the tasks.
    tasks.shutdown().await;

    println!("Session {} has ended", peer_addr);
    ctx
}

async fn recv(mut reader: ReadHalf<TcpStream>, ctx: Arc<SessionContext>) {
    loop {
        match recv_internal(&mut reader).await {
            Ok(Recv::Buf(data)) => {
                _ = ctx
                    .in_message_tx
                    .read()
                    .await
                    .send((ctx.clone(), data))
                    .await;
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

async fn recv_internal(
    reader: &mut ReadHalf<TcpStream>,
) -> Result<Recv, Box<dyn Error + Send + Sync>> {
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

async fn send(mut writer: WriteHalf<TcpStream>, mut out_message_rx: mpsc::Receiver<OutMessage>) {
    loop {
        match out_message_rx.recv().await {
            Some(data) => {
                if let Err(e) = writer.write_all(&data[..]).await {
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
