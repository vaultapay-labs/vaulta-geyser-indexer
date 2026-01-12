use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

/// Utility functions for the indexer

/// Parse a pubkey from string
pub fn parse_pubkey(s: &str) -> anyhow::Result<Pubkey> {
    Pubkey::from_str(s)
        .map_err(|e| anyhow::anyhow!("Invalid pubkey '{}': {}", s, e))
}

/// Format a pubkey for display
pub fn format_pubkey(pubkey: &Pubkey) -> String {
    pubkey.to_string()
}

/// Calculate latency in milliseconds
pub fn latency_ms(start: std::time::Instant) -> u64 {
    start.elapsed().as_millis() as u64
}

/// Format bytes as human-readable size
pub fn format_bytes(bytes: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    format!("{:.2} {}", size, UNITS[unit_index])
}
