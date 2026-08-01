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

# Create data directory for ZeroClaw runtime state
mkdir -p data sops skills scripts

echo ""
echo "=========================================================="
echo "✨ Environment initialized successfully!"
echo "To start the agent via Docker, run:"
echo "   docker-compose up -d"
echo ""
echo "To run the automated Prompt-Injection security test suite:"
echo "   python3 scripts/test_prompt_inj.py"
echo "=========================================================="
