# Justfile for yft-sandbox-mcp

# Default recipe runs local CI
default: ci

# Run all local CI checks (formatting, linting, tests)
ci: format lint test
    @echo "All local CI checks passed successfully!"

# Check code formatting without making changes
format:
    cargo fmt --all -- --check

# Run clippy linter with warnings treated as errors
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Run all unit and integration tests
test:
    cargo test --all-targets --all-features

# Automatically fix code formatting and safe clippy suggestions
fix:
    cargo fmt --all
    cargo clippy --fix --allow-dirty --allow-staged --all-targets --all-features

# Build debug binary
build:
    cargo build

# Build release binary
build-release:
    cargo build --release

# Install and configure git pre-push hook for local CI enforcement
install-hooks:
    chmod +x .githooks/pre-push
    git config core.hooksPath .githooks
    @echo "Git hooks configured successfully to use .githooks/"
