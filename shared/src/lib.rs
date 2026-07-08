//! DagLock Shared — Common constants and validation utilities.
//!
//! This crate provides shared constants (fee denominator, timeouts, limits)
//! and validation helpers used across all DagLock components:
//! contracts, indexer, CLI, WASM SDK, web, and bot.

pub mod constants;
pub mod validation;

// Re-export commonly used items
pub use constants::*;
pub use validation::{
    calculate_fee, calculate_net_amount, kas_to_sompi, parse_hex, sompi_to_kas,
    validate_amount_sompi, validate_kaspa_address, validate_template_hash, validate_timeout,
    validate_timeout_duration, validate_trade_hash, validate_trade_hash_optional, TradeHash,
    ValidationError, ValidationResult,
};
