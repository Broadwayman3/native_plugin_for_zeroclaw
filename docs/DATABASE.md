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
| `created_at` | TIMESTAMP | DEFAULT CURRENT_TIMESTAMP |
| `updated_at` | TIMESTAMP | DEFAULT CURRENT_TIMESTAMP |

**Status values**: `pending`, `paid`, `cancelled`, `partially_paid`

**Indexes**: `idx_invoices_tx_sig` on `tx_signature` WHERE `tx_signature IS NOT NULL` (partial unique)

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

### processed_updates

| Column | Type | Constraints |
|--------|------|-------------|
| `update_id` | INTEGER | PRIMARY KEY |
| `processed_at` | TIMESTAMP | DEFAULT CURRENT_TIMESTAMP |

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
