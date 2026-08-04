#!/usr/bin/env bash
set -e

# Source Cargo env if available
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

echo "================================================================="
echo "🏆 ZeroClaw Solana POS Agent - Complete Automated Verification"
echo "================================================================="

echo "1. Initializing Environment..."
./scripts/setup.sh

echo ""
echo "2. Building & Validating Rust WASM Plugin (solana-pos-core)..."
./scripts/build_wasm.sh

if command -v wasm-tools >/dev/null 2>&1; then
    echo "📋 Validating WASI Component Model Spec via wasm-tools..."
    wasm-tools validate plugins/solana-pos-core/target/wasm32-wasip2/release/solana_pos_core.wasm --features component-model
    echo "✅ WASI Component validation PASSED!"
else
    echo "ℹ️  wasm-tools not found in PATH; skipping optional component validation."
fi

echo ""
echo "3. Running pos-backend Tests..."
cargo test --manifest-path pos-backend/Cargo.toml

echo ""
echo "4. Running pos-core-logic Tests..."
cargo test --manifest-path plugins/solana-pos-core/pos-core-logic/Cargo.toml

echo ""
echo "5. Running solana-pos-core WASM Plugin Tests..."
cd plugins/solana-pos-core && cargo test --lib --release && cd - > /dev/null

echo ""
echo "6. Running Pre-Commit Safety Check..."
./scripts/pre_commit.sh

echo ""
echo "================================================================="
echo "✨ ALL VERIFICATION STEPS PASSED PERFECTLY!"
echo "================================================================="
