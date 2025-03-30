use bytes::Bytes;
use crate::protocol::{ProtocolCategory, deserialize_header};
use std::error::Error;
use std::fmt;
use std::fmt::Formatter;
use std::net::SocketAddr;
use std::sync::Arc;
use bevy_ecs::component::Component;
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{RwLock, broadcast, mpsc};
use tokio::task::JoinSet;

pub type InMessage = (Arc<SessionContext>, ProtocolCategory, Bytes);
pub type OutMessage = Bytes;

#[derive(Component)]
pub struct Session {
    pub ctx: Arc<SessionContext>,
}

impl Session {
    pub fn new(ctx: Arc<SessionContext>) -> Self {
        Self { ctx }
    }
}

pub struct SessionContext {
    pub is_open: RwLock<bool>,
    pub peer_addr: SocketAddr,

    pub in_message_tx: RwLock<mpsc::Sender<InMessage>>,
    pub out_message_tx: mpsc::Sender<OutMessage>,
    pub close_tx: mpsc::Sender<()>,
}

impl SessionContext {
    pub fn new(
        peer_addr: SocketAddr,
        in_message_tx: mpsc::Sender<InMessage>,
        out_message_tx: mpsc::Sender<OutMessage>,
        close_tx: mpsc::Sender<()>,
    ) -> SessionContext {
        let is_open = RwLock::new(true);
        let in_message_tx = RwLock::new(in_message_tx);

        SessionContext {
            is_open,
            peer_addr,
            in_message_tx,
            out_message_tx,
            close_tx,
        }
    }

    pub async fn close(&self) {
        *self.is_open.write().await = false;
        _ = self.close_tx.send(()).await;
    }

    pub async fn is_open(&self) -> bool {
        *self.is_open.read().await
    }
}

impl fmt::Display for SessionContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Session({})", self.peer_addr)
    }
}

enum Recv {
    Message(ProtocolCategory, Bytes),
    EOF,
    InvalidHeader,
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

    let ctx = Arc::new(SessionContext::new(peer_addr, in_message_tx, out_message_tx, close_tx));
    let ctx_recv = ctx.clone();

    println!("{} has started", ctx);

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
    ctx.close().await;
    close_rx.close();

    println!("{} has ended", ctx);
    ctx
}

async fn recv(mut reader: ReadHalf<TcpStream>, ctx: Arc<SessionContext>) {
    loop {
        match recv_internal(&mut reader).await {
            Ok(Recv::Message(protocol, data)) => {
                _ = ctx
                    .in_message_tx
                    .read()
                    .await
                    .send((ctx.clone(), protocol, data))
                    .await;
            }
            Ok(Recv::EOF) => {
                break;
            }
            Ok(Recv::InvalidHeader) => {
                eprintln!("Invalid header received");
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
    let mut header_buf = [0u8; 4];
    let n = reader.read_exact(&mut header_buf).await?;
    if n == 0 {
        return Ok(Recv::EOF);
    }

    let (protocol, body_len) = deserialize_header(&header_buf);
    if protocol == ProtocolCategory::None {
        return Ok(Recv::InvalidHeader);
    }

    let mut body_buf = vec![0u8; body_len as usize];
    let n = reader.read_exact(&mut body_buf).await?;
    if n == 0 {
        return Ok(Recv::EOF);
    }

    Ok(Recv::Message(protocol, Bytes::from(body_buf)))
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
