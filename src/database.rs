use crate::types::VaultState;
use std::str::FromStr;
use anyhow::Result;
use sqlx::PgPool;
use std::collections::HashMap;
use time::OffsetDateTime;

/// PostgreSQL database interface
pub struct Database {
    pool: PgPool,
}

impl Database {
    /// Create a new database connection pool
    pub async fn new(connection_string: &str, max_connections: u32) -> Result<Self> {
        let options = sqlx::postgres::PgConnectOptions::from_str(connection_string)?;
        let pool = PgPool::connect_with(options).await?;
        
        // Initialize schema
        Self::init_schema(&pool).await?;
        
        Ok(Self { pool })
    }
    
    /// Initialize database schema
    async fn init_schema(pool: &PgPool) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS vault_states (
                vault_address TEXT PRIMARY KEY,
                owner TEXT NOT NULL,
                balance BIGINT NOT NULL,
                assets JSONB NOT NULL DEFAULT '{}',
                permissions JSONB NOT NULL DEFAULT '[]',
                last_updated TIMESTAMPTZ NOT NULL,
                slot BIGINT NOT NULL,
                write_version BIGINT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            
            CREATE INDEX IF NOT EXISTS idx_vault_states_owner ON vault_states(owner);
            CREATE INDEX IF NOT EXISTS idx_vault_states_slot ON vault_states(slot);
            CREATE INDEX IF NOT EXISTS idx_vault_states_updated ON vault_states(last_updated);
            
            CREATE TABLE IF NOT EXISTS account_updates (
                id BIGSERIAL PRIMARY KEY,
                pubkey TEXT NOT NULL,
                slot BIGINT NOT NULL,
                write_version BIGINT NOT NULL,
                data BYTEA,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            
            CREATE INDEX IF NOT EXISTS idx_account_updates_pubkey ON account_updates(pubkey);
            CREATE INDEX IF NOT EXISTS idx_account_updates_slot ON account_updates(slot);
            "#
        )
        .execute(pool)
        .await?;
        
        Ok(())
    }
    
    /// Upsert vault state
    pub async fn upsert_vault_state(&self, state: &VaultState) -> Result<()> {
        let assets_json = serde_json::to_string(&state.assets)?;
        let permissions_json = serde_json::to_string(&state.permissions)?;
        
        sqlx::query(
            r#"
            INSERT INTO vault_states (
                vault_address, owner, balance, assets, permissions,
                last_updated, slot, write_version, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())
            ON CONFLICT (vault_address) DO UPDATE SET
                owner = EXCLUDED.owner,
                balance = EXCLUDED.balance,
                assets = EXCLUDED.assets,
                permissions = EXCLUDED.permissions,
                last_updated = EXCLUDED.last_updated,
                slot = EXCLUDED.slot,
                write_version = EXCLUDED.write_version,
                updated_at = NOW()
            "#
        )
        .bind(state.vault_address.to_string())
        .bind(state.owner.to_string())
        .bind(state.balance as i64)
        .bind(assets_json)
        .bind(permissions_json)
        .bind(state.last_updated)
        .bind(state.slot as i64)
        .bind(state.write_version as i64)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    /// Get vault state by address
    pub async fn get_vault_state(&self, vault_address: &str) -> Result<Option<VaultState>> {
        let row = sqlx::query(
            r#"
            SELECT vault_address, owner, balance, assets, permissions,
                   last_updated, slot, write_version
            FROM vault_states
            WHERE vault_address = $1
            "#
        )
        .bind(vault_address)
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(row) = row {
            let vault_address_str: String = row.try_get(0)?;
            let owner_str: String = row.try_get(1)?;
            let balance: i64 = row.try_get(2)?;
            let assets_json: String = row.try_get(3)?;
            let permissions_json: String = row.try_get(4)?;
            let last_updated: OffsetDateTime = row.try_get(5)?;
            let slot: i64 = row.try_get(6)?;
            let write_version: i64 = row.try_get(7)?;
            
            let vault_address: Pubkey = vault_address_str.parse()?;
            let owner: Pubkey = owner_str.parse()?;
            
            let assets: HashMap<String, AssetBalance> = serde_json::from_str(&assets_json)?;
            let permissions: Vec<Permission> = serde_json::from_str(&permissions_json)?;
            
            Ok(Some(VaultState {
                vault_address,
                owner,
                balance: balance as u64,
                assets,
                permissions,
                last_updated,
                slot: slot as u64,
                write_version: write_version as u64,
            }))
        } else {
            Ok(None)
        }
    }
    
    /// Batch upsert vault states
    pub async fn batch_upsert_vault_states(&self, states: &[VaultState]) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        
        for state in states {
            let assets_json = serde_json::to_string(&state.assets)?;
            let permissions_json = serde_json::to_string(&state.permissions)?;
            
            sqlx::query(
                r#"
                INSERT INTO vault_states (
                    vault_address, owner, balance, assets, permissions,
                    last_updated, slot, write_version, updated_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())
                ON CONFLICT (vault_address) DO UPDATE SET
                    owner = EXCLUDED.owner,
                    balance = EXCLUDED.balance,
                    assets = EXCLUDED.assets,
                    permissions = EXCLUDED.permissions,
                    last_updated = EXCLUDED.last_updated,
                    slot = EXCLUDED.slot,
                    write_version = EXCLUDED.write_version,
                    updated_at = NOW()
                "#
            )
            .bind(state.vault_address.to_string())
            .bind(state.owner.to_string())
            .bind(state.balance as i64)
            .bind(assets_json)
            .bind(permissions_json)
            .bind(state.last_updated)
            .bind(state.slot as i64)
            .bind(state.write_version as i64)
            .execute(&mut *tx)
            .await?;
        }
        
        tx.commit().await?;
        Ok(())
    }
    
    /// Get latest slot indexed
    pub async fn get_latest_slot(&self) -> Result<u64> {
        let row = sqlx::query("SELECT COALESCE(MAX(slot), 0) FROM vault_states")
            .fetch_one(&self.pool)
            .await?;
        
        let slot: i64 = row.try_get(0)?;
        Ok(slot as u64)
    }
}
