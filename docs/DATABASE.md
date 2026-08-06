# Database Schema

SQLite with WAL mode. Defined in `pos-backend/src/db/schema.rs`.

## Tables

### invoices

| Column | Type | Constraints |
|--------|------|-------------|
| `id` | TEXT | PRIMARY KEY |
| `reference_pubkey` | TEXT | UNIQUE NOT NULL |
| `fiat_currency` | TEXT | NOT NULL |
| `fiat_amount` | REAL | NOT NULL |
| `usdc_amount` | REAL | NOT NULL |
| `status` | TEXT | NOT NULL DEFAULT 'pending' |
| `tx_signature` | TEXT | UNIQUE (partial, WHERE IS NOT NULL) |
| `customer_address` | TEXT | |
| `pix_id` | TEXT | |
| `pix_payload` | TEXT | |
| `tax_rate_pct` | REAL | DEFAULT 0.0 |
| `items_breakdown` | TEXT | |
| `telegram_chat_id` | INTEGER | |
| `telegram_msg_id` | INTEGER | |
| `telegram_expired_notified` | INTEGER | DEFAULT 0 |
| `created_at` | TIMESTAMP | DEFAULT CURRENT_TIMESTAMP |
| `updated_at` | TIMESTAMP | DEFAULT CURRENT_TIMESTAMP |

**Status values**: `pending`, `paid`, `cancelled`, `partially_paid`, `expired`

**Indexes**: `idx_invoices_tx_sig` on `tx_signature` WHERE `tx_signature IS NOT NULL` (partial unique)

### pending_webhook_updates

| Column | Type | Constraints |
|--------|------|-------------|
| `update_id` | INTEGER | PRIMARY KEY |
| `chat_id` | INTEGER | |
| `payload` | TEXT | NOT NULL |
| `status` | TEXT | NOT NULL DEFAULT 'pending' |
| `attempts` | INTEGER | DEFAULT 0 |
| `next_retry_at` | TIMESTAMP | |
| `locked_at` | TIMESTAMP | |
| `created_at` | TIMESTAMP | DEFAULT CURRENT_TIMESTAMP |

**Status values**: `pending`, `processing`, `retry_pending`, `cancelled`

**Indexes**: `idx_pending_fifo` composite index on `(chat_id, status, update_id)` for $O(1)$ FIFO batch query execution

### failed_updates (Dead-Letter Queue / DLQ)

| Column | Type | Constraints |
|--------|------|-------------|
| `update_id` | INTEGER | PRIMARY KEY |
| `chat_id` | INTEGER | |
| `payload` | TEXT | NOT NULL |
| `error_message` | TEXT | NOT NULL |
| `retry_count` | INTEGER | DEFAULT 0 |
| `failed_at` | TIMESTAMP | DEFAULT CURRENT_TIMESTAMP |

### processed_updates

| Column | Type | Constraints |
|--------|------|-------------|
| `update_id` | INTEGER | PRIMARY KEY |
| `processed_at` | TIMESTAMP | DEFAULT CURRENT_TIMESTAMP |
| `status` | TEXT | DEFAULT 'processed' |
| `retry_count` | INTEGER | DEFAULT 0 |

**Status values**: `processed`, `retry_pending`, `failed`

### telegram_fsm_sessions

| Column | Type | Constraints |
|--------|------|-------------|
| `chat_id` | INTEGER | NOT NULL, PART OF PRIMARY KEY |
| `user_id` | INTEGER | NOT NULL, PART OF PRIMARY KEY |
| `state` | TEXT | NOT NULL |
| `payload_json` | TEXT | |
| `updated_at` | TIMESTAMP | DEFAULT CURRENT_TIMESTAMP |

**Primary Key**: Composite `PRIMARY KEY (chat_id, user_id)`

### system_settings

| Column | Type | Constraints |
|--------|------|-------------|
| `key` | TEXT | PRIMARY KEY |
| `value` | TEXT | NOT NULL |

**Key Entries**:
- `telegram_update_offset`: Last processed Telegram update_id (persisted with `IN_MEMORY_OFFSET` atomic coordination).
- `lang_{chat_id}`: Language code preference per chat_id (e.g. `lang_123456` = `"uk"`), backed by $O(1)$ LRU cache.
- `quick_receipt_amount` & `quick_receipt_currency`: Quick receipt POS defaults.

### squads_proposals

