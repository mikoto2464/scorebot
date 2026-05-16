pub mod config;

use std::env;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use dotenvy::dotenv;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use tracing::error;
use tracing_subscriber::fmt::init;
use crate::config::ScorebotConfig;

#[tokio::main]
async fn main() {
    init();
    dotenv().ok();

    let db_url = env::var("DATABASE_URL").unwrap_or("sqlite:yukino.db".to_string());
    let options = SqliteConnectOptions::from_str(&db_url)
        .unwrap()
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5));
    let pool = SqlitePoolOptions::new()
        .connect_with(options)
        .await
        .map_err(|e| {
            error!("Failed to connect to SQLite database: {}", e);
        }).unwrap();
    let bot_app_id = env::var("APP_ID")
        .map_err(|e| {
            error!("Failed to get APP_ID: {}", e);
        }).unwrap();
    let bot_app_secret = env::var("SECRET")
        .map_err(|e| {
            error!("Failed to get SECRET: {}", e);
        }).unwrap();
    let config = Arc::new(ScorebotConfig {
        bot_app_id,
        bot_app_secret,
        db: pool.clone(),
    });

    
}
