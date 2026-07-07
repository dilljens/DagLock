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

    #[arg(long, default_value = "testnet-10")]
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

    #[arg(long, default_value_t = false)]
    pub auto_settle_escrows: bool,

    #[arg(long, default_value_t = false)]
    pub auto_escalate_disputes: bool,

    #[arg(long, default_value_t = false)]
    pub auto_sweep_deposits: bool,

    #[arg(long)]
    pub ai_mediator_api_key: Option<String>,

    #[arg(long, default_value = "gpt-4o")]
    pub ai_mediator_model: String,

    /// Skip Ed25519 chat signature verification (dev mode only).
    #[arg(long, default_value_t = false)]
    pub mock_chat_sig: bool,

    /// Interval between anchor batch flushes (seconds).
    #[arg(long, default_value_t = 300)]
    pub anchor_interval_seconds: u64,

    /// Optional private key hex for broadcasting anchor txs.
    /// If set, the service tries to construct and broadcast Kaspa self-pay
    /// transactions carrying the anchor payload.  Without this flag, payloads
    /// are logged for manual broadcast.
    #[arg(long)]
    pub anchor_wallet_key: Option<String>,

    /// Hours after case resolution to auto-wipe revealed chat evidence.
    #[arg(long, default_value_t = 24)]
    pub evidence_auto_wipe_hours: u64,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_args() -> Args {
        Args {
            host: "0.0.0.0".into(),
            port: 8543,
            wrpc_url: None,
            no_wrpc: true,
            network: "testnet-10".into(),
            database_url: "sqlite::memory:".into(),
            daglock_kas_template: None,
            daglock_krc20_template: None,
            daglock_vault_softlock_template: None,
            daglock_vault_multisig_template: None,
            daglock_reputation_template: None,
            log_level: "info".into(),
            cors_origin: "*".into(),
            allow_mainnet: false,
            db_type: "sqlite".into(),
            mock_auth: false,
            treasury_pubkey: None,
            auto_sweep_vaults: false,
            auto_settle_escrows: false,
            auto_escalate_disputes: false,
            auto_sweep_deposits: false,
            ai_mediator_api_key: None,
            ai_mediator_model: "gpt-4o".to_string(),
            mock_chat_sig: false,
            anchor_interval_seconds: 300,
            anchor_wallet_key: None,
            evidence_auto_wipe_hours: 24,
        }
    }

    #[test]
    fn valid_testnet_config_passes() {
        valid_args().validate();
    }

    #[test]
    fn invalid_network_panics() {
        let mut args = valid_args();
        args.network = "invalidnet".into();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| args.validate()));
        assert!(result.is_err());
    }

    #[test]
    fn invalid_log_level_panics() {
        let mut args = valid_args();
        args.log_level = "verbose".into();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| args.validate()));
        assert!(result.is_err());
    }

    #[test]
    fn invalid_db_type_panics() {
        let mut args = valid_args();
        args.db_type = "mysql".into();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| args.validate()));
        assert!(result.is_err());
    }

    #[test]
    fn zero_port_panics() {
        let mut args = valid_args();
        args.port = 0;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| args.validate()));
        assert!(result.is_err());
    }

    #[test]
    fn mock_auth_on_mainnet_panics() {
        let mut args = valid_args();
        args.network = "mainnet".into();
        args.allow_mainnet = true;
        args.mock_auth = true;
        std::env::set_var("DAGLOCK_MESSAGE_KEY", "ab".repeat(32));
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| args.validate()));
        assert!(result.is_err());
        std::env::remove_var("DAGLOCK_MESSAGE_KEY");
    }

    #[test]
    fn mainnet_without_allow_flag_panics() {
        let mut args = valid_args();
        args.network = "mainnet".into();
        args.allow_mainnet = false;
        std::env::set_var("DAGLOCK_MESSAGE_KEY", "ab".repeat(32));
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| args.validate()));
        assert!(result.is_err());
        std::env::remove_var("DAGLOCK_MESSAGE_KEY");
    }

    #[test]
    fn mainnet_with_allow_flag_passes() {
        let mut args = valid_args();
        args.network = "mainnet".into();
        args.allow_mainnet = true;
        args.no_wrpc = true;
        args.database_url = "sqlite::memory:".into();
        std::env::set_var("DAGLOCK_MESSAGE_KEY", "ab".repeat(32));
        args.validate();
        std::env::remove_var("DAGLOCK_MESSAGE_KEY");
    }
}
