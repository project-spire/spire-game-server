use crate::core::config;

mod core;
mod physics;
mod protocol;
mod rooms;

#[tokio::main]
async fn main() {
    config::init();

    _ = core::server::run_server().await;
}
