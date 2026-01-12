use crate::database::Database;
use crate::redis_cache::RedisCache;
use crate::types::{AccountUpdate, AssetBalance, VaultState};
use anyhow::Result;
use solana_sdk::pubkey::{Pubkey, PubkeyError};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use time::OffsetDateTime;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info};

/// High-performance indexer for vault state
pub struct Indexer {
    database: Arc<Database>,
    cache: Option<Arc<RedisCache>>,
    vault_program_id: Pubkey,
    batch_size: usize,
    update_tx: mpsc::UnboundedSender<AccountUpdate>,
}

impl Indexer {
    /// Create a new indexer
    pub async fn new(
        database: Database,
        cache: Option<RedisCache>,
        vault_program_id: &str,
        batch_size: usize,
    ) -> Result<Self> {
        let vault_program_id = Pubkey::from_str(vault_program_id)?;
        
        let (tx, mut rx) = mpsc::unbounded_channel();
        
        let db = Arc::new(database);
        let cache_arc = cache.map(Arc::new);
        
        // Spawn indexing task
        let db_clone = db.clone();
        let cache_clone = cache_arc.clone();
        
        tokio::spawn(async move {
            let mut batch = Vec::new();
            let mut flush_interval = interval(Duration::from_millis(100));
            
            loop {
                tokio::select! {
                    update = rx.recv() => {
                        if let Some(update) = update {
                            batch.push(update);
                            
                            if batch.len() >= batch_size {
                                if let Err(e) = Self::process_batch(
                                    &db_clone,
                                    cache_clone.as_ref(),
                                    &batch,
                                ).await {
                                    error!("Error processing batch: {}", e);
                                }
                                batch.clear();
                            }
                        } else {
                            // Channel closed
                            break;
                        }
                    }
                    _ = flush_interval.tick() => {
                        if !batch.is_empty() {
                            if let Err(e) = Self::process_batch(
                                &db_clone,
                                cache_clone.as_ref(),
                                &batch,
                            ).await {
                                error!("Error processing batch: {}", e);
                            }
                            batch.clear();
                        }
                    }
                }
            }
        });
        
        Ok(Self {
            database: db,
            cache: cache_arc,
            vault_program_id,
            batch_size,
            update_tx: tx,
        })
    }
    
    /// Process account update
    pub fn process_update(&self, update: AccountUpdate) -> Result<()> {
        // Check if this is a vault account
        if update.owner != self.vault_program_id {
            return Ok(()); // Not a vault account, skip
        }
        
        self.update_tx.send(update)
            .map_err(|e| anyhow::anyhow!("Failed to send update: {}", e))?;
        
        Ok(())
    }
    
    /// Process batch of updates
    async fn process_batch(
        database: &Database,
        cache: Option<&RedisCache>,
        updates: &[AccountUpdate],
    ) -> Result<()> {
        let start = std::time::Instant::now();
        
        let mut vault_states = Vec::new();
        
        for update in updates {
            // Parse vault state from account data
            if let Some(state) = Self::parse_vault_state(update)? {
                vault_states.push(state);
            }
        }
        
        if vault_states.is_empty() {
            return Ok(());
        }
        
        // Write to database
        database.batch_upsert_vault_states(&vault_states).await?;
        
        // Update cache
        if let Some(cache) = cache {
            cache.batch_set(&vault_states).await?;
        }
        
        let elapsed = start.elapsed();
        debug!("Processed {} vault states in {:?}", vault_states.len(), elapsed);
        
        Ok(())
    }
    
    /// Parse vault state from account update
    fn parse_vault_state(update: &AccountUpdate) -> Result<Option<VaultState>> {
        // In a real implementation, we'd parse the account data according to
        // the vault program's account structure
        // This is a simplified version
        
        if update.data.len() < 32 {
            return Ok(None);
        }
        
        // Extract owner (first 32 bytes)
        let owner_bytes: [u8; 32] = update.data[0..32]
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid owner pubkey length"))?;
        let owner = Pubkey::from(owner_bytes);
        
        // Extract balance (next 8 bytes)
        let balance = if update.data.len() >= 40 {
            u64::from_le_bytes(
                update.data[32..40].try_into()
                    .map_err(|_| anyhow::anyhow!("Invalid balance"))?
            )
        } else {
            update.lamports
        };
        
        // Parse assets and permissions from remaining data
        // This is simplified - real implementation would deserialize properly
        let assets = HashMap::new();
        let permissions = Vec::new();
        
        let state = VaultState {
            vault_address: update.pubkey,
            owner,
            balance,
            assets,
            permissions,
            last_updated: OffsetDateTime::now_utc(),
            slot: update.slot,
            write_version: update.write_version,
        };
        
        Ok(Some(state))
    }
    
    /// Get vault state (with cache lookup)
    pub async fn get_vault_state(&self, vault_address: &str) -> Result<Option<VaultState>> {
        // Try cache first
        if let Some(cache) = &self.cache {
            if let Some(state) = cache.get(vault_address).await? {
                return Ok(Some(state));
            }
        }
        
        // Fallback to database
        let state = self.database.get_vault_state(vault_address).await?;
        
        // Update cache if found
        if let Some(ref state) = state {
            if let Some(cache) = &self.cache {
                cache.set(state).await?;
            }
        }
        
        Ok(state)
    }
}
