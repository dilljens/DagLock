//! Shared constants for DagLock protocol.

/// Fee denominator for the 0.5% protocol fee (1/200).
/// Used across covenants, indexer, CLI, web, and WASM SDK.
pub const FEE_DENOMINATOR: u64 = 200;

/// Fee as basis points (50 bps = 0.5%).
pub const FEE_BASIS_POINTS: u16 = 50;

/// Maximum allowed escrow amount in sompi (1M KAS).
pub const MAX_ESCROW_AMOUNT_SOMPI: u64 = 100_000_000_000_000;

/// Minimum allowed escrow amount in sompi (1 sompi).
pub const MIN_ESCROW_AMOUNT_SOMPI: u64 = 1;

/// Template hash length in bytes (BLAKE2b-160 = 20 bytes, matches P2SH).
pub const TEMPLATE_HASH_LENGTH: usize = 20;

/// Trade hash length in bytes (SHA-256 = 32 bytes).
pub const TRADE_HASH_LENGTH: usize = 32;

/// Default escrow timeout in seconds (24 hours).
pub const DEFAULT_TIMEOUT_SECONDS: u64 = 86_400;

/// Maximum escrow timeout in seconds (1 year).
pub const MAX_TIMEOUT_SECONDS: u64 = 31_536_000;

/// Jury case voting period in seconds (72 hours).
pub const JURY_VOTING_PERIOD_SECONDS: u64 = 259_200;

/// Recency window for reputation scoring in seconds (90 days).
pub const REPUTATION_RECENCY_WINDOW_SECONDS: i64 = 7_776_000;

/// Vouch expiration in seconds (6 months).
pub const VOUCH_EXPIRATION_SECONDS: i64 = 15_768_000;

/// Message encryption key length in bytes (AES-256-GCM = 32 bytes).
pub const MESSAGE_KEY_LENGTH: usize = 32;

/// Maximum preimage length for atomic swaps.
pub const MAX_PREIMAGE_LENGTH: usize = 1024;

/// CoinGecko API rate limit (calls per month on free tier).
pub const COINGECKO_MONTHLY_LIMIT: u32 = 10_000;

/// Market price update interval in seconds (15 minutes).
pub const MARKET_PRICE_UPDATE_INTERVAL_SECONDS: u64 = 900;

/// DAA score polling interval in seconds (10 seconds).
pub const DAA_POLL_INTERVAL_SECONDS: u64 = 10;

/// WebSocket broadcast channel capacity.
pub const WS_CHANNEL_CAPACITY: usize = 100;

/// Maximum request body size in bytes (1 MB).
pub const MAX_REQUEST_BODY_SIZE: usize = 1_048_576;

/// Default API port.
pub const DEFAULT_API_PORT: u16 = 8443;

/// Kaspa address prefix for mainnet.
pub const KASPA_MAINNET_PREFIX: &str = "kaspa:";

/// Kaspa address prefix for testnet.
pub const KASPA_TESTNET_PREFIX: &str = "kaspatest:";

/// Kaspa address prefix for simnet/devnet.
pub const KASPA_SIMNET_PREFIX: &str = "kaspadev:";
