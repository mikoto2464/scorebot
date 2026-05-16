use sqlx::SqlitePool;

#[derive(Clone)]
pub struct ScorebotConfig {
    pub bot_app_id: String,
    pub bot_app_secret: String,
    pub db: SqlitePool
}
