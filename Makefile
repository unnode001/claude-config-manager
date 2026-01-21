.PHONY: help build test fmt clippy clean run install release

help: ## Show this help message
	@echo 'Claude Config Manager - Makefile'
	@echo ''
	@echo 'Usage:'
	@echo '  make <target>'
	@echo ''
	@echo 'Targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-20s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

build: ## Build all crates
	cargo build --workspace

build-release: ## Build release version
	cargo build --release --workspace

test: ## Run all tests
	cargo test --workspace

test-verbose: ## Run tests with output
	cargo test --workspace -- --nocapture

test-watch: ## Run tests in watch mode
	cargo test --workspace --watch

fmt: ## Format code
	cargo fmt --all

fmt-check: ## Check formatting
	cargo fmt --all -- --check

clippy: ## Run linter
	cargo clippy --workspace

clippy-fix: ## Fix linter warnings
	cargo clippy --fix --workspace --allow-dirty

clean: ## Clean build artifacts
	cargo clean

run: ## Run the CLI
	cargo run --bin ccm -- $(ARGS)

install: ## Install ccm locally
	cargo install --path .

release: VERSION ?= "local"
release: ## Build release binaries
	@echo "Building release $(VERSION)"
	@if [ "$(shell uname)" = "Linux" ]; then \
		./scripts/build.sh $(VERSION); \
	elif [ "$(shell uname)" = "Darwin" ]; then \
		./scripts/build.sh $(VERSION); \
	else \
		powershell -ExecutionPolicy Bypass -File scripts/build.ps1 -Version $(VERSION); \
	fi

check: fmt-check clippy test ## Run all checks

all: check build ## Run all checks and build

# Platform-specific release targets
release-all: ## Build for all platforms (requires cross-compilation)
	@echo "Cross-platform release requires special setup"
	@echo "See: https://github.com/johnthagen/min-sized-rust"
