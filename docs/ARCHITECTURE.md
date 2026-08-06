# Architecture

## Overview

ZeroClaw Solana POS Agent is an autonomous AI cash register operating in Telegram/WhatsApp. The backend is written entirely in Rust, with a WASM Tier 3 plugin for cryptographic operations.

```
┌──────────────────────────────────────────────────────────────┐
│                     ZeroClaw Host Runtime                      │
│  ┌──────────────────┐     ┌──────────────────────────────┐   │
│  │ Telegram Channel │────▶│ SOP Engine (cron, approvals) │   │
│  └──────────────────┘     └──────────┬───────────────────┘   │
│                                      │ HTTP                   │
│  ┌───────────────────────────────────▼─────────────────────┐ │
│  │              pos-backend (Rust Binary)                    │ │
│  │  ┌─────────┐  ┌──────────┐  ┌────────────────────────┐ │ │
│  │  │  Axum   │  │ rusqlite │  │  pos-core-logic        │ │ │
│  │  │  REST   │  │  SQLite  │  │  (shared with WASM)    │ │ │
│  │  │  API    │  │  WAL     │  │  - Token-2022 fees     │ │ │
│  │  │         │  │          │  │  - Solana Pay URL      │ │ │
│  │  │         │  │          │  │  - Squads v4 Borsh     │ │ │
│  │  └─────────┘  └──────────┘  └────────────────────────┘ │ │
│  └─────────────────────────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │  WASM Plugin (solana-pos-core) ← calls pos-core-logic   │ │
│  └─────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────┘
```

## Crate Structure

### pos-backend

Main binary crate. Provides REST API (18 REST API routes), database layer, and domain logic.

```
pos-backend/src/
├── main.rs              # Axum HTTP server entrypoint
├── lib.rs               # Crate root
├── config.rs            # AppConfig struct (stale_update_ttl_secs, quick_receipt, env loader)
├── error.rs             # AppError enum (thiserror)
├── api/                 # REST endpoints (18 routes) & Telegram listener
│   ├── mod.rs           # Router builder, CORS, AppState, stale update TTL filter
│   ├── actions.rs       # Solana Actions/Blinks (Dialect v2 spec)
│   ├── invoices.rs      # Invoice CRUD & settings handlers
│   ├── nonce.rs         # Durable nonce pool management
│   ├── pos_flow.rs      # POS order creation
│   ├── sales.rs         # Sales summary & x402 machine commerce
│   ├── x402.rs          # x402 micropayment gated analytics
│   └── telegram/        # Telegram Bot API integration & listener
│       ├── mod.rs       # Telegram exports, update processor & stale TTL check
│       ├── admin_session.rs # Group admin detection & context extraction
│       ├── chat_action.rs # SendChatAction typing status helper
│       ├── client.rs    # Reqwest Telegram API client & multipart photo senders
│       ├── client_queue.rs # Rate-limited outbound message queue manager
│       ├── events.rs    # Telegram update event dispatching & callback routing
│       ├── fsm.rs       # Telegram FSM state types
│       ├── fsm_store.rs # Persistent Telegram FSM DAO
│       ├── handlers/    # Telegram command & callback query handlers
│       ├── lang_cache.rs # Thread-safe O(1) lru::LruCache for user language preferences
│       ├── lifecycle.rs # Service spawner & Webhook to Polling circuit breaker failover
│       ├── locks.rs     # WatermarkTracker (Low Watermark), ChatQueueDispatcher (bounded 64)
│       ├── orders.rs    # POS text order parsing & receipt builder
│       ├── polling.rs   # Long Polling worker with Low Watermark & JoinHandle panic isolation
│       ├── qr.rs        # Inline QR code receipt PNG generator
│       ├── rate_limiter.rs # Keyed rate-limiter GC worker & global HTTP 429 pause timer
│       ├── state.rs     # Update offset & language preference DB operations
│       ├── verifier.rs  # Solana RPC invoice payment verifier loop
│       ├── webhook.rs   # Webhook POST handler returning 500 on DB acquire timeout (>4.5s)
│       ├── webhook_db.rs # Webhook DB queue DAO & UpdateIdempotencyCache in RAM
│       └── webhook_worker.rs # Webhook queue worker with Semaphore(50) backpressure
├── db/                  # SQLite data access (WAL mode & PRAGMA busy_timeout=5000)
│   ├── mod.rs           # Connection factory & pool creation
│   ├── schema.rs        # DDL, migrations, nonce seeding, idx_pending_fifo
│   ├── invoices.rs      # Invoice DAO
│   ├── nonce.rs         # Nonce account pool
│   ├── squads.rs        # Squads v4 proposals DAO
│   ├── fsm_dao.rs       # Telegram FSM sessions DAO
│   ├── sop_checkpoints.rs # SOP execution state checkpoints
│   ├── updates.rs       # FIFO update queue, DLQ, deduplication & max retry recording
│   └── seed.rs          # Sample data
└── domain/              # Business logic
    ├── constants.rs     # USDC/SOL mints, Base58 alphabet
    ├── sanitizer.rs     # SSRF guard, input sanitization, link-aware MarkdownV2
    ├── verification.rs  # Triple Payment Verification
    ├── i18n.rs          # 13-language i18n dispatcher
    ├── i18n_strings/    # Translation tables (13 languages)
    ├── validators.rs    # Input validators
    ├── price_feed.rs    # Multi-tier fiat rate fallback (Jupiter → Switchboard → cache)
    ├── keyboards.rs     # Telegram inline keyboards
    ├── order_parser.rs  # POS text → order parser
    ├── formatters.rs    # QR URLs, Base58, pubkey formatting
    └── pix_brl.rs       # Brazil PIX QR code
```

