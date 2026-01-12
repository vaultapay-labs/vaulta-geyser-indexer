use crate::types::{CacheEntry, VaultState};
use anyhow::Result;
use redis::AsyncCommands;
use std::time::Duration;
use time::OffsetDateTime;
use tracing::debug;

/// Redis cache for sub-100ms vault state queries
pub struct RedisCache {
    client: redis::Client,
    ttl_seconds: u64,
}

impl RedisCache {
    /// Create a new Redis cache client
    pub async fn new(url: &str, ttl_seconds: u64) -> Result<Self> {
        let client = redis::Client::open(url)?;
        
        // Test connection
        let mut conn = client.get_async_connection().await?;
        redis::cmd("PING").query_async::<_, String>(&mut conn).await?;
        
        Ok(Self {
            client,
            ttl_seconds,
        })
    }
    
    /// Get vault state from cache
    pub async fn get(&self, vault_address: &str) -> Result<Option<VaultState>> {
        let mut conn = self.client.get_async_connection().await?;
        
        let key = format!("vault:{}", vault_address);
        let data: Option<String> = conn.get(&key).await?;
        
        if let Some(data) = data {
            let entry: CacheEntry = serde_json::from_str(&data)?;
            
            // Check if expired
            let now = OffsetDateTime::now_utc();
            let age = (now - entry.cached_at).whole_seconds() as u64;
            
            if age < entry.ttl_seconds {
                debug!("Cache hit for vault: {}", vault_address);
                return Ok(Some(entry.vault_state));
            } else {
                debug!("Cache expired for vault: {}", vault_address);
                // Delete expired entry
                let _: () = conn.del(&key).await?;
            }
        }
        
        Ok(None)
    }
    
    /// Set vault state in cache
    pub async fn set(&self, state: &VaultState) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        
        let key = format!("vault:{}", state.vault_address);
        let entry = CacheEntry {
            vault_state: state.clone(),
            cached_at: OffsetDateTime::now_utc(),
            ttl_seconds: self.ttl_seconds,
        };
        
        let data = serde_json::to_string(&entry)?;
        conn.set_ex(&key, data, self.ttl_seconds as usize).await?;
        
        debug!("Cached vault state: {}", state.vault_address);
        Ok(())
    }
    
    /// Delete vault state from cache
    pub async fn delete(&self, vault_address: &str) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("vault:{}", vault_address);
        let _: () = conn.del(&key).await?;
        Ok(())
    }
    
    /// Batch set vault states
    pub async fn batch_set(&self, states: &[VaultState]) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        let mut pipe = redis::pipe();
        
        for state in states {
            let key = format!("vault:{}", state.vault_address);
            let entry = CacheEntry {
                vault_state: state.clone(),
                cached_at: OffsetDateTime::now_utc(),
                ttl_seconds: self.ttl_seconds,
            };
            let data = serde_json::to_string(&entry)?;
            pipe.set_ex(&key, data, self.ttl_seconds as usize);
        }
        
        pipe.query_async(&mut conn).await?;
        Ok(())
    }
    
    /// Get cache statistics
    pub async fn get_stats(&self) -> Result<CacheStats> {
        let mut conn = self.client.get_async_connection().await?;
        
        let info: String = redis::cmd("INFO")
            .arg("stats")
            .query_async(&mut conn)
            .await?;
        
        // Parse INFO output (simplified)
        let keyspace_hits = Self::parse_info_value(&info, "keyspace_hits")
            .unwrap_or(0);
        let keyspace_misses = Self::parse_info_value(&info, "keyspace_misses")
            .unwrap_or(0);
        
        let total = keyspace_hits + keyspace_misses;
        let hit_rate = if total > 0 {
            keyspace_hits as f64 / total as f64
        } else {
            0.0
        };
        
        Ok(CacheStats {
            keyspace_hits,
            keyspace_misses,
            hit_rate,
        })
    }
    
    fn parse_info_value(info: &str, key: &str) -> Option<u64> {
        for line in info.lines() {
            if line.starts_with(&format!("{}:", key)) {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() > 1 {
                    return parts[1].trim().parse().ok();
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub keyspace_hits: u64,
    pub keyspace_misses: u64,
    pub hit_rate: f64,
}