| Column | Type | Constraints |
|--------|------|-------------|
| `proposal_index` | INTEGER | PRIMARY KEY |
| `invoice_id` | TEXT | NOT NULL, FK → invoices(id) |
| `recipient_pubkey` | TEXT | NOT NULL |
| `amount_usdc` | REAL | NOT NULL |
| `status` | TEXT | NOT NULL DEFAULT 'created' |
| `tx_base64` | TEXT | |
| `created_at` | TIMESTAMP | DEFAULT CURRENT_TIMESTAMP |

### nonce_accounts

| Column | Type | Constraints |
|--------|------|-------------|
| `pubkey` | TEXT | PRIMARY KEY |
| `status` | TEXT | NOT NULL DEFAULT 'free' |
| `locked_at` | TIMESTAMP | |

**Status values**: `free`, `locked`, `stale_needs_refresh`

**Seed data**: 3 nonce accounts auto-inserted if table is empty.

### sop_checkpoints

| Column | Type | Constraints |
|--------|------|-------------|
| `id` | TEXT | PRIMARY KEY |
| `sop_id` | TEXT | NOT NULL |
| `step_id` | TEXT | NOT NULL |
| `state_data` | TEXT | |
| `status` | TEXT | NOT NULL DEFAULT 'pending' |
| `created_at` | TIMESTAMP | DEFAULT CURRENT_TIMESTAMP |
| `updated_at` | TIMESTAMP | DEFAULT CURRENT_TIMESTAMP |

## Pragmas

```sql
PRAGMA journal_mode=WAL;
PRAGMA busy_timeout=5000;
PRAGMA synchronous=NORMAL;
PRAGMA cache_size=-64000;  -- 64MB RAM cache
```

## Connection Pooling & Webhook Zero Data Loss

- **`deadpool-sqlite` Pool Acquisition Timeout**: `pool.get()` is wrapped in `tokio::time::timeout(Duration::from_millis(4500), ...)`. If connection pool acquisition exceeds 4.5 seconds under high load, the Webhook handler fast-fails with `HTTP 500 Internal Server Error`, ensuring Telegram's gateway retries update delivery instead of dropping data.

## SQLite RAII Transaction Safety (`TransactionRollbackGuard`)

- **`BEGIN IMMEDIATE` Write Lock**: Batch claiming in `fetch_pending_batch()` executes an immediate write transaction lock to eliminate `SQLITE_BUSY` contention across worker threads.
- **RAII Rollback Guard**: A custom `TransactionRollbackGuard<'a>(&'a Connection, bool)` struct implements `Drop`. If any `panic!` or error occurs before explicit `COMMIT` (`guard.1 = true`), `ROLLBACK` executes automatically upon drop, preventing uncommitted transaction leaks.

## Strict Per-Chat FIFO Queue & Lease Expiration

- **Atomic Batch Claim Query**: Uses `UPDATE ... RETURNING` with 30-second lease expiration (`locked_at < datetime('now', '-30 seconds')`).
- **Head-of-Line Unblocking**: Enforces strict FIFO update execution per `chat_id` for active/ready updates, while unblocking incoming commands when prior updates are waiting in exponential backoff (`retry_pending` with `next_retry_at > datetime('now')`).

## Offset & State Persistence

- **Offset Storage**: Last processed Telegram update offset is persisted in SQLite `system_settings` table under `telegram_update_offset`.
- **User Language Preference**: Chat language preferences are stored under key `lang_{chat_id}` in `system_settings`, backed by an in-memory thread-safe $O(1)$ LRU cache (`lang_cache.rs`).

## DLQ & DB Retry Backoff Policy

- **Atomic Failure Recording**: `db::updates::record_failure_and_check_max_retries` performs up to **3 retry attempts** with exponential backoff (`5s * 2^(attempt-1)`) before moving failed updates to `failed_updates` (DLQ).

## Atomic State Transitions

All invoice status updates use atomic guards to prevent race conditions:

```sql
UPDATE invoices SET status = ? WHERE id = ? AND status = 'pending'
```

## Nonce Allocation

Uses `UPDATE ... RETURNING` for atomic allocation:

```sql
UPDATE nonce_accounts SET status = 'locked', locked_at = CURRENT_TIMESTAMP
WHERE pubkey = (SELECT pubkey FROM nonce_accounts WHERE status = 'free' LIMIT 1)
RETURNING pubkey;
```

Falls back to `BEGIN IMMEDIATE` transaction for SQLite < 3.35.0.

