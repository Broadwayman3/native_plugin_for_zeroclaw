# Security Architecture

## Threat Model Matrix

| Attacker Role | Attack Vector | Defense Mechanism | Status |
|---|---|---|---|
| Prompt Injector | System prompt override | ZeroClaw Context Isolation & AST Sanitizer | Mitigated |
| Chat Impersonator | Impersonate store owner | Telegram ID validation against MANAGER_TELEGRAM_ID | Mitigated |
| Malicious User | Extract secrets | config_read sandbox; secrets never passed to LLM | Mitigated |
| Draining Attacker | Massive refund request | Hardcoded limit (max_single_refund_usdc = 50.0) | Mitigated |
| Text Spoofer | Fake payment confirmation | Agent ignores text; payment verified via RPC polling only | Mitigated |
| Vault Attacker | Bypass Squads v4, drain vault | Agent restricted to Proposer role; execution requires threshold signers | Mitigated |
| Dusting Attacker | 1-lamport micro-dusting | Triple Payment Verification: reference + mint + amount >= expected | Mitigated |
| Fake Token Spoofer | Pay with fake SPL token | Triple Payment Verification: strictly enforces USDC Mint | Mitigated |
| Nonce Collision | Parallel refund approvals | Nonce Account Pool: unique nonce per pending approval | Mitigated |
| Context Flooder | Flood LLM context window | Context truncator: caps payload size (<150 tokens) | Mitigated |
| OOM Flood Attacker | Massive update spam (1000s/sec) | Per-chat bounded MPSC channels (capacity 64) with lightweight DLQ fallback | Mitigated |
| Panic Freeze Attacker | Trigger runtime panic in update handler | `inner_handle` JoinHandle panic isolation; DLQ commitment advances Low Watermark | Mitigated |
| Replay / Stale Attacker | Replay old Telegram updates | `stale_update_ttl_secs` timestamp validation with clock-skew check (`msg_date >= now`) | Mitigated |
| Webhook DoS Attacker | Huge body / memory exhaustion | Webhook Body Limit: strict 128 KB request body size cap | Mitigated |
| Secret Token Spoofer | Fake Telegram webhook POSTs | Constant-time token comparison (`subtle::ConstantTimeEq`) on `X-Telegram-Bot-Api-Secret-Token` | Mitigated |
| Webhook Failure / Data Loss | DB exhaustion / dropped update | Synchronous WAL insert with 4.5s pool acquire timeout returning HTTP 500 for gateway retries | Mitigated |
| Transaction Panic Leak | Uncommitted SQLite transaction on panic | `TransactionRollbackGuard`: RAII `Drop` implementation executes `ROLLBACK` automatically on panic | Mitigated |
| Anonymous Admin Race | Supergroup admin FSM collision | `admin_session.rs`: Stateless mode (`user_id = 0`) for `from`-less messages & linked channel posts | Mitigated |
| NTP Time Drift | Rate limiter pause freeze/panic | `rate_limiter.rs`: Monotonic `tokio::time::Instant` timer with auto-reset guard | Mitigated |

## Custody Architecture

- **Tier 1 (Payments)**: Direct customer-to-merchant wallet settlement via Solana Pay URLs
- **Tier 3 (WASM Core)**: Rust plugin compiled to WASI WebAssembly sandbox
- **Squads v4 Multisig**: Agent operates solely as `Proposer`. Store managers hold threshold signers; key theft cannot drain funds

## Triple Payment Verification

All payment confirmations are verified against three conditions:

1. **Reference Key Matching**: Transaction must include the invoice's unique Ed25519 reference public key
2. **Token Mint Enforcement**: Token transfer mint must exactly match USDC Mint (`EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v` for mainnet, `4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU` for devnet)
3. **Amount Sufficiency**: `paid_amount_atomic_units >= expected_amount_atomic_units`

## SSRF Protection

`validate_safe_rpc_url` blocks:
- Private IP ranges: `127.0.0.1`, `192.168.x.x`, `10.x.x.x`
- Cloud metadata: `169.254.169.254`
- Loopback: `localhost`, `::1`
- IPv6 reserved: `fe80::/10`, `fc00::/7`, `2001:db8::/32`

## Dead-Letter Queue (DLQ) & Failure Handling

When processing incoming webhook updates from SQLite FIFO queue (`pending_webhook_updates`) or Long Polling updates:
1. Updates are attempted up to **3 times** with exponential retry delay.
2. If processing fails 3 consecutive times OR if a panic occurs, the update is moved to the **`failed_updates`** table (Dead-Letter Queue).
3. `watermark_guard.complete()` is invoked **strictly after DLQ commitment**, guaranteeing Low Watermark advancement without data loss or offset freeze.
4. A sanitized rate-limit / network notice is dispatched to the user/chat to inform them of the status.

## Telegram Defense Matrix

1. **Input Sanitization (`sanitize_external_input`)**: All incoming Telegram user text undergoes NFKC normalization, Cyrillic homoglyph stripping, zero-width space removal, and prompt-injection regex scrubbing.
2. **Link-Aware MarkdownV2 Escaping (`escape_telegram_markdown_v2_preserve_links`)**: Escapes MarkdownV2 reserved characters while preserving valid URI links (`solana:`, `solana:pay`, `https:`). Template code blocks (`/refund {} 1.0`) preserve literal dots and dashes inside backticks to guarantee clean 1-click copy-paste execution.
3. **Low Watermark Offset Tracking (`WatermarkTracker`)**: In-flight update IDs are tracked in `BTreeSet<i64>`. Offset advances to `min(pending_ids)`, eliminating head-of-line batch blocking.
4. **Per-Chat Bounded MPSC Queues (`ChatQueueDispatcher`)**: Bounded capacity 64 per chat channel guarantees strict session FIFO order and bounds RAM usage. Idle chat channels are purged after 60s inactivity.
5. **Inner JoinHandle Panic Isolation (`polling.rs`)**: Inner tasks run inside `tokio::spawn`. Panics (`is_panic()`) are caught, logged, and isolated in SQLite DLQ without freezing offset progression.
6. **In-Memory Idempotency Cache (`UpdateIdempotencyCache`)**: Atomic LRU cache in RAM checks `is_update_processed` before executing DB queries, preventing redundant processing.
7. **Stateless Supergroup Admin Handling (`admin_session.rs`)**: Messages from anonymous group admins or linked channel forwards (missing `from` field) run in Stateless One-Shot Mode (`user_id = 0`), preventing FSM cross-contamination.
8. **Fast-Track Callback Queries (`callbacks.rs`)**: `answerCallbackQuery` is acknowledged immediately prior to DB locking operations, avoiding `query is too old` Telegram client timeouts.

## Security Audit Results

6/6 prompt injection tests passed:
- Jailbreak attack (system prompt override)
- Manager impersonation
- Secret key extraction
- Daily limit bypass
- Fake payment confirmation injection
- Squads v4 direct transfer bypass

Full audit log: 6/6 prompt injection tests passed (SEC-01 through SEC-06).
