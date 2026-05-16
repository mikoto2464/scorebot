pub mod config;

use std::env;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use dotenvy::dotenv;

// 1. 引入 anyhow 的 Context trait，这样才能在 Result 上调用 .context()
use anyhow::Context;

use interprocess::local_socket::{GenericFilePath, GenericNamespaced, ToFsName, ToNsName, NameType};
use interprocess::local_socket::tokio::Listener;
use interprocess::local_socket::traits::tokio::Listener as _ListenerTrait;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use tracing_subscriber::fmt::init;
use scorebot_common::SOCKET_NAME;
use tracing::info;
use crate::config::ScorebotConfig;

#[tokio::main]
// 2. 将返回值修改为 anyhow::Result<()>
async fn main() -> anyhow::Result<()> {
    init();
    dotenv().ok();

    let db_url = env::var("DATABASE_URL").unwrap_or("sqlite:yukino.db".to_string());
    let options = SqliteConnectOptions::from_str(&db_url)
        .context("Failed to parse DATABASE_URL")?
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5));

    let pool = SqlitePoolOptions::new()
        .connect_with(options)
        .await
        .context("Failed to connect to SQLite database")?;

    let bot_app_id = env::var("APP_ID")
        .context("Failed to get APP_ID")?;
    let bot_app_secret = env::var("APP_SECRET")
        .context("Failed to get SECRET")?;

    let _config = Arc::new(ScorebotConfig {
        bot_app_id,
        bot_app_secret,
        db: pool.clone(),
    });

    // 跨平台获取套接字名字
    let socket_name = if GenericNamespaced::is_supported() {
        SOCKET_NAME.to_ns_name::<GenericNamespaced>()
            .context("Failed to get SOCKET_NAME")?
    } else {
        SOCKET_NAME.to_fs_name::<GenericFilePath>()
            .context("Failed to get SOCKET_NAME")?
    };

    // 绑定本地套接字
    let _listener = Listener::from_options(
        interprocess::local_socket::ListenerOptions::new().name(socket_name)
    ).with_context(|| format!("Failed to bind socket: {}", SOCKET_NAME))?;

    info!("Bound socket to: {}", SOCKET_NAME);

    Ok(())
}