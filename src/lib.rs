//! # Vaulta Geyser Indexer
//!
//! High-performance Geyser plugin for real-time vault state indexing with PostgreSQL
//! and Redis backend. Provides sub-100ms latency for vault state queries.
//!
//! ## Features
//!
//! - **Real-time Indexing**: Geyser plugin for instant account updates
//! - **PostgreSQL Backend**: Persistent storage for vault state
//! - **Redis Caching**: Sub-100ms query latency
//! - **High Performance**: Optimized for high-throughput indexing
//! - **Fault Tolerant**: Automatic recovery and retry logic
//!
//! ## Example
//!
//! ```rust,no_run
//! use vaulta_geyser_indexer::GeyserIndexerPlugin;
//!
//! // Plugin is loaded by Solana validator
//! // Configuration via config file
//! ```

pub mod config;
pub mod database;
pub mod geyser_plugin;
pub mod indexer;
pub mod redis_cache;
pub mod types;
pub mod utils;

pub use geyser_plugin::GeyserIndexerPlugin;
