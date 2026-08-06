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
| `status` | TEXT | DEFAULT 'pending' |
| `attempts` | INTEGER | DEFAULT 0 |
| `locked_at` | TIMESTAMP | |
| `created_at` | TIMESTAMP | DEFAULT CURRENT_TIMESTAMP |
| `next_retry_at` | TIMESTAMP | DEFAULT CURRENT_TIMESTAMP |
| `retry_count` | INTEGER | DEFAULT 0 |

**Status values**: `pending`, `processing`, `retry_pending`, `done`, `cancelled`

**Indexes**: `idx_pending_fifo` composite index on `(chat_id, status, update_id)` for $O(1)$ FIFO batch query execution

### failed_updates (Dead-Letter Queue / DLQ)

| Column | Type | Constraints |
|--------|------|-------------|
| `update_id` | INTEGER | PRIMARY KEY |
| `chat_id` | INTEGER | |
| `payload` | TEXT | NOT NULL |
| `error_message` | TEXT | |
| `failed_at` | TIMESTAMP | DEFAULT CURRENT_TIMESTAMP |
| `retry_count` | INTEGER | DEFAULT 0 |

**Lifecycle**: Updates failing due to handler errors, execution timeouts (60s), or queue backpressure saturation after 3 retries (`max_retries = 3`) are recorded and isolated in DLQ (`failed_updates`) without halting Low Watermark offset progression.

### processed_updates

| Column | Type | Constraints |
|--------|------|-------------|
| `update_id` | INTEGER | PRIMARY KEY |
| `processed_at` | TIMESTAMP | DEFAULT CURRENT_TIMESTAMP |
| `status` | TEXT | DEFAULT 'processed' |
| `retry_count` | INTEGER | DEFAULT 0 |

**Status values**: `processed`, `retry_pending`, `failed`

**3-Phase Idempotency Registration & LRU Cache**: `IDEMPOTENCY_CACHE` in `webhook_db.rs` wraps an in-memory thread-safe `LruCache<i64, ()>` (capacity 10,000). Incoming update IDs perform a read-only pre-dispatch check (`is_update_processed`). Insertion into SQLite `processed_updates` via `check_and_register` and marking in `IDEMPOTENCY_CACHE` (`mark_cached_processed`) occurs strictly **post-dispatch upon verified handler completion**, eliminating duplicate payment processing while preserving retry capabilities on transient failures.

### telegram_fsm_sessions

| Column | Type | Constraints |
|--------|------|-------------|
| `chat_id` | INTEGER | NOT NULL, PART OF PRIMARY KEY |
| `user_id` | INTEGER | NOT NULL, PART OF PRIMARY KEY |
| `state` | TEXT | NOT NULL |
| `payload_json` | TEXT | NOT NULL |
| `updated_at` | INTEGER | NOT NULL |

**Primary Key**: Composite `PRIMARY KEY (chat_id, user_id)`

### system_settings

| Column | Type | Constraints |
|--------|------|-------------|
| `key` | TEXT | PRIMARY KEY |
| `value` | TEXT | NOT NULL |

**Key Entries**:
- `telegram_update_offset`: Last processed Telegram Low Watermark update_id (persisted with atomic memory + DB coordination).
- `lang_{chat_id}`: Language code preference per chat_id (e.g. `lang_123456` = `"uk"`), backed by $O(1)$ LRU cache (`lang_cache.rs`).
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
```

## Connection Pooling & Deadline Control

- **`deadpool-sqlite` Pool Acquisition Timeout**: Connection pool acquisition uses a `4500ms` deadline timeout in `enqueue_update_payload`. If DB connection pool acquisition fails under unrecoverable SQLite lock contention, Webhook returns `HTTP 503 Service Unavailable`, instructing Telegram to retry update delivery without dropping messages.

## Head-of-Line Unblocking & Atomic CAS Operations

- **Canonical Session Locking**: Single `ChatSession` per-user/chat lock protects FSM state mutations without deadlocks. For anonymous group admins (`chat_id < 0`, `user_id = 0`), lock keys scope to `(chat_id, 0)`, preventing lock starvation for regular group chat members.
- **Atomic Invoice CAS**: Status transitions use atomic Compare-And-Swap statements checking `rows_updated == 1`:
```sql
UPDATE invoices SET status = 'cancelled' WHERE id = ? AND status = 'pending'
```

## Offset & State Persistence

- **Offset Storage**: Last processed Telegram update Low Watermark offset is persisted in SQLite `system_settings` table under `telegram_update_offset` and synchronized in RAM.
- **User Language Preference**: Chat language preferences are stored under key `lang_{chat_id}` in `system_settings`, backed by an in-memory thread-safe $O(1)$ LRU cache (`lang_cache.rs`).
