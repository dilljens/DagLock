use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(name = "daglock-indexer", version = "0.1.0")]
pub struct Args {
    #[arg(long, default_value = "0.0.0.0")]
    pub host: String,

    #[arg(long, default_value_t = 8543)]
    pub port: u16,

    #[arg(long)]
    pub wrpc_url: Option<String>,

    /// Skip wRPC connection entirely (local dev).
    #[arg(long, default_value_t = false)]
    pub no_wrpc: bool,

    #[arg(long, default_value = "testnet-12")]
    pub network: String,

    #[arg(long, default_value = "sqlite:daglock.db")]
    pub database_url: String,

    #[arg(long)]
    pub daglock_kas_template: Option<String>,

    #[arg(long)]
    pub daglock_krc20_template: Option<String>,

    #[arg(long, default_value = "info")]
    pub log_level: String,

    /// Allowed CORS origin. Set to a specific domain in production, * for dev.
    #[arg(long, default_value = "https://daglock.com")]
    pub cors_origin: String,

    /// Block mainnet if this flag isn't explicitly set (production safety).
    #[arg(long)]
    pub allow_mainnet: bool,

    /// Database type: sqlite or postgres.
    #[arg(long, default_value = "sqlite")]
    pub db_type: String,

    /// Use mock authentication (any hex string passes).
    /// For dev/testnet only — panics if used with --network mainnet.
    #[arg(long, default_value_t = true)]
    pub mock_auth: bool,

    /// Canonical treasury public key (64 hex chars).
    /// When set, the compile API rejects requests with a different treasury key.
    #[arg(long)]
    pub treasury_pubkey: Option<String>,
}

impl Args {}
