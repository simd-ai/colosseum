use anyhow::Result;
use sqlx::PgPool;

/// Shared application state passed to all Axum handlers.
pub struct AppState {
    pub pool: PgPool,
    pub redis: redis::Client,
    pub config: sol_common::AppConfig,
    pub http_client: reqwest::Client,
}

impl AppState {
    pub async fn new(config: sol_common::AppConfig) -> Result<Self> {
        let pool = PgPool::connect(&config.database_url).await?;
        let redis = redis::Client::open(config.redis_url.clone())?;

        tracing::info!("✅ Connected to PostgreSQL");
        tracing::info!("✅ Connected to Redis");

        Ok(AppState {
            pool,
            redis,
            config,
            http_client: reqwest::Client::new(),
        })
    }
}
