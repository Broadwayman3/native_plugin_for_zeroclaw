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

# 2. Custom AST Safety Linter (SQL Injection & Math Guard)
echo "📋 Running Python AST Static Code Safety Linter..."
python3 scripts/lint_safety_ast.py

# 3. Bandit Security Static Analysis
if command -v bandit >/dev/null 2>&1; then
    echo "🛡️ Running Bandit Security Analysis..."
    bandit -r scripts/ -x scripts/test_* --severity-level medium
fi

# 4. Python Type Safety & Style Checks (mypy, flake8/ruff)
if command -v mypy >/dev/null 2>&1; then
    echo "🔬 Running mypy type checking..."
    mypy scripts/validators.py scripts/sanitizer.py scripts/pos_core/*.py --ignore-missing-imports
fi

if command -v flake8 >/dev/null 2>&1; then
    echo "🎨 Running flake8 linter..."
    flake8 scripts/validators.py scripts/sanitizer.py scripts/pos_backend.py scripts/pos_core/*.py --max-line-length=120 --ignore=E501,W503
elif command -v ruff >/dev/null 2>&1; then
    echo "🎨 Running ruff linter..."
    ruff check scripts/
fi

# 5. Rust Formatting, WASM Check & Clippy Linter
cd plugins/solana-pos-core
if command -v cargo >/dev/null 2>&1; then
    cargo fmt --check
    cargo clippy --target wasm32-wasip2 -- -D warnings
    cargo check --target wasm32-wasip2
    cargo build --target wasm32-wasip2 --release
fi
cd - > /dev/null

# 6. Python Schema Validation & Sanitizer Self-Tests
python3 scripts/validators.py
python3 scripts/sanitizer.py

# 7. Security Audit & Prompt Injection Suite
python3 scripts/test_prompt_inj.py

# 8. Comprehensive Boundary Cases (305 Tests)
python3 scripts/test_boundary_cases.py

echo "================================================================="
echo "✅ All pre-commit security & linting checks passed! Commit allowed."
echo "================================================================="
