{
  description = "CIM-IPLD: Content-addressed storage for the Composable Information Machine";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "clippy" "rustfmt" "rust-analyzer" ];
        };

        nativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
          cargo-nextest
          cargo-watch
          openssl.dev
        ];

        buildInputs = with pkgs; [
          openssl
          zstd
          protobuf
        ] ++ lib.optionals stdenv.isDarwin [
          darwin.apple_sdk.frameworks.SystemConfiguration
          darwin.apple_sdk.frameworks.Security
        ];
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "cim-ipld";
          version = "0.3.0";
          src = pkgs.lib.cleanSourceWith {
            src = ./.;
            filter = path: type:
              let baseName = baseNameOf path;
              in !(baseName == "target" && type == "directory") &&
                 !(baseName == ".direnv" && type == "directory") &&
                 !(baseName == "result") &&
                 !(baseName == ".git" && type == "directory") &&
                 !(baseName == "flake.lock") &&
                 !(baseName == ".gitignore");
          };
          
          cargoLock.lockFile = ./Cargo.lock;
          
          inherit nativeBuildInputs buildInputs;
          
          doCheck = true;
          checkFlags = [
            # Skip tests that require NATS to be running
            "--skip=content_types::persistence::tests::test_encryption_wrapper"
          ];
        };

        devShells.default = pkgs.mkShell {
          inherit buildInputs;
          nativeBuildInputs = nativeBuildInputs ++ (with pkgs; [
            # Rust development tools
            rust-analyzer
            cargo-edit
            cargo-audit
            cargo-outdated
            cargo-tarpaulin
            cargo-llvm-cov
            cargo-machete
            cargo-deny
            cargo-expand
            cargo-bloat
            cargo-udeps
            
            # Search and file tools
            ripgrep
            fd
            bat
            eza
            tokei
            
            # Development utilities
            hyperfine
            just
            watchexec
            git
            gh
            jq
            yq
            
            # NATS tools
            natscli
            nats-server
            
            # Documentation tools
            mdbook
            mdbook-mermaid
            mdbook-plantuml
            plantuml
            
            # Code quality tools
            typos
            shellcheck
            
            # Performance analysis
            flamegraph
            perf-tools
            heaptrack
            valgrind
          ]);

          shellHook = ''
            echo "🚀 CIM-IPLD Development Environment"
            echo ""
            echo "Key tools available:"
            echo "  • cargo-tarpaulin    - Code coverage analysis"
            echo "  • cargo-llvm-cov     - Alternative coverage tool"
            echo "  • ripgrep (rg)       - Fast code search"
            echo "  • cargo-watch        - Auto-rebuild on changes"
            echo "  • NATS CLI & server  - For testing NATS integration"
            echo "  • flamegraph         - Performance profiling"
            echo ""
            echo "Useful commands:"
            echo "  cargo test                      # Run all tests"
            echo "  cargo tarpaulin                 # Generate coverage report"
            echo "  cargo tarpaulin --out Html      # Generate HTML coverage report"
            echo "  cargo llvm-cov --html           # Alternative coverage with LLVM"
            echo "  cargo watch -x test             # Auto-run tests on changes"
            echo "  cargo audit                     # Security audit"
            echo "  cargo outdated                  # Check for outdated dependencies"
            echo "  cargo bloat                     # Analyze binary size"
            echo "  just test-coverage              # Run tests with coverage (if justfile exists)"
            echo "  rg 'pattern' src/               # Search code with ripgrep"
            echo "  nats-server -js                 # Start NATS with JetStream"
            echo ""
            
            # Set up cargo aliases for convenience
            mkdir -p .cargo
            cat > .cargo/config.toml << 'EOF'
[alias]
# Coverage commands
cov = "tarpaulin --out Html --output-dir target/coverage"
cov-ci = "tarpaulin --out Xml --output-dir target/coverage"
cov-lcov = "tarpaulin --out Lcov --output-dir target/coverage"
llvm-cov = "llvm-cov --html --output-dir target/llvm-coverage"
llvm-cov-lcov = "llvm-cov --lcov --output-path target/llvm-coverage/lcov.info"

# Development commands
wt = "watch -x test"
wc = "watch -x check"
wr = "watch -x 'run --example'"

# Maintenance commands
bloat = "bloat --release --crates"
unused = "machete"
EOF

            # Create justfile if it doesn't exist
            if [ ! -f "justfile" ]; then
              cat > justfile << 'EOF'
# List available commands
default:
    @just --list

# Run all tests
test:
    cargo test --all-features

# Run tests with coverage report
test-coverage:
    cargo tarpaulin --out Html --output-dir target/coverage
    @echo "Coverage report generated at target/coverage/tarpaulin-report.html"

# Run tests with LLVM coverage
test-coverage-llvm:
    cargo llvm-cov --html --output-dir target/llvm-coverage
    @echo "Coverage report generated at target/llvm-coverage/html/index.html"

# Run benchmarks
bench:
    cargo bench

# Check code with clippy
lint:
    cargo clippy -- -D warnings

# Format code
fmt:
    cargo fmt

# Run security audit
audit:
    cargo audit

# Check for outdated dependencies
outdated:
    cargo outdated

# Start NATS server for testing
nats:
    nats-server -js -sd ./target/nats-data

# Clean build artifacts
clean:
    cargo clean
    rm -rf target/coverage target/llvm-coverage

# Run example with NATS
run-example example:
    cargo run --example {{example}}

# Search for pattern in code
search pattern:
    rg "{{pattern}}" src/

# Count lines of code
loc:
    tokei src/
EOF
              echo "Created justfile with common commands"
            fi
          '';

          RUST_BACKTRACE = 1;
          RUST_LOG = "debug";
          
          # For OpenSSL
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
          
          # NATS server data directory
          NATS_DATA_DIR = "./target/nats-data";
        };

        apps.test = flake-utils.lib.mkApp {
          drv = pkgs.writeShellScriptBin "test-cim-ipld" ''
            cargo test --all-features
          '';
        };

        apps.bench = flake-utils.lib.mkApp {
          drv = pkgs.writeShellScriptBin "bench-cim-ipld" ''
            cargo bench
          '';
        };

        apps.coverage = flake-utils.lib.mkApp {
          drv = pkgs.writeShellScriptBin "coverage-cim-ipld" ''
            cargo tarpaulin --out Html --output-dir target/coverage
            echo "Coverage report generated at target/coverage/tarpaulin-report.html"
          '';
        };
      });
}