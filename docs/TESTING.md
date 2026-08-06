# Testing

## Overview

The project has **520 total tests** (481 in `pos-backend`, 31 in `pos-core-logic`, 8 in `solana-pos-core`) including property-based tests across three crates.

| Crate | Tests | Focus |
|-------|-------|-------|
| `pos-backend` | 475 + 6 proptest | REST API, DB, Telegram listener, FSM, i18n, security, edge cases |
| `pos-core-logic` | 27 + 4 proptest | Shared business logic (Solana Pay, Squads, Token-2022) |
| `solana-pos-core` | 5 + 3 proptest | WASM plugin unit tests |

## Test Numbering

Tests are numbered sequentially: `test_001` through `test_394`.

## Running Tests

### All crates

```bash
./scripts/verify_all.sh
```

### Individual crates

```bash
# pos-backend (481 tests)
cargo test --manifest-path pos-backend/Cargo.toml -- --test-threads=1

# pos-core-logic (31 tests)
cargo test --manifest-path plugins/solana-pos-core/pos-core-logic/Cargo.toml

# solana-pos-core WASM plugin (8 tests)
cd plugins/solana-pos-core && cargo test --lib --release
```

## Test Modules

| Module | Tests | Area |
|--------|-------|------|
| `test_squads_multisig.rs` | 50 | Squads v4 proposal building, Borsh serialization |
| `test_telegram_handlers.rs` | 38 | Telegram callback/text handler flows |
| `test_edge_storage.rs` | 31 | SQLite WAL, concurrent writes, migrations |
| `test_token2022.rs` | 30 | Token-2022 fee calculation, u128 precision |
| `test_telegram_listener.rs` | 24 | Telegram listener, FIFO queue, FSM session persistence |
| `test_solana_pay.rs` | 20 | Solana Pay URL generation, reference keys |
| `test_sanitizer.rs` | 20 | Input sanitization, SSRF, prompt injection |
| `test_nonce_pools.rs` | 20 | Nonce pool allocation, release, collision guards |
| `test_database.rs` | 20 | Invoice CRUD, status transitions, cleanup |
| `test_zeroclaw_integration.rs` | 18 | End-to-end ZeroClaw agent integration |
| `test_telegram_final.rs` | 14 | SQLite update deduplication, atomic cancellation, offset persistence |
| `test_telegram_edge_cases.rs` | 12 | Idempotent invoice cancellation, paid status conflicts |
| `test_verification.rs` | 10 | Triple Payment Verification engine |
| `test_price_feed.rs` | 10 | Fiat rate fallback, circuit breaker |
| `test_pix_brl.rs` | 10 | PIX QR CRC16 checksum validation |
| `test_i18n.rs` | 10 | i18n string translation, locale switching |
| `test_api.rs` | 10 | REST API endpoint contract tests |
| `test_listener_vulnerabilities.rs` | 8 | Telegram security, edited message filtering, DoS guards |
| `test_security.rs` | 6 | API key redaction, secret masking |
| `test_prompt_injection.rs` | 6 | Prompt injection defense tests |
| `test_qa_red_team.rs` | 5 | Red team security audit |

## Property-Based Testing (proptest)

`pos-core-logic` and `pos-backend` use proptest for fuzzing:
- `solana_pay.rs` — Solana Pay URL generation
- `squads.rs` — Squads v4 instruction building
- `token2022.rs` — Token-2022 fee calculation
- `test_proptest_order_parser.rs` — POS text order parser boundary fuzzing

## CI/CD

Tests run automatically on push/PR via `.github/workflows/ci.yml`:
1. shellcheck on bash scripts
2. `cargo fmt --check`
3. `cargo clippy -- -D warnings`
4. `cargo test` for all three crates
5. WASM build + WASI validation
6. Full `verify_all.sh` pipeline