### pos-core-logic

Shared library crate. Contains business logic used by both the backend and the WASM plugin.

```
plugins/solana-pos-core/pos-core-logic/src/
├── lib.rs               # Re-exports all modules
├── constants.rs         # Shared constants (mints, program IDs)
├── solana_pay.rs        # Solana Pay URL builder, reference key generator
├── squads.rs            # Squads v4 multisig proposal instruction builder
└── token2022.rs         # Token-2022 transfer fee calculator (u128 precision)
```

### solana-pos-core (WASM Plugin)

Compiled to `wasm32-wasip2` via WIT contract interface. Runs in ZeroClaw's Tier 3 WASM sandbox.

```
plugins/solana-pos-core/src/
└── lib.rs               # WASM entrypoint (wit-bindgen), exports 3 functions
```

## WIT ABI Contract

Defined in `wit/v0/pos_core.wit`:

```wit
package zeroclaw:plugin@0.1.0;

interface pos-core {
    record invoice-request {
        merchant-pubkey: string,
        amount-usdc: f64,
        reference-pubkey: string,
        spl-token-mint: string,
    }

    record invoice-instruction-result {
        success: bool,
        solana-pay-url: string,
        reference-key: string,
        token2022-fee-usdc: f64,
        error: option<string>,
    }

    record squads-proposal-request {
        multisig-pubkey: string,
        vault-pubkey: string,
        proposer-pubkey: string,
        recipient-pubkey: string,
        amount-usdc: f64,
        proposal-index: u64,
        memo: string,
    }

    record squads-proposal-result {
        success: bool,
        proposal-tx-base64: string,
        proposal-index: u64,
        error: option<string>,
    }

    build-solana-pay-instruction: func(req: invoice-request) -> invoice-instruction-result;
    calculate-token2022-fee: func(amount: f64, fee-basis-points: u16, max-fee: u64, decimals: u8) -> f64;
    build-squads-v4-proposal: func(req: squads-proposal-request) -> squads-proposal-result;
}

world plugin {
    export pos-core;
}
```

## Data Flow & Resilience

### Telegram Listener Dataflow & Update Processing Architecture

```
Telegram Gateways / API
         │
         ▼
Poller / Webhook Listener (guarded by PollerActiveGuard RAII)
         │
┌────────┴──────────────────────────┐
│ WatermarkTracker (BTreeSet<i64>)  │ ◄── Tracks Low Watermark (Min Unconfirmed ID)
└────────┬──────────────────────────┘
         │ async enqueue_timeout (2s timeout, Semaphore(100) OOM guard)
┌────────▼──────────────────────────┐
│ ChatQueueDispatcher (MPSC cap=64) │ ◄── Strict Per-Chat FIFO Order
└────────┬──────────────────────────┘
         │
┌────────▼──────────────────────────┐
│ inner_handle (tokio::spawn)       │ ◄── Panic Isolation & 30s Timeout
└────────┬──────────────────────────┘
         │
   ┌─────┴────────────────┐
   ▼                      ▼
Success               DLQ Commit
   │                      │
   └──────────┬───────────┘
              ▼
watermark_guard.complete() ──▶ Advance Offset in SQLite & RAM (Unconditional on Timeout)
```

