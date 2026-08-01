#!/usr/bin/env bash
set -e

echo "🔍 Running ZeroClaw Pre-Commit Automated Safety Check..."

if [ -f "$HOME/.cargo/bin/cargo" ]; then
    export PATH="$HOME/.cargo/bin:$PATH"
fi

# 1. Formatting & Linter Checks
cd plugins/solana-pos-core
if command -v cargo >/dev/null 2>&1; then
    cargo fmt --check
    cargo check --target wasm32-wasip2
    cargo build --target wasm32-wasip2 --release
fi
cd - > /dev/null

# 2. Python Type Safety & Schema Validation
python3 scripts/validators.py
python3 scripts/sanitizer.py

# 3. Security Audit & Prompt Injection Test
python3 scripts/test_prompt_inj.py

# 4. Comprehensive Boundary Cases (120 Tests)
python3 scripts/test_boundary_cases.py

echo "✅ All pre-commit checks passed successfully! Commit allowed."
