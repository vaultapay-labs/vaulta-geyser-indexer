use crate::config::Config;
use crate::indexer::Indexer;
use crate::types::AccountUpdate;
use anyhow::Result;
use solana_geyser_plugin_interface::geyser_plugin_interface::{
    GeyserPlugin, ReplicaAccountInfo, ReplicaAccountInfoVersions, Result as GeyserResult,
};
use std::sync::{Arc, Mutex};
use tracing::{error, info, warn};

/// Geyser plugin implementation
pub struct GeyserIndexerPlugin {
    indexer: Arc<Mutex<Option<Arc<Indexer>>>>,
    config: Arc<Mutex<Option<Config>>>,
}

impl GeyserIndexerPlugin {
    pub fn new() -> Self {
        Self {
            indexer: Arc::new(Mutex::new(None)),
            config: Arc::new(Mutex::new(None)),
        }
    }
}

unsafe impl Send for GeyserIndexerPlugin {}
unsafe impl Sync for GeyserIndexerPlugin {}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn _create_plugin() -> *mut dyn GeyserPlugin {
    let plugin = GeyserIndexerPlugin::new();
    let plugin: Box<dyn GeyserPlugin> = Box::new(plugin);
    Box::into_raw(plugin)
}

impl GeyserPlugin for GeyserIndexerPlugin {
    fn name(&self) -> &'static str {
        "vaulta-geyser-indexer"
    }
    
    fn on_load(&mut self, config_file: &str) -> GeyserResult<()> {
        info!("Loading Vaulta Geyser Indexer plugin...");
        
        // Load configuration
        let config = match Config::from_file(config_file) {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to load config file, using environment variables: {}", e);
                Config::from_env().map_err(|e| {
                    error!("Failed to load config: {}", e);
                    solana_geyser_plugin_interface::geyser_plugin_interface::GeyserPluginError::ConfigFileReadError {
                        msg: format!("Failed to load config: {}", e),
                    }
                })?
            }
        };
        
        let config_inner = config.inner().clone();
        
        // Initialize indexer asynchronously
        let indexer_arc = self.indexer.clone();
        let config_arc = self.config.clone();
        
        tokio::spawn(async move {
            match initialize_indexer(&config_inner).await {
                Ok(indexer) => {
                    *indexer_arc.lock().unwrap() = Some(Arc::new(indexer));
                    *config_arc.lock().unwrap() = Some(config);
                    info!("Vaulta Geyser Indexer initialized successfully");
                }
                Err(e) => {
                    error!("Failed to initialize indexer: {}", e);
                }
            }
        });
        
        Ok(())
    }
    
    fn on_unload(&mut self) {
        info!("Unloading Vaulta Geyser Indexer plugin");
        *self.indexer.lock().unwrap() = None;
        *self.config.lock().unwrap() = None;
    }
    
    fn update_account(
        &mut self,
        account: ReplicaAccountInfoVersions,
        slot: u64,
        is_startup: bool,
    ) -> GeyserResult<()> {
        let indexer_guard = self.indexer.lock().unwrap();
        
        if let Some(indexer) = indexer_guard.as_ref() {
            let account_update = match convert_account_info(account, slot, is_startup) {
                Ok(update) => update,
                Err(e) => {
                    error!("Failed to convert account info: {}", e);
                    return Ok(());
                }
            };
            
            if let Err(e) = indexer.process_update(account_update) {
                error!("Failed to process account update: {}", e);
            }
        } else {
            // Indexer not initialized yet, skip
        }
        
        Ok(())
    }
    
    fn notify_end_of_startup(&mut self) -> GeyserResult<()> {
        info!("Startup complete, switching to real-time indexing mode");
        Ok(())
    }
}

/// Initialize indexer with configuration
async fn initialize_indexer(config: &crate::types::PluginConfig) -> Result<Indexer> {
    use crate::database::Database;
    use crate::redis_cache::RedisCache;
    
    // Initialize database
    let db_conn_string = format!(
        "postgresql://{}:{}@{}:{}/{}",
        config.database.username,
        config.database.password,
        config.database.host,
        config.database.port,
        config.database.database,
    );
    
    let database = Database::new(
        &db_conn_string,
        config.database.max_connections,
    ).await?;
    
    // Initialize Redis cache if enabled
    let cache = if config.enable_cache {
        Some(RedisCache::new(
            &config.redis.url,
            config.redis.ttl_seconds,
        ).await?)
    } else {
        None
    };
    
    // Create indexer
    let indexer = Indexer::new(
        database,
        cache,
        &config.vault_program_id,
        config.batch_size,
    ).await?;
    
    Ok(indexer)
}

/// Convert Geyser account info to our AccountUpdate type
fn convert_account_info(
    account: ReplicaAccountInfoVersions,
    slot: u64,
    is_startup: bool,
) -> Result<AccountUpdate> {
    let account_info = match account {
        ReplicaAccountInfoVersions::V0_0_1(info) => info,
        ReplicaAccountInfoVersions::V0_0_2(info) => info,
    };
    
    Ok(AccountUpdate {
        pubkey: *account_info.pubkey,
        lamports: account_info.lamports,
        owner: *account_info.owner,
        executable: account_info.executable,
        rent_epoch: account_info.rent_epoch,
        data: account_info.data.to_vec(),
        write_version: account_info.write_version,
        slot,
        is_startup,
    })
}
