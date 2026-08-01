#!/usr/bin/env bash
set -e

echo "🔍 Running ZeroClaw Pre-Commit Automated Safety Check..."

if [ -f "$HOME/.cargo/bin/cargo" ]; then
    export PATH="$HOME/.cargo/bin:$PATH"
fi

# 1. AST Safety Linter (SQL Injection & Math Guard)
python3 scripts/lint_safety_ast.py

# 2. Rust Formatting, WASM Check & Clippy Linter
cd plugins/solana-pos-core
if command -v cargo >/dev/null 2>&1; then
    cargo fmt --check
    cargo clippy --target wasm32-wasip2 -- -D warnings
    cargo check --target wasm32-wasip2
    cargo build --target wasm32-wasip2 --release
fi
cd - > /dev/null

# 3. Python Type Safety & Schema Validation
python3 scripts/validators.py
python3 scripts/sanitizer.py

# 4. Security Audit & Prompt Injection Test
python3 scripts/test_prompt_inj.py

# 5. Comprehensive Boundary Cases (250 Tests)
python3 scripts/test_boundary_cases.py

echo "✅ All pre-commit checks passed successfully! Commit allowed."
