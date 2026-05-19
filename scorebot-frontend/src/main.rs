pub mod sender;

use anyhow::Context;
use dotenvy::dotenv;
use interprocess::local_socket::traits::tokio::Stream;
use tracing_subscriber::fmt::init;
use crate::sender::QueueCacheSender;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init();
    dotenv().ok();

    let sender = QueueCacheSender::new();

    Ok(())
}
