//! DagLock Indexer library.
//!
//! Provides the REST API, database, and listener components.
//! Used by integration tests and as a dependency for other crates.

pub mod api;
pub mod config;
pub mod db;
pub mod listener;
pub mod types;
