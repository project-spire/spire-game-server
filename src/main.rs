mod core;

// include!(concat!(env!("OUT_DIR"), "/spire.msg.rs"));

#[tokio::main]
async fn main() {
    let mut server = core::server::Server::new();
    let _ = server.run("localhost:6400").await;
}
