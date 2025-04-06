use bytes::{Bytes, BytesMut};
use crate::protocol::{HEADER_SIZE, ProtocolCategory, deserialize_header};
use std::fmt;
use std::fmt::Formatter;
use std::net::SocketAddr;
use std::sync::{atomic::AtomicBool, Arc};
use std::sync::atomic::Ordering;
use bevy_ecs::component::Component;
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinSet;

pub type InMessage = (Arc<SessionContext>, ProtocolCategory, Bytes);
pub type OutMessage = Bytes;

pub enum SessionCommand {
    Close,
    RoomTransfer { in_message_tx: mpsc::Sender<InMessage> }
}

pub struct SessionContext {
    pub is_open: AtomicBool,
    pub peer_addr: SocketAddr,

    pub out_message_tx: mpsc::Sender<OutMessage>,
    pub command_tx: mpsc::Sender<SessionCommand>,
}

impl SessionContext {
    pub fn new(
        peer_addr: SocketAddr,
        out_message_tx: mpsc::Sender<OutMessage>,
        command_tx: mpsc::Sender<SessionCommand>,
    ) -> SessionContext {
        let is_open = AtomicBool::new(true);

        SessionContext {
            is_open,
            peer_addr,

            out_message_tx,
            command_tx,
        }
    }

    pub async fn close(&self) {
        if !self.is_open.swap(false, Ordering::SeqCst) {
            return;
        }

        _ = self.command_tx.send(SessionCommand::Close).await;
    }

    pub fn is_open(&self) -> bool {
        self.is_open.load(Ordering::SeqCst)
    }
}

impl fmt::Display for SessionContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Session({})", self.peer_addr)
    }
}

#[derive(Component)]
pub struct Session {
    pub ctx: Arc<SessionContext>,
}

impl Session {
    pub fn new(ctx: Arc<SessionContext>) -> Self {
        Self { ctx }
    }
}

pub async fn run_session(
    stream: TcpStream,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> Arc<SessionContext> {
    let peer_addr = stream
        .peer_addr()
        .unwrap_or(SocketAddr::from(([0, 0, 0, 0], 0)));
    let (reader, writer) = tokio::io::split(stream);

    let (out_message_tx, out_message_rx) = mpsc::channel(32);
    let (command_tx, command_rx) = mpsc::channel(1);

    let ctx = Arc::new(SessionContext::new(peer_addr, out_message_tx, command_tx));

    println!("{} has started", ctx);

    let mut tasks = JoinSet::new();
    tasks.spawn(async move {
        recv(reader, command_rx).await;
    });
    tasks.spawn(async move {
        send(writer, out_message_rx).await;
    });

    tokio::select! {
        _ = tasks.join_next() => {}
        _ = shutdown_rx.recv() => {}
    }
    // Reaching this section means that the session has been shutdown or had errored.
    // So abort the tasks.
    tasks.shutdown().await;
    ctx.close().await;

    println!("{} has ended", ctx);
    ctx
}

async fn recv(mut reader: ReadHalf<TcpStream>, mut command_rx: mpsc::Receiver<SessionCommand>) {
    let in_message_tx = None;

    loop {
        let mut header_buf = BytesMut::with_capacity(HEADER_SIZE);
        let mut header_bytes_read = 0;
        while header_bytes_read < HEADER_SIZE {
            tokio::select! {
                n = reader.read_buf(&mut header_buf[header_bytes_read..HEADER_SIZE]) => match n {
                    Ok(n) if n == 0 => {
                        return; // EOF
                    }
                    Ok(n) => {
                        header_bytes_read += n;
                    }
                    Err(e) => {
                        todo!();
                        return;
                    }
                }
                // command = command_rx.recv() => match command {
                //     todo!();
                // }
            }
        }

        let header = deserialize_header(&header_buf[..HEADER_SIZE].try_into().unwrap());
        let body_buf = BytesMut::with_capacity(header.length);
        let mut body_bytes_read = 0;
        while body_bytes_read < header.length {
            tokio::select! {
                n = reader.read_buf(&mut body_buf[body_bytes_read..header.length]) => match n {
                    Ok(n) if n == 0 => {
                        return; // EOF
                    }
                    Ok(n) => {
                        body_bytes_read += n;
                    }
                    Err(e) => {
                        todo!();
                        return;
                    }
                }
                // command = command_rx.recv().await => match command {
                //     todo!();
                // }
            }
        }
    }
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
