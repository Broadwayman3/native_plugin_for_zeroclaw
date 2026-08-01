# Project Rules & Development Guidelines: ZeroClaw Solana POS Agent

These rules enforce strict architectural, logical, mathematical, and security standards across the codebase to prevent regressions, tech debt, and vulnerabilities.

---

## 🏗️ 1. Architecture & Domain Modularization
- **Modular Core Structure**: All business and domain logic must be maintained in `scripts/pos_core/` split by responsibility:
  - `db.py`: Database connection, WAL mode, schema initialization & cleanup.
  - `nonce_pool.py`: Durable Nonce allocation, TTL auto-release (15 min) & revert recovery.
  - `solana_pay.py`: Solana transaction verification, balance deltas, atomic unit conversions & Squads v4 refund instructions.
  - `pix_brl.py`: EMV QRCPS PIX string generation & CRC16 CCITT-FALSE calculation.
  - `price_feed.py`: Multi-tier circuit breaker price feed fallback.
  - `router.py`: Query-aware stdlib micro-router for `http.server`.
- **Zero-Dependency Mandate**: Do NOT introduce heavy external Python frameworks (`FastAPI`, `Flask`, `Django`, `SQLAlchemy`). Keep zero external Python runtime dependencies using standard library modules.
- **Entrypoint Scoping**: `scripts/pos_backend.py` is strictly an entrypoint script for starting the HTTP REST API server and handling `--test` dry runs.

---

## 💾 2. Database & Concurrency Integrity
- **WAL Mode & Busy Timeouts**: SQLite must operate with `PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;`.
- **Database Path Parameterization**: All database routines MUST accept an optional `db_path: str = DB_PATH` parameter or an active `conn: sqlite3.Connection` object to ensure test isolation (`data/test_boundary.db`).
- **Atomic State Transitions**: Updates to invoice states must use atomic query guards (`UPDATE invoices SET status = ? WHERE id = ? AND status = 'pending'`) to eliminate double fulfillment race conditions.
- **SQL Injection Prevention**: Never concatenate raw strings into SQL queries. Always use parameterized queries (`cursor.execute("...", (param,))`).

---

## 🔢 3. Deterministic Financial Math & Precision Guards
- **Zero Float Drift**: Financial token amounts and fees must NEVER rely on raw float arithmetic.
- **Atomic Units Conversion**: Convert currency amounts to 64-bit integer atomic units (USDC = 6 decimals, SOL = 9 decimals) using `token_to_atomic_units()`. Critical fee calculations must occur in Rust `u128` WASM (`plugins/solana-pos-core`).
- **Boundary Guards**: Always protect conversion routines against `NaN`, `Infinity`, negative amounts, and `u64` integer overflow caps.

---

## 🧪 4. Single Source of Truth & Test Rigor
- **Modular Test Architecture**: Tests are organized in high-cohesion domain modules inside `scripts/tests/` (`test_payment_verification.py`, `test_database_concurrency.py`, `test_nonce_pools.py`, `test_token2022_math.py`, `test_fiat_pix.py`, `test_squads_multisig.py`).
- **No Test Logic Duplication**: Test scripts MUST NOT re-define production calculation functions. They must import directly from `pos_core`.
- **Pytest & CLI Dual Compatibility**: All test modules must support standard `pytest` discovery (`pytest scripts/tests/`) while preserving direct entrypoint execution (`python3 scripts/test_boundary_cases.py`).
- **1-Command Full Verification**: All changes must pass `./scripts/verify_all.sh` before merging.

---

## 🛡️ 5. Security, Sanitization & Non-Custodial Architecture
- **Prompt Injection Defense**: External inputs (memos, customer names, merchant tags) MUST be sanitized via `sanitizer.sanitize_external_input()`.
- **API Key & Secret Redaction**: Secrets, RPC keys, and Telegram tokens must be masked with `sanitizer.redact_api_key()` before logging or raising exceptions.
- **SSRF Protection**: RPC URLs must be validated with `sanitizer.validate_safe_rpc_url()` to block private IP and cloud metadata requests.
- **Keyless Sandbox & Squads Proposer**: The agent operates strictly in Tier 1 (Solana Pay URLs) and Tier 3 (WASM sandbox). Refunds must build Squads v4 proposals where the agent acts solely as a `Proposer`, leaving threshold signing authority to store owners.
