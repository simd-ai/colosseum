use anyhow::Result;

/// Application configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub redis_url: String,
    pub solana_rpc_url: String,
    pub solana_ws_url: String,
    pub authority_keypair_path: String,
    pub provider_registry_program_id: String,
    pub job_escrow_program_id: String,
    pub compute_receipts_program_id: String,
    pub grid_token_mint: String,
    pub grid_token_decimals: u8,
    pub scheduler_port: u16,
    pub verifier_port: u16,
    pub scheduler_url: String,
    pub verifier_url: String,
}

impl AppConfig {
    /// Load configuration from environment variables.
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        Ok(AppConfig {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://solgrid:solgrid@localhost:5432/solgrid".into()),
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".into()),
            solana_rpc_url: std::env::var("SOLANA_RPC_URL")
                .unwrap_or_else(|_| "https://api.devnet.solana.com".into()),
            solana_ws_url: std::env::var("SOLANA_WS_URL")
                .unwrap_or_else(|_| "wss://api.devnet.solana.com".into()),
            authority_keypair_path: std::env::var("AUTHORITY_KEYPAIR_PATH")
                .unwrap_or_else(|_| "~/.config/solana/id.json".into()),
            provider_registry_program_id: std::env::var("PROVIDER_REGISTRY_PROGRAM_ID")
                .unwrap_or_else(|_| "Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS".into()),
            job_escrow_program_id: std::env::var("JOB_ESCROW_PROGRAM_ID")
                .unwrap_or_else(|_| "HmbTLCmaGtYhSsT3D2RzKFkN3CKqZbHN4KxrFEhMccgH".into()),
            compute_receipts_program_id: std::env::var("COMPUTE_RECEIPTS_PROGRAM_ID")
                .unwrap_or_else(|_| "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".into()),
            grid_token_mint: std::env::var("GRID_TOKEN_MINT")
                .unwrap_or_else(|_| "11111111111111111111111111111111".into()),
            grid_token_decimals: std::env::var("GRID_TOKEN_DECIMALS")
                .unwrap_or_else(|_| "6".into())
                .parse()
                .unwrap_or(6),
            scheduler_port: std::env::var("SCHEDULER_PORT")
                .unwrap_or_else(|_| "8080".into())
                .parse()
                .unwrap_or(8080),
            verifier_port: std::env::var("VERIFIER_PORT")
                .unwrap_or_else(|_| "8081".into())
                .parse()
                .unwrap_or(8081),
            scheduler_url: std::env::var("SCHEDULER_URL")
                .unwrap_or_else(|_| "http://localhost:8080".into()),
            verifier_url: std::env::var("VERIFIER_URL")
                .unwrap_or_else(|_| "http://localhost:8081".into()),
        })
    }
}
