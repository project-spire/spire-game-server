mod character;
mod core;
mod item;
mod physics;
mod world;

use protocol;

#[tokio::main]
async fn main() {
    core::config::init();

    _ = core::server::run_server().await;
}
