#!/usr/bin/env bash
set -e

echo "=========================================================="
echo "🚀 Initializing ZeroClaw Solana POS Payment Terminal Agent"
echo "=========================================================="

# Check if .env exists, if not copy from .env.example
if [ ! -f ".env" ]; then
    echo "📋 Copying .env.example -> .env"
    cp .env.example .env
    echo "⚠️  Please update .env with your actual TELEGRAM_BOT_TOKEN, RPC_URL and Wallet addresses."
else
    echo "✅ Found existing .env file."
fi

# Check if config.toml exists, if not copy from config.example.toml
if [ ! -f "config.toml" ]; then
    echo "📋 Copying config.example.toml -> config.toml"
    cp config.example.toml config.toml
else
    echo "✅ Found existing config.toml file."
fi

# Check if rustup is available for WASM target setup
if command -v rustup >/dev/null 2>&1; then
    echo "🦀 Checking Rust WASM target wasm32-wasip2..."
    rustup target add wasm32-wasip2 2>/dev/null || echo "ℹ️ wasm32-wasip2 target ready."
else
    echo "ℹ️ rustup not found in PATH; skipping WASM target check."
fi

# Create data directory for ZeroClaw runtime state & SQLite DB with 777 permissions (prevents Docker permission denied errors)
echo "📁 Setting up data directory permissions for Docker & SQLite WAL..."
mkdir -p data sops skills scripts plugins/solana-pos-core
chmod -R 777 data

# Install Git Pre-Commit Hook for guaranteed pre-commit checks
if [ -d ".git" ]; then
    echo "⚙️ Installing Git Pre-Commit Hook (.git/hooks/pre-commit)..."
    mkdir -p .git/hooks
    cp scripts/pre_commit.sh .git/hooks/pre-commit
    chmod +x .git/hooks/pre-commit
    echo "✅ Git Pre-Commit Hook installed successfully!"
fi

echo ""
echo "=========================================================="
echo "✨ Environment initialized successfully!"
echo "To build the Tier 3 Rust WASM plugin, run:"
echo "   ./scripts/build_wasm.sh"
echo ""
echo "To start the local SQLite POS API backend, run:"
echo "   python3 scripts/pos_backend.py 8080"
echo ""
echo "To run the automated Prompt-Injection security test suite:"
echo "   python3 scripts/test_prompt_inj.py"
echo "=========================================================="
