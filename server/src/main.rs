use crate::core::config;

mod character;
mod core;
mod physics;
mod rooms;

#[tokio::main]
async fn main() {
    config::init();

    _ = core::server::run_server().await;
}
