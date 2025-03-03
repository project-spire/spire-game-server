use crate::core::config;

mod core;
mod physics;
mod rooms;
mod protocol;

#[tokio::main]
async fn main() {
    config::init();

    let _ = core::server::run_server(6400).await;
}
