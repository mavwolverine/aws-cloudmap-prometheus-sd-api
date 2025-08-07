# AWS Cloud Map Prometheus Service Discovery API - Makefile
# 
# This Makefile provides common development and build tasks for the Rust project.
# Run 'make help' to see all available targets.

# Project configuration
PROJECT_NAME := aws-cloudmap-prometheus-sd-api
BINARY_NAME := aws-cloudmap-prometheus-sd-api
VERSION ?= $(shell git describe --tags --always --dirty 2>/dev/null || echo "dev")
GIT_COMMIT ?= $(shell git rev-parse HEAD 2>/dev/null || echo "unknown")
BUILD_DATE ?= $(shell date -u +"%Y-%m-%dT%H:%M:%SZ")

# Docker configuration
IMAGE_NAME := $(PROJECT_NAME)
REGISTRY ?= 
TAG ?= latest

# Directories
TARGET_DIR := target
RELEASE_DIR := $(TARGET_DIR)/release
DEBUG_DIR := $(TARGET_DIR)/debug
DOC_DIR := $(TARGET_DIR)/doc

# Colors for output
RED := \033[0;31m
GREEN := \033[0;32m
YELLOW := \033[0;33m
BLUE := \033[0;34m
NC := \033[0m # No Color

.PHONY: help
help: ## Show this help message
	@echo "$(BLUE)$(PROJECT_NAME) - Available targets:$(NC)"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(GREEN)%-20s$(NC) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(YELLOW)Environment variables:$(NC)"
	@echo "  VERSION     - Version tag (default: git describe)"
	@echo "  REGISTRY    - Docker registry prefix"
	@echo "  TAG         - Docker image tag (default: latest)"
	@echo ""

.PHONY: build
build: ## Build the project in debug mode
	@echo "$(BLUE)Building $(PROJECT_NAME) (debug)...$(NC)"
	cargo build
	@echo "$(GREEN)✓ Debug build complete$(NC)"

.PHONY: build-release
build-release: ## Build the project in release mode (optimized)
	@echo "$(BLUE)Building $(PROJECT_NAME) (release)...$(NC)"
	cargo build --release
	@echo "$(GREEN)✓ Release build complete: $(RELEASE_DIR)/$(BINARY_NAME)$(NC)"

.PHONY: test
test: ## Run all tests
	@echo "$(BLUE)Running tests...$(NC)"
	cargo test
	@echo "$(GREEN)✓ All tests passed$(NC)"

.PHONY: test-verbose
test-verbose: ## Run tests with verbose output
	@echo "$(BLUE)Running tests (verbose)...$(NC)"
	cargo test -- --nocapture
	@echo "$(GREEN)✓ All tests passed$(NC)"

.PHONY: test-coverage
test-coverage: ## Run tests with coverage report (requires cargo-tarpaulin)
	@echo "$(BLUE)Running tests with coverage...$(NC)"
	@if ! command -v cargo-tarpaulin >/dev/null 2>&1; then \
		echo "$(YELLOW)Installing cargo-tarpaulin...$(NC)"; \
		cargo install cargo-tarpaulin; \
	fi
	cargo tarpaulin --out Html --output-dir coverage
	@echo "$(GREEN)✓ Coverage report generated in coverage/$(NC)"

.PHONY: bench
bench: ## Run benchmarks
	@echo "$(BLUE)Running benchmarks...$(NC)"
	cargo bench
	@echo "$(GREEN)✓ Benchmarks complete$(NC)"

.PHONY: check
check: ## Check code without building
	@echo "$(BLUE)Checking code...$(NC)"
	cargo check
	@echo "$(GREEN)✓ Code check passed$(NC)"

.PHONY: clippy
clippy: ## Run clippy linter
	@echo "$(BLUE)Running clippy...$(NC)"
	cargo clippy -- -D warnings
	@echo "$(GREEN)✓ Clippy checks passed$(NC)"

.PHONY: fmt
fmt: ## Format code using rustfmt
	@echo "$(BLUE)Formatting code...$(NC)"
	cargo fmt
	@echo "$(GREEN)✓ Code formatted$(NC)"

.PHONY: fmt-check
fmt-check: ## Check if code is formatted correctly
	@echo "$(BLUE)Checking code formatting...$(NC)"
	cargo fmt -- --check
	@echo "$(GREEN)✓ Code formatting is correct$(NC)"

.PHONY: doc
doc: ## Generate documentation
	@echo "$(BLUE)Generating documentation...$(NC)"
	cargo doc --no-deps
	@echo "$(GREEN)✓ Documentation generated in $(DOC_DIR)/$(NC)"

.PHONY: doc-open
doc-open: ## Generate and open documentation in browser
	@echo "$(BLUE)Generating and opening documentation...$(NC)"
	cargo doc --open --no-deps
	@echo "$(GREEN)✓ Documentation opened in browser$(NC)"

.PHONY: run
run: ## Run the application in debug mode
	@echo "$(BLUE)Running $(PROJECT_NAME)...$(NC)"
	cargo run

.PHONY: run-release
run-release: build-release ## Run the application in release mode
	@echo "$(BLUE)Running $(PROJECT_NAME) (release)...$(NC)"
	./$(RELEASE_DIR)/$(BINARY_NAME)

.PHONY: install
install: build-release ## Install the binary to ~/.cargo/bin
	@echo "$(BLUE)Installing $(PROJECT_NAME)...$(NC)"
	cargo install --path .
	@echo "$(GREEN)✓ $(PROJECT_NAME) installed$(NC)"

.PHONY: clean
clean: ## Clean build artifacts
	@echo "$(BLUE)Cleaning build artifacts...$(NC)"
	cargo clean
	rm -rf coverage/
	@echo "$(GREEN)✓ Clean complete$(NC)"

.PHONY: deps
deps: ## Update dependencies
	@echo "$(BLUE)Updating dependencies...$(NC)"
	cargo update
	@echo "$(GREEN)✓ Dependencies updated$(NC)"

.PHONY: audit
audit: ## Audit dependencies for security vulnerabilities
	@echo "$(BLUE)Auditing dependencies...$(NC)"
	@if ! command -v cargo-audit >/dev/null 2>&1; then \
		echo "$(YELLOW)Installing cargo-audit...$(NC)"; \
		cargo install cargo-audit; \
	fi
	cargo audit
	@echo "$(GREEN)✓ Security audit complete$(NC)"

.PHONY: outdated
outdated: ## Check for outdated dependencies
	@echo "$(BLUE)Checking for outdated dependencies...$(NC)"
	@if ! command -v cargo-outdated >/dev/null 2>&1; then \
		echo "$(YELLOW)Installing cargo-outdated...$(NC)"; \
		cargo install cargo-outdated; \
	fi
	cargo outdated
	@echo "$(GREEN)✓ Outdated dependencies check complete$(NC)"

.PHONY: size
size: build-release ## Show binary size information
	@echo "$(BLUE)Binary size information:$(NC)"
	@ls -lh $(RELEASE_DIR)/$(BINARY_NAME)
	@echo ""
	@echo "$(BLUE)Detailed size breakdown:$(NC)"
	@if command -v bloaty >/dev/null 2>&1; then \
		bloaty $(RELEASE_DIR)/$(BINARY_NAME); \
	else \
		echo "$(YELLOW)Install 'bloaty' for detailed size analysis$(NC)"; \
		size $(RELEASE_DIR)/$(BINARY_NAME) 2>/dev/null || echo "$(YELLOW)'size' command not available$(NC)"; \
	fi

.PHONY: docker-build
docker-build: ## Build Docker image
	@echo "$(BLUE)Building Docker image...$(NC)"
	docker build -t $(IMAGE_NAME):$(TAG) .
	@if [ -n "$(REGISTRY)" ]; then \
		docker tag $(IMAGE_NAME):$(TAG) $(REGISTRY)/$(IMAGE_NAME):$(TAG); \
		echo "$(GREEN)✓ Docker image built and tagged: $(REGISTRY)/$(IMAGE_NAME):$(TAG)$(NC)"; \
	else \
		echo "$(GREEN)✓ Docker image built: $(IMAGE_NAME):$(TAG)$(NC)"; \
	fi

.PHONY: docker-build-dev
docker-build-dev: ## Build development Docker image
	@echo "$(BLUE)Building development Docker image...$(NC)"
	docker build -f Dockerfile.dev -t $(IMAGE_NAME):dev .
	@echo "$(GREEN)✓ Development Docker image built: $(IMAGE_NAME):dev$(NC)"

.PHONY: docker-push
docker-push: docker-build ## Build and push Docker image
	@if [ -z "$(REGISTRY)" ]; then \
		echo "$(RED)Error: REGISTRY environment variable must be set$(NC)"; \
		exit 1; \
	fi
	@echo "$(BLUE)Pushing Docker image...$(NC)"
	docker push $(REGISTRY)/$(IMAGE_NAME):$(TAG)
	@echo "$(GREEN)✓ Docker image pushed: $(REGISTRY)/$(IMAGE_NAME):$(TAG)$(NC)"

.PHONY: docker-run
docker-run: ## Run the application in Docker
	@echo "$(BLUE)Running $(PROJECT_NAME) in Docker...$(NC)"
	docker run --rm -p 3030:3030 \
		-e AWS_REGION \
		-e AWS_PROFILE \
		-e AWS_ACCESS_KEY_ID \
		-e AWS_SECRET_ACCESS_KEY \
		-e AWS_SESSION_TOKEN \
		$(IMAGE_NAME):$(TAG)

.PHONY: docker-compose-up
docker-compose-up: ## Start services with docker-compose
	@echo "$(BLUE)Starting services with docker-compose...$(NC)"
	docker-compose up -d
	@echo "$(GREEN)✓ Services started$(NC)"
	@echo "$(YELLOW)Available services:$(NC)"
	@echo "  - API: http://localhost:3030/cloudmap_sd"

.PHONY: docker-compose-up-dev
docker-compose-up-dev: ## Start development services with docker-compose
	@echo "$(BLUE)Starting development services...$(NC)"
	docker-compose --profile dev up -d
	@echo "$(GREEN)✓ Development services started$(NC)"

.PHONY: docker-compose-up-monitoring
docker-compose-up-monitoring: ## Start services with monitoring stack
	@echo "$(BLUE)Starting services with monitoring...$(NC)"
	docker-compose --profile monitoring up -d
	@echo "$(GREEN)✓ Services with monitoring started$(NC)"
	@echo "$(YELLOW)Available services:$(NC)"
	@echo "  - API: http://localhost:3030/cloudmap_sd"
	@echo "  - Prometheus: http://localhost:9090"
	@echo "  - Grafana: http://localhost:3000 (admin/admin)"

.PHONY: docker-compose-down
docker-compose-down: ## Stop and remove docker-compose services
	@echo "$(BLUE)Stopping docker-compose services...$(NC)"
	docker-compose down
	@echo "$(GREEN)✓ Services stopped$(NC)"

.PHONY: docker-compose-logs
docker-compose-logs: ## Show docker-compose logs
	@echo "$(BLUE)Showing docker-compose logs...$(NC)"
	docker-compose logs -f

.PHONY: docker-clean
docker-clean: ## Clean Docker images and containers
	@echo "$(BLUE)Cleaning Docker artifacts...$(NC)"
	docker-compose down --rmi all --volumes --remove-orphans 2>/dev/null || true
	docker rmi $(IMAGE_NAME):$(TAG) 2>/dev/null || true
	docker rmi $(IMAGE_NAME):dev 2>/dev/null || true
	docker system prune -f
	@echo "$(GREEN)✓ Docker cleanup complete$(NC)"

.PHONY: dev-setup
dev-setup: ## Set up development environment
	@echo "$(BLUE)Setting up development environment...$(NC)"
	@echo "Installing useful cargo tools..."
	cargo install cargo-watch cargo-tarpaulin cargo-audit cargo-outdated
	@echo "$(GREEN)✓ Development environment setup complete$(NC)"
	@echo ""
	@echo "$(YELLOW)Useful development commands:$(NC)"
	@echo "  cargo watch -x 'run'           # Auto-reload on file changes"
	@echo "  cargo watch -x 'test'          # Auto-test on file changes"
	@echo "  make test-coverage              # Generate test coverage"

.PHONY: dev-watch
dev-watch: ## Watch for changes and rebuild/run automatically
	@echo "$(BLUE)Watching for changes...$(NC)"
	@if ! command -v cargo-watch >/dev/null 2>&1; then \
		echo "$(YELLOW)Installing cargo-watch...$(NC)"; \
		cargo install cargo-watch; \
	fi
	cargo watch -x 'run'

.PHONY: dev-test-watch
dev-test-watch: ## Watch for changes and run tests automatically
	@echo "$(BLUE)Watching for changes and running tests...$(NC)"
	@if ! command -v cargo-watch >/dev/null 2>&1; then \
		echo "$(YELLOW)Installing cargo-watch...$(NC)"; \
		cargo install cargo-watch; \
	fi
	cargo watch -x 'test'

.PHONY: version
version: ## Show version information
	@echo "$(BLUE)Version Information:$(NC)"
	@echo "  Project:     $(PROJECT_NAME)"
	@echo "  Version:     $(VERSION)"
	@echo "  Git Commit:  $(GIT_COMMIT)"
	@echo "  Build Date:  $(BUILD_DATE)"
	@echo ""
	@echo "$(BLUE)Rust Information:$(NC)"
	@rustc --version
	@cargo --version

.PHONY: ci
ci: fmt-check clippy test ## Run CI checks (format, lint, test)
	@echo "$(GREEN)✓ All CI checks passed$(NC)"

.PHONY: release-prep
release-prep: clean ci build-release test doc ## Prepare for release (clean, check, build, test, doc)
	@echo "$(GREEN)✓ Release preparation complete$(NC)"
	@echo ""
	@echo "$(BLUE)Release artifacts:$(NC)"
	@ls -la $(RELEASE_DIR)/$(BINARY_NAME)
	@echo ""
	@echo "$(YELLOW)Next steps:$(NC)"
	@echo "  1. Tag the release: git tag v$(VERSION)"
	@echo "  2. Push the tag: git push origin v$(VERSION)"
	@echo "  3. Create GitHub release with binary: $(RELEASE_DIR)/$(BINARY_NAME)"

# Default target
.DEFAULT_GOAL := help
