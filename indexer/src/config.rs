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

    #[arg(long)]
    pub daglock_vault_softlock_template: Option<String>,

    #[arg(long)]
    pub daglock_vault_multisig_template: Option<String>,

    #[arg(long)]
    pub daglock_reputation_template: Option<String>,

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
    #[arg(long, default_value_t = false)]
    pub mock_auth: bool,

    /// Canonical treasury public key (64 hex chars).
    /// When set, the compile API rejects requests with a different treasury key.
    #[arg(long)]
    pub treasury_pubkey: Option<String>,

    #[arg(long, default_value_t = false)]
    pub auto_sweep_vaults: bool,
}

impl Args {
    /// Validate all configuration values at startup.
    /// Panics with a clear message on invalid config.
    pub fn validate(&self) {
        // Network must be a known value
        match self.network.as_str() {
            "mainnet" | "testnet-12" | "testnet-11" | "testnet-10" | "devnet" | "simnet" => {}
            other => panic!(
                "Invalid network: '{other}'. Expected: mainnet, testnet-12, testnet-11, testnet-10, devnet, simnet"
            ),
        }

        // Log level must be valid
        match self.log_level.to_lowercase().as_str() {
            "error" | "warn" | "info" | "debug" | "trace" => {}
            other => {
                panic!("Invalid log level: '{other}'. Expected: error, warn, info, debug, trace")
            }
        }

        // Port must be > 0
        if self.port == 0 {
            panic!("Port must be > 0, got 0");
        }

        // DB type must be sqlite or postgres
        match self.db_type.as_str() {
            "sqlite" | "postgres" => {}
            other => panic!("Invalid db type: '{other}'. Expected: sqlite, postgres"),
        }

        // Treasury pubkey format: 64 hex chars if provided
        if let Some(ref key) = self.treasury_pubkey {
            let clean = key.strip_prefix("0x").unwrap_or(key);
            if clean.len() != 64 || !clean.chars().all(|c| c.is_ascii_hexdigit()) {
                panic!(
                    "Invalid treasury_pubkey: must be 64 hex chars (got {} chars: '{clean}')",
                    clean.len()
                );
            }
        }

        // Mock auth on mainnet is forbidden
        if self.mock_auth && self.network == "mainnet" {
            panic!(
                "REFUSING TO START: --mock-auth is set but network is mainnet. \
                 Mock authentication accepts any signature — never use on mainnet."
            );
        }

        // Mainnet requires --allow-mainnet flag
        if self.network == "mainnet" && !self.allow_mainnet {
            panic!(
                "DagLock refuses to start on mainnet without --allow-mainnet flag. \
                 Set --allow-mainnet to acknowledge production risk."
            );
        }

        // Mainnet requires DAGLOCK_MESSAGE_KEY
        if self.network == "mainnet" && std::env::var("DAGLOCK_MESSAGE_KEY").is_err() {
            panic!(
                "DAGLOCK_MESSAGE_KEY environment variable must be set on mainnet. \
                 Generate one with: openssl rand -hex 32"
            );
        }
    }
}
