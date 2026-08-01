#!/usr/bin/env bash
set -e

echo "=========================================================="
echo "⚙️  Building ZeroClaw Tier 3 Rust WASM Plugin (solana-pos-core)"
echo "=========================================================="

PLUGIN_DIR="plugins/solana-pos-core"

if [ ! -d "$PLUGIN_DIR" ]; then
    echo "❌ Plugin directory $PLUGIN_DIR not found!"
    exit 1
fi

echo "📋 Validating Cargo manifest and WIT contract interface..."
if [ -f "$PLUGIN_DIR/Cargo.toml" ] && [ -f "wit/v0/pos_core.wit" ]; then
    echo "✅ WIT interface: wit/v0/pos_core.wit"
    echo "✅ Cargo manifest: $PLUGIN_DIR/Cargo.toml"
else
    echo "❌ Missing WIT contract or Cargo.toml!"
    exit 1
fi

if command -v cargo >/dev/null 2>&1; then
    # Ensuring target wasm32-wasip2 is installed
    if command -v rustup >/dev/null 2>&1; then
        rustup target add wasm32-wasip2 2>/dev/null || true
    fi

    echo ""
    echo "🦀 Executing Rust unit and property-based tests..."
    cd "$PLUGIN_DIR"
    cargo test --lib --release

    echo ""
    echo "🔨 Compiling WASM module to target wasm32-wasip2..."
    cargo build --target wasm32-wasip2 --release
    cd - > /dev/null

    WASM_FILE="$PLUGIN_DIR/target/wasm32-wasip2/release/solana_pos_core.wasm"
    if [ -f "$WASM_FILE" ]; then
        SIZE=$(du -h "$WASM_FILE" | cut -f1)
        echo ""
        echo "=========================================================="
        echo "🎉 WASM Plugin successfully built!"
        echo "📦 Binary: $WASM_FILE ($SIZE)"
        echo "🎯 Target: wasm32-wasip2 (WASI Component Model)"
        echo "=========================================================="
    else
        echo "❌ WASM build failed: binary not found!"
        exit 1
    fi
else
    echo ""
    echo "⚠️  Cargo is not installed in current environment."
    echo "ℹ️  CI/CD pipeline (.github/workflows/ci.yml) will execute full compilation via dtolnay/rust-toolchain with wasm32-wasip2 target."
    echo "=========================================================="
    echo "✅ Cargo manifest & WIT interface verified!"
    echo "=========================================================="
fi