1. **Idempotency Registration & Single-Lock LRU**: Incoming `update_id`s are checked against `UpdateIdempotencyCache` in RAM via `check_and_mark_processed(update_id)` under a single `Mutex` lock before hitting SQLite `processed_updates`.
2. **Stale Update & Callback TTL Filter**: Top-level `message` and `edited_message` timestamps (`edit_date`) are validated against `stale_update_ttl_secs` (default 300s) with clock-skew tolerance. `callback_query` inline button clicks are checked against TTL; expired buttons trigger `answerCallbackQuery("⚠️ Action expired")` to release the Telegram UI spinner.
3. **Low Watermark Offset Tracking**: `WatermarkTracker` tracks in-flight `update_id`s in a `BTreeSet`. Persistent offset advances to the Low Watermark (`min(pending_ids)`), allowing continuous polling without head-of-line batch blocking. Watermark completion is unconditional on timeout to eliminate update offset freezing.
4. **Per-Chat Bounded FIFO Queue Dispatcher & Backpressure**: `ChatQueueDispatcher` queues tasks in session-bound `mpsc::channel(64)` channels, guaranteeing strict FIFO order within each chat without blocking the Poller loop. Dispatch tasks use a 2-second `enqueue_timeout` backpressure wait inside non-blocking tasks guarded by `Semaphore(100)` for OOM protection. Idle queues are safely purged after 60s of inactivity.
5. **Panic-Safe Isolation & RAII Guard**: `PollerActiveGuard` implements `Drop` to guarantee `IS_POLLER_ACTIVE` is restored to `false` even on poller task panic. Inner update tasks are spawned via `tokio::spawn`. Panics (`is_panic()`) are caught, logged, and isolated in SQLite DLQ (`failed_updates`) without halting Low Watermark progression.
6. **Circuit Breaker Failover**: Webhook registration failures trip a 5-minute circuit breaker (`WEBHOOK_COOLDOWN_SECS = 300`). SQLite `pending_webhook_updates` are drained before starting Long Polling worker.
7. **Strict Lock Hierarchy (Deadlock Prevention)**: Global session locks follow `chat_lock` -> `invoice_lock`. Background worker tasks (RPC `verifier.rs`, Squads watcher) must NEVER acquire `chat_lock` after acquiring `invoice_lock`.

### Payment Flow

1. Customer sends invoice request via Telegram (Webhook queue or Long Polling)
2. `pos_flow.rs` / `orders.rs` parses text → order JSON
3. `price_feed.rs` fetches fiat→USDC rate (Jupiter → Switchboard → cache → static fallback)
4. `pos-core-logic` builds Solana Pay URL + calculates Token-2022 fee
5. Invoice saved to SQLite (status: `pending`)
6. QR code returned to customer via Telegram inline keyboard
7. Customer pays via Solana wallet
8. In-process Tokio verifier worker (`verifier.rs`) polls Solana RPC for reference key signatures
9. Triple Payment Verification confirms: reference key + token mint + amount
10. Invoice status → `paid`

### Refund Flow

1. Customer requests refund
2. `pos-core-logic` builds Squads v4 proposal (agent = Proposer only)
3. Nonce account allocated from pool
4. Human Approval Checkpoint: Telegram notification to manager
5. Manager signs in Phantom/Squads App
6. Squads v4 executes transfer from vault

## Dependencies

| Crate | Key Dependencies |
|-------|-----------------|
| `pos-backend` | axum, rusqlite (bundled), tokio, serde, regex, thiserror, tracing, tracing-subscriber, deadpool-sqlite |
| `pos-core-logic` | serde, rand |
| `solana-pos-core` | wit-bindgen 0.30.0, pos-core-logic (path) |
