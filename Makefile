# Vaulta Geyser Indexer - Build and Test Automation
# High-performance Geyser plugin for real-time vault state indexing

.PHONY: all build test bench run clean fmt clippy docs install help setup-db

# Default target
all: fmt clippy build

# ---------------------------------------------------------------------------
# Build Commands
# ---------------------------------------------------------------------------

build:
	@echo "  [Cargo] Building vaulta-geyser-indexer in release mode..."
	@cargo build --release
	@echo "  [✓] Build complete! Plugin: target/release/libvaulta_geyser_indexer.so"

build-dev:
	@echo "  [Cargo] Building vaulta-geyser-indexer in dev mode..."
	@cargo build
	@echo "  [✓] Dev build complete! Plugin: target/debug/libvaulta_geyser_indexer.so"

# ---------------------------------------------------------------------------
# Testing Commands
# ---------------------------------------------------------------------------

test: build-dev
	@echo "  [Test] Running test suite..."
	@cargo test --lib --tests -- --nocapture
	@echo "  [✓] All tests passed!"

test-verbose:
	@echo "  [Test] Running test suite with verbose output..."
	@cargo test --lib --tests -- --nocapture --test-threads=1

test-integration:
	@echo "  [Test] Running integration tests..."
	@cargo test --test '*' -- --nocapture

# ---------------------------------------------------------------------------
# Benchmarking
# ---------------------------------------------------------------------------

bench:
	@echo "  [Bench] Running benchmarks..."
	@cargo bench -- --output-format html
	@echo "  [✓] Benchmarks complete! Check target/criterion/ for reports"

bench-quick:
	@echo "  [Bench] Running quick benchmarks..."
	@cargo bench -- --quick

# ---------------------------------------------------------------------------
# Code Quality
# ---------------------------------------------------------------------------

fmt:
	@echo "  [Fmt] Formatting code..."
	@cargo fmt --all
	@echo "  [✓] Code formatted!"

clippy:
	@echo "  [Clippy] Running linter..."
	@cargo clippy --all-targets --all-features -- -D warnings
	@echo "  [✓] Linting passed!"

check: fmt clippy test
	@echo "  [✓] All checks passed!"

# ---------------------------------------------------------------------------
# Documentation
# ---------------------------------------------------------------------------

docs:
	@echo "  [Docs] Generating documentation..."
	@cargo doc --no-deps --open
	@echo "  [✓] Documentation generated!"

docs-build:
	@echo "  [Docs] Building documentation..."
	@cargo doc --no-deps
	@echo "  [✓] Documentation built in target/doc/"

# ---------------------------------------------------------------------------
# Database Setup
# ---------------------------------------------------------------------------

setup-db:
	@echo "  [DB] Setting up PostgreSQL database..."
	@echo "  Please ensure PostgreSQL is running and create database:"
	@echo "  CREATE DATABASE vaulta_indexer;"
	@echo "  The plugin will create tables automatically on first run"

# ---------------------------------------------------------------------------
# Installation
# ---------------------------------------------------------------------------

install: build
	@echo "  [Install] Plugin built successfully!"
	@echo "  [Install] Copy target/release/libvaulta_geyser_indexer.so to your Solana validator plugins directory"
	@echo "  [Install] Configure in validator config file (see README.md)"

# ---------------------------------------------------------------------------
# Cleanup
# ---------------------------------------------------------------------------

clean:
	@echo "  [Clean] Removing build artifacts..."
	@cargo clean
	@rm -rf target/
	@echo "  [✓] Clean complete!"

clean-all: clean
	@echo "  [Clean] Removing all generated files..."
	@rm -rf results/
	@rm -rf logs/
	@echo "  [✓] Deep clean complete!"

# ---------------------------------------------------------------------------
# Development Workflow
# ---------------------------------------------------------------------------

dev: fmt clippy test build-dev
	@echo "  [✓] Development build ready!"

ci: fmt clippy test bench
	@echo "  [✓] CI checks complete!"

# ---------------------------------------------------------------------------
# Help
# ---------------------------------------------------------------------------

help:
	@echo "Vaulta Geyser Indexer - Build System"
	@echo ""
	@echo "Available targets:"
	@echo "  make build          - Build in release mode"
	@echo "  make build-dev      - Build in dev mode"
	@echo "  make test           - Run test suite"
	@echo "  make bench          - Run benchmarks"
	@echo "  make fmt            - Format code"
	@echo "  make clippy         - Run linter"
	@echo "  make check          - Run fmt, clippy, and test"
	@echo "  make docs           - Generate and open documentation"
	@echo "  make setup-db       - Show database setup instructions"
	@echo "  make install        - Build plugin (copy to validator)"
	@echo "  make clean          - Remove build artifacts"
	@echo "  make dev            - Full dev workflow (fmt, clippy, test, build)"
	@echo "  make ci             - CI workflow (fmt, clippy, test, bench)"
	@echo "  make help           - Show this help message"
