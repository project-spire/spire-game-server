mod character;
mod core;
mod item;
mod physics;
mod player;
mod world;

use protocol;

#[tokio::main]
async fn main() {
    core::config::Config::init();

    _ = core::server::run_server().await;
}
