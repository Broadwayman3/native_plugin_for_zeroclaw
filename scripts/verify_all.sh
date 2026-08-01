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
echo "2. Validating Fail-Closed JSON Schema & Context Truncation Engine..."
python3 scripts/validators.py

echo ""
echo "3. Building & Validating Rust WASM Plugin (solana-pos-core)..."
./scripts/build_wasm.sh

if command -v wasm-tools >/dev/null 2>&1; then
    echo "📋 Validating WASI Component Model Spec via wasm-tools..."
    wasm-tools validate plugins/solana-pos-core/target/wasm32-wasip2/release/solana_pos_core.wasm --features component-model
    echo "✅ WASI Component validation PASSED!"
else
    echo "ℹ️  wasm-tools not found in PATH; skipping optional component validation."
fi

echo ""
echo "4. Testing Local SQLite Database, Nonce Pool & x402 Engine..."
python3 scripts/pos_backend.py --test

echo ""
echo "5. Running Prompt Injection & Security Audit Suite..."
python3 scripts/test_prompt_inj.py

echo ""
echo "6. Executing Pre-Commit Automated Safety Check..."
./scripts/pre_commit.sh

echo ""
echo "7. Running Full 130 Comprehensive Boundary & Edge Case Tests..."
python3 scripts/test_boundary_cases.py

echo ""
echo "================================================================="
echo "✨ ALL VERIFICATION STEPS PASSED PERFECTLY (100% READY FOR 1ST PLACE)!"
echo "================================================================="
