use crate::types::PluginConfig;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub struct Config {
    inner: PluginConfig,
}

impl Config {
    /// Load configuration from file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read config file: {:?}", path.as_ref()))?;
        
        let config: PluginConfig = toml::from_str(&content)
            .context("Failed to parse config file")?;
        
        Ok(Self { inner: config })
    }
    
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        let mut config = PluginConfig::default();
        
        // Override with environment variables
        if let Ok(program_id) = std::env::var("VAULT_PROGRAM_ID") {
            config.vault_program_id = program_id;
        }
        
        if let Ok(host) = std::env::var("DB_HOST") {
            config.database.host = host;
        }
        
        if let Ok(port) = std::env::var("DB_PORT") {
            config.database.port = port.parse().unwrap_or(5432);
        }
        
        if let Ok(database) = std::env::var("DB_NAME") {
            config.database.database = database;
        }
        
        if let Ok(username) = std::env::var("DB_USER") {
            config.database.username = username;
        }
        
        if let Ok(password) = std::env::var("DB_PASSWORD") {
            config.database.password = password;
        }
        
        if let Ok(redis_url) = std::env::var("REDIS_URL") {
            config.redis.url = redis_url;
        }
        
        Ok(Self { inner: config })
    }
    
    pub fn inner(&self) -> &PluginConfig {
        &self.inner
    }
}
