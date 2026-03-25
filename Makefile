# InQL — Incan library package
# =============================
#
# Requires `incan` on your PATH (build/install from the Incan compiler repository).
# CI builds Incan from source first; locally, use your own toolchain.
#
# Override the binary if needed: `make build INCAN=/path/to/incan`

INCAN ?= incan
export INCAN_NO_BANNER ?= 1

.DEFAULT_GOAL := help

.PHONY: help
help: ## Show Make targets
	@echo "\033[1mInQL\033[0m — typed relational layer (Incan library)."
	@echo "Requires \033[36m$(INCAN)\033[0m on PATH, or set \033[36mINCAN=\033[0m/path/to/incan."
	@echo ""
	@grep -E '^[a-zA-Z0-9_.-]+:.*?##' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'

# =============================================================================
# Build & test (primary — Incan-first)
# =============================================================================

.PHONY: build
build: ## Build the library (`incan build --lib`)
	@echo "\033[1mBuilding InQL library...\033[0m"
	@$(INCAN) build --lib

.PHONY: test
test: ## Run package tests (`incan test`)
	@echo "\033[1mRunning InQL tests...\033[0m"
	@$(INCAN) test

.PHONY: build-locked
build-locked: ## Build with `--locked` (stricter; requires current `incan.lock`)
	@echo "\033[1mBuilding InQL library (locked)...\033[0m"
	@$(INCAN) build --lib --locked

.PHONY: test-locked
test-locked: ## Run tests with `--locked`
	@echo "\033[1mRunning InQL tests (locked)...\033[0m"
	@$(INCAN) test --locked

# =============================================================================
# Formatting (Incan source)
# =============================================================================

.PHONY: fmt
fmt: ## Format `.incn` sources (`incan fmt`)
	@echo "\033[1mFormatting Incan sources...\033[0m"
	@$(INCAN) fmt .

.PHONY: fmt-check
fmt-check: ## Check formatting without writing (`incan fmt --check`)
	@echo "\033[1mChecking Incan source formatting...\033[0m"
	@$(INCAN) fmt --check .

# =============================================================================
# Aggregates (local gates)
# =============================================================================

.PHONY: check
check: fmt-check build test ## Format check, build, and test
	@echo "\033[32m✓ check passed\033[0m"

.PHONY: pre-commit
pre-commit: fmt-check build test ## Fast gate before commit (same as `check`)
	@echo "\033[32m✓ pre-commit gate passed\033[0m"

.PHONY: ci
ci: fmt-check build test ## Same steps as GitHub Actions `inql` job
	@echo "\033[32m✓ ci gate passed\033[0m"

.PHONY: verify
verify: ci ## Alias for `ci`

# =============================================================================
# Miscellaneous
# =============================================================================

.PHONY: version
version: ## Print `incan` version (requires toolchain on PATH)
	@$(INCAN) --version

.PHONY: clean
clean: ## Remove Incan `target/` outputs under this package
	@echo "\033[1mCleaning...\033[0m"
	@rm -rf target
	@echo "\033[32m✓ Clean\033[0m"
