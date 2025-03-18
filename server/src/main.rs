mod character;
mod core;
mod physics;
mod rooms;
mod world;

use protocol;

use crate::core::config;

#[tokio::main]
async fn main() {
    config::init();

    _ = core::server::run_server().await;
}
