use std::net::{IpAddr, SocketAddr};

mod core;

// include!(concat!(env!("OUT_DIR"), "/spire.msg.rs"));

#[tokio::main]
async fn main() {
    let mut server = core::server::Server::new();
    let _ = server.run(6400).await;
}
