# Copyright 2025 Cowboy AI, LLC.

.PHONY: all build test clean fmt lint doc coverage bench install-tools help

# Default target
all: fmt lint build test

# Build the project
build:
	@echo "Building..."
	@cargo build --release

# Run tests
test:
	@echo "Running tests..."
	@cargo test

# Clean build artifacts
clean:
	@echo "Cleaning..."
	@cargo clean
	@rm -rf target/

# Format code
fmt:
	@echo "Formatting code..."
	@cargo fmt

# Run linter
lint:
	@echo "Running clippy..."
	@cargo clippy -- -D warnings

# Generate documentation
doc:
	@echo "Generating documentation..."
	@cargo doc --no-deps --open

# Run code coverage
coverage:
	@echo "Running coverage..."
	@cargo tarpaulin --out Html

# Run benchmarks
bench:
	@echo "Running benchmarks..."
	@cargo bench

# Install development tools
install-tools:
	@echo "Installing development tools..."
	@cargo install cargo-tarpaulin
	@cargo install cargo-audit
	@cargo install cargo-outdated
	@cargo install cargo-edit

# Start NATS server
nats:
	@echo "Starting NATS server with JetStream..."
	@nats-server -js

# Help
help:
	@echo "Available targets:"
	@echo "  make build       - Build the project"
	@echo "  make test        - Run tests"
	@echo "  make clean       - Clean build artifacts"
	@echo "  make fmt         - Format code"
	@echo "  make lint        - Run clippy linter"
	@echo "  make doc         - Generate and open documentation"
	@echo "  make coverage    - Run code coverage"
	@echo "  make bench       - Run benchmarks"
	@echo "  make nats        - Start NATS server"
	@echo "  make install-tools - Install dev tools"
	@echo "  make help        - Show this help"