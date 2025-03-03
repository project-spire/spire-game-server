mod core;
mod physics;
mod rooms;
// include!(concat!(env!("OUT_DIR"), "/spire.msg.rs"));

#[tokio::main]
async fn main() {
    let _ = core::server::run_server(6400).await;
}
