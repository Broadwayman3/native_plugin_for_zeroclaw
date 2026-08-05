#!/usr/bin/env bash
set -e

echo "🔍 Running ZeroClaw Pre-Commit Automated Safety Check..."

if [ -f "$HOME/.cargo/bin/cargo" ]; then
    export PATH="$HOME/.cargo/bin:$PATH"
fi

# 1. ShellScript Security Check
if command -v shellcheck >/dev/null 2>&1; then
    echo "📋 Running shellcheck on bash scripts..."
    shellcheck scripts/*.sh
fi

# 2. Rust Formatting Check
echo "📋 Checking Rust code formatting..."
cargo fmt --check --manifest-path pos-backend/Cargo.toml
cargo fmt --check --manifest-path plugins/solana-pos-core/Cargo.toml

# 3. Rust Clippy Linter (Strict)
echo "🔬 Running cargo clippy on pos-backend..."
cargo clippy --manifest-path pos-backend/Cargo.toml -- -D warnings

echo "🔬 Running cargo clippy on solana-pos-core (WASM target)..."
cd plugins/solana-pos-core
cargo clippy --target wasm32-wasip2 -- -D warnings
cargo check --target wasm32-wasip2
cd - > /dev/null

# 4. Run All Tests
echo "🧪 Running pos-backend tests..."
cargo test --manifest-path pos-backend/Cargo.toml -- --test-threads=1

echo "🧪 Running pos-core-logic tests..."
cargo test --manifest-path plugins/solana-pos-core/pos-core-logic/Cargo.toml

echo "🧪 Running solana-pos-core WASM plugin tests..."
cd plugins/solana-pos-core && cargo test --lib --release && cd - > /dev/null

echo "================================================================="
echo "✅ All pre-commit security & linting checks passed! Commit allowed."
echo "================================================================="
