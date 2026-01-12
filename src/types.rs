use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use time::OffsetDateTime;

/// Vault account state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultState {
    pub vault_address: Pubkey,
    pub owner: Pubkey,
    pub balance: u64,
    pub assets: HashMap<String, AssetBalance>,
    pub permissions: Vec<Permission>,
    pub last_updated: OffsetDateTime,
    pub slot: u64,
    pub write_version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetBalance {
    pub mint: Pubkey,
    pub amount: u64,
    pub decimals: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub pubkey: Pubkey,
    pub permission_type: PermissionType,
    pub granted_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionType {
    Owner,
    Admin,
    Operator,
    Viewer,
}

/// Account update event from Geyser
#[derive(Debug, Clone)]
pub struct AccountUpdate {
    pub pubkey: Pubkey,
    pub lamports: u64,
    pub owner: Pubkey,
    pub executable: bool,
    pub rent_epoch: u64,
    pub data: Vec<u8>,
    pub write_version: u64,
    pub slot: u64,
    pub is_startup: bool,
}

/// Indexing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingStats {
    pub total_accounts_indexed: u64,
    pub accounts_per_second: f64,
    pub average_latency_ms: f64,
    pub cache_hit_rate: f64,
    pub last_indexed_slot: u64,
    pub uptime_seconds: u64,
}

/// Cache entry for vault state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub vault_state: VaultState,
    pub cached_at: OffsetDateTime,
    pub ttl_seconds: u64,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub max_connections: u32,
    pub connection_timeout_seconds: u64,
}

/// Redis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub ttl_seconds: u64,
    pub max_connections: u32,
    pub connection_timeout_seconds: u64,
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub vault_program_id: String,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub batch_size: usize,
    pub flush_interval_ms: u64,
    pub enable_cache: bool,
    pub log_level: String,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            vault_program_id: "Vault1111111111111111111111111111111111111111".to_string(),
            database: DatabaseConfig {
                host: "localhost".to_string(),
                port: 5432,
                database: "vaulta_indexer".to_string(),
                username: "postgres".to_string(),
                password: "postgres".to_string(),
                max_connections: 10,
                connection_timeout_seconds: 30,
            },
            redis: RedisConfig {
                url: "redis://localhost:6379".to_string(),
                ttl_seconds: 300,
                max_connections: 10,
                connection_timeout_seconds: 5,
            },
            batch_size: 1000,
            flush_interval_ms: 100,
            enable_cache: true,
            log_level: "info".to_string(),
        }
    }
}
