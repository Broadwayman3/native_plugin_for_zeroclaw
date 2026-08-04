# AGENTS.md — ZeroClaw Solana POS Agent

## Mandatory Pre-Commit Checks

**Commits are FORBIDDEN unless ALL checks pass with ZERO errors and ZERO warnings.**

On every commit, checks run on the **files touched by the diff** (see `git diff --name-only`), NOT the whole tree.

### Lint & Format (touched files only)

```bash
# Must pass with EXIT=0 and NO output
cargo fmt --check --manifest-path pos-backend/Cargo.toml
cargo fmt --check --manifest-path plugins/solana-pos-core/Cargo.toml
```

### Clippy Linter (touched files only)

```bash
# Must pass with EXIT=0 and ZERO warnings
cargo clippy --manifest-path pos-backend/Cargo.toml -- -D warnings
cargo clippy --manifest-path plugins/solana-pos-core/Cargo.toml -- -D warnings
```

### Test Suite

```bash
# Must pass with 100% rate (all tests green, no failures)
cargo test --manifest-path pos-backend/Cargo.toml
cargo test --manifest-path plugins/solana-pos-core/pos-core-logic/Cargo.toml
cd plugins/solana-pos-core && cargo test --lib --release
```

## File Size Rules

- **Maximum 400 lines per file.** No god classes. No monoliths.
- Files exceeding 400 lines MUST be split into smaller modules.
- Each module MUST have a single responsibility.

## Code Standards

- Rust idioms: prefer `Result` over `unwrap()`, use `thiserror` for error types
- All SQL MUST use parameterized queries (no string interpolation in SQL)
- All DB connections MUST use `try/finally` pattern (rusqlite `Connection` dropped properly)
- All Telegram user input MUST go through `sanitize_external_input()`
- All Telegram output text MUST be escaped via `escape_telegram_markdown_v2()`
- Manager-only actions MUST check `MANAGER_TELEGRAM_ID`
- Financial calculations MUST use `u128` atomic units (no float for on-chain amounts)

## Project Structure

```
pos-backend/src/
├── main.rs              # Axum HTTP server entrypoint
├── config.rs            # AppConfig struct, env-var loader
├── error.rs             # AppError enum (thiserror)
├── api/                 # REST endpoints
│   ├── mod.rs           # Router builder, CORS, AppState
│   ├── actions.rs       # Solana Actions/Blinks
│   ├── invoices.rs      # Invoice CRUD
│   ├── nonce.rs         # Durable nonce pool
│   ├── pos_flow.rs      # POS order creation
│   ├── sales.rs         # Sales summary
│   └── x402.rs          # x402 machine commerce
├── db/                  # SQLite data access
│   ├── schema.rs        # DDL, migrations, nonce seeding
│   ├── invoices.rs      # Invoice DAO
│   ├── nonce.rs         # Nonce account pool
│   ├── squads.rs        # Squads v4 proposals
│   └── ...
└── domain/              # Business logic
    ├── sanitizer.rs     # SSRF guard, input sanitization
    ├── verification.rs  # Triple Payment Verification
    ├── i18n.rs          # 13-language i18n dispatcher
    ├── price_feed.rs    # Multi-tier fiat rate fallback
    └── ...

plugins/solana-pos-core/
├── src/lib.rs           # WASM entrypoint (wit-bindgen)
└── pos-core-logic/src/  # Shared business logic
    ├── solana_pay.rs    # Solana Pay URL builder
    ├── squads.rs        # Squads v4 proposal builder
    └── token2022.rs     # Token-2022 fee calculator
```

## Test Conventions

- Tests are numbered sequentially: `test_001` through `test_310`
- Each test module is self-contained
- Property-based testing via `proptest` for financial math
- Run full suite: `./scripts/verify_all.sh`

## Documentation

- `docs/ARCHITECTURE.md` — crate structure, WIT ABI, data flow
- `docs/API.md` — REST API reference (12 endpoints)
- `docs/SECURITY.md` — threat model, defense matrix
- `docs/DATABASE.md` — schema, migrations, pragmas
- `docs/DEPLOYMENT.md` — Docker, local dev, env vars
- `docs/TESTING.md` — test strategy, module breakdown
