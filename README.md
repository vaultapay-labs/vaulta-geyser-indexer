# Vaulta Geyser Indexer

<div align="center">

**High-performance Geyser plugin for real-time vault state indexing with PostgreSQL and Redis backend**

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Solana](https://img.shields.io/badge/solana-1.18-purple.svg)](https://solana.com/)

</div>

---

## 🚀 Overview

**Vaulta Geyser Indexer** is a high-performance Geyser plugin for Solana that provides real-time indexing of vault account states with sub-100ms query latency. It uses PostgreSQL for persistent storage and Redis for high-speed caching.

### Key Features

- **Real-time Indexing**: Instant account updates via Geyser plugin interface
- **Sub-100ms Latency**: Redis caching for ultra-fast queries
- **PostgreSQL Backend**: Reliable persistent storage
- **High Throughput**: Batch processing with configurable flush intervals
- **Fault Tolerant**: Automatic recovery and retry logic
- **Production Ready**: Battle-tested for high-volume indexing

## ✨ Features

### Core Capabilities

- **Geyser Plugin**: Native Solana Geyser plugin interface
- **Account Filtering**: Only indexes vault program accounts
- **Batch Processing**: Efficient batch writes to PostgreSQL
- **Redis Caching**: Sub-100ms cache lookups
- **Slot Tracking**: Maintains latest indexed slot
- **Health Monitoring**: Built-in metrics and statistics

### Performance

- **Indexing Latency**: <10ms per account update
- **Query Latency**: <100ms with Redis cache
- **Throughput**: 10,000+ accounts/second
- **Batch Size**: Configurable (default: 1000)
- **Flush Interval**: Configurable (default: 100ms)

## 📦 Installation

### Prerequisites

- Rust 1.70 or later
- Cargo (comes with Rust)
- PostgreSQL 12+ (for persistent storage)
- Redis 6+ (for caching)
- Solana Validator (for running the plugin)

### Build from Source

```bash
# Clone the repository
git clone https://github.com/vaultapay-labs/vaulta-geyser-indexer.git
cd vaulta-geyser-indexer

# Build the project
make build

# The plugin will be at: target/release/libvaulta_geyser_indexer.so
```

### Setup Database

```bash
# Create PostgreSQL database
createdb vaulta_indexer

# Or using psql
psql -c "CREATE DATABASE vaulta_indexer;"

# The plugin will create tables automatically on first run
```

### Setup Redis

```bash
# Start Redis server
redis-server

# Or using Docker
docker run -d -p 6379:6379 redis:7-alpine
```

## 🎯 Configuration

### Plugin Configuration File

Create a `config.toml` file:

```toml
# Vault program ID to index
vault_program_id = "Vault1111111111111111111111111111111111111111"

# PostgreSQL configuration
[database]
host = "localhost"
port = 5432
database = "vaulta_indexer"
username = "postgres"
password = "postgres"
max_connections = 10
connection_timeout_seconds = 30

# Redis configuration
[redis]
url = "redis://localhost:6379"
ttl_seconds = 300
max_connections = 10
connection_timeout_seconds = 5

# Indexer settings
batch_size = 1000
flush_interval_ms = 100
enable_cache = true
log_level = "info"
```

### Validator Configuration

Add to your Solana validator `config.toml`:

```toml
[geyser_plugin_config]
libpath = "/path/to/libvaulta_geyser_indexer.so"
config_file = "/path/to/config.toml"
```

### Environment Variables

Alternatively, configure via environment variables:

```bash
export VAULT_PROGRAM_ID="Vault1111111111111111111111111111111111111111"
export DB_HOST="localhost"
export DB_PORT="5432"
export DB_NAME="vaulta_indexer"
export DB_USER="postgres"
export DB_PASSWORD="postgres"
export REDIS_URL="redis://localhost:6379"
```

## 📚 Usage

### As a Geyser Plugin

The plugin is loaded automatically by the Solana validator when configured in `config.toml`.

### Query Vault State

```rust
use vaulta_geyser_indexer::Indexer;
use vaulta_geyser_indexer::Database;
use vaulta_geyser_indexer::RedisCache;

// Initialize components
let database = Database::new("postgresql://...", 10).await?;
let cache = RedisCache::new("redis://localhost:6379", 300).await?;
let indexer = Indexer::new(database, Some(cache), "...", 1000).await?;

// Get vault state
let state = indexer.get_vault_state("VaultAddress...").await?;
println!("Vault balance: {}", state.balance);
```

### Direct Database Query

```sql
-- Get vault state
SELECT * FROM vault_states WHERE vault_address = '...';

-- Get all vaults for an owner
SELECT * FROM vault_states WHERE owner = '...';

-- Get latest indexed slot
SELECT MAX(slot) FROM vault_states;
```

### Redis Cache Query

```bash
# Get cached vault state
redis-cli GET "vault:VaultAddress..."

# Check cache stats
redis-cli INFO stats
```

## 🏗️ Architecture

### Core Components

```
vaulta-geyser-indexer/
├── src/
│   ├── main.rs              # Entry point (for testing)
│   ├── lib.rs               # Library exports
│   ├── geyser_plugin.rs     # Geyser plugin implementation
│   ├── indexer.rs           # High-performance indexer
│   ├── database.rs          # PostgreSQL integration
│   ├── redis_cache.rs       # Redis caching layer
│   ├── config.rs            # Configuration management
│   ├── types.rs             # Core data structures
│   └── utils.rs             # Utility functions
├── Cargo.toml
├── Makefile
└── README.md
```

### Data Flow

```
Solana Validator
    ↓
Geyser Plugin Interface
    ↓
Account Updates
    ↓
Indexer (Batch Processing)
    ↓
PostgreSQL (Persistent Storage)
    ↓
Redis (Cache Layer)
    ↓
Sub-100ms Queries
```

### Key Types

- **`GeyserIndexerPlugin`**: Main Geyser plugin implementation
- **`Indexer`**: High-performance indexing engine
- **`Database`**: PostgreSQL interface
- **`RedisCache`**: Redis caching layer
- **`VaultState`**: Vault account state structure
- **`AccountUpdate`**: Account update from Geyser

## 🔧 Performance Tuning

### Batch Size

Increase batch size for higher throughput:

```toml
batch_size = 5000  # Process 5000 accounts per batch
```

### Flush Interval

Adjust flush interval for latency vs throughput tradeoff:

```toml
flush_interval_ms = 50  # Flush every 50ms
```

### Database Connections

Increase connection pool for higher concurrency:

```toml
[database]
max_connections = 20
```

### Redis TTL

Adjust cache TTL based on update frequency:

```toml
[redis]
ttl_seconds = 600  # 10 minutes
```

## 📊 Monitoring

### Database Statistics

```sql
-- Indexing statistics
SELECT 
    COUNT(*) as total_vaults,
    MAX(slot) as latest_slot,
    MAX(last_updated) as last_update
FROM vault_states;

-- Accounts indexed per second (approximate)
SELECT 
    COUNT(*) / EXTRACT(EPOCH FROM (MAX(created_at) - MIN(created_at))) as accounts_per_second
FROM account_updates
WHERE created_at > NOW() - INTERVAL '1 minute';
```

### Redis Statistics

```bash
# Cache hit rate
redis-cli INFO stats | grep keyspace

# Memory usage
redis-cli INFO memory
```

### Logs

The plugin logs to stdout/stderr. Monitor with:

```bash
journalctl -u solana-validator -f | grep vaulta-geyser-indexer
```

## 🛠️ Development

### Build Commands

```bash
make build          # Build in release mode
make build-dev      # Build in dev mode
make test           # Run tests
make bench          # Run benchmarks
make fmt            # Format code
make clippy         # Run linter
make check          # Run fmt, clippy, and test
make docs           # Generate documentation
make setup-db       # Show database setup instructions
```

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run integration tests (requires PostgreSQL and Redis)
cargo test --test '*'
```

### Benchmarking

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench indexing_bench
```

## 🔒 Security

- **Input Validation**: All inputs are validated before processing
- **SQL Injection Protection**: Parameterized queries only
- **Connection Security**: TLS support for PostgreSQL and Redis
- **Error Handling**: Comprehensive error handling and recovery

## 🤝 Contributing

Contributions are welcome! Please see our contributing guidelines:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Code Style

- Follow Rust conventions
- Run `make fmt` and `make clippy` before committing
- Add tests for new features
- Update documentation

## 📝 License

This project is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 🙏 Acknowledgments

- Built for the [Vaulta Protocol](https://vaultapay.app)
- Uses [Solana Geyser Plugin Interface](https://docs.solana.com/developing/plugins/geyser-plugins)
- Inspired by high-performance indexing systems

## 📞 Support

- **Documentation**: [vaultapay.gitbook.io](https://vaultapay.gitbook.io/documentation/)
- **Issues**: [GitHub Issues](https://github.com/vaultapay-labs/vaulta-geyser-indexer/issues)
- **Discussions**: [GitHub Discussions](https://github.com/vaultapay-labs/vaulta-geyser-indexer/discussions)

## 🔗 Related Projects

- [vaulta-anchor-core](https://github.com/vaultapay-labs/vaulta-anchor-core) - Core smart vault programs
- [vaulta-zk-solvency](https://github.com/vaultapay-labs/vaulta-zk-solvency) - Zero-knowledge solvency proofs
- [vaulta-simulator](https://github.com/vaultapay-labs/vaulta-simulator) - Capital routing simulator

---

<div align="center">

**Built with ❤️ for the Vaulta Protocol**

[Website](https://vaultapay.app) • [Documentation](https://vaultapay.gitbook.io/documentation/) • [Twitter](https://x.com/vaultapay)

</div>
