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
| Webhook DoS Attacker | Huge body / memory exhaustion | Webhook Body Limit: strict 64 KB request body size cap | Mitigated |
| Secret Token Spoofer | Fake Telegram webhook POSTs | Constant-time token comparison (`constant_time_eq`) on `X-Telegram-Bot-Api-Secret-Token` | Mitigated |
| Webhook Failure / Data Loss | DB exhaustion / dropped update | Synchronous WAL insert with 4.5s pool acquire timeout returning HTTP 500 for gateway retries | Mitigated |
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

When processing incoming webhook updates from SQLite FIFO queue (`pending_webhook_updates`):
1. Updates are attempted up to **3 times** with exponential retry delay.
2. If processing fails 3 consecutive times, the update is moved to the **`failed_updates`** table (Dead-Letter Queue).
3. A sanitized notification is dispatched to the user/chat to inform them of the processing error.

## Telegram Defense Matrix

1. **Input Sanitization (`sanitize_external_input`)**: All incoming Telegram user text undergoes NFKC normalization, Cyrillic homoglyph stripping, zero-width space removal, and prompt-injection regex scrubbing.
2. **MarkdownV2 Escaping (`escape_telegram_markdown_v2`)**: All outgoing response text is escaped against MarkdownV2 reserved characters (`_`, `*`, `[`, `]`, `(`, `)`, `~`, `` ` ``, `>`, `#`, `+`, `-`, `=`, `|`, `{`, `}`, `.`, `!`) to prevent formatting syntax injection into Telegram client UI.
3. **Invoice Lock Isolation (`extract_invoice_id`)**: Extracting `INV-` tokens from callback queries or commands routes synchronization through `LockKey::Invoice(invoice_id)`, preventing group chat session deadlocks when multiple users interact with a shared invoice.
4. **Per-Chat Rate Limiting & GC Worker (`rate_limiter.rs`)**: Outbound Telegram requests use sliding-window rate limiting (`Priority::Normal`) with periodic 10-minute GC passes (`retain_recent_keys()`) and global HTTP 429 monotonic pause signals.
5. **Stateless Supergroup Admin Handling (`admin_session.rs`)**: Messages from anonymous group admins or linked channel forwards (missing `from` field) run in Stateless One-Shot Mode (`user_id = 0`), preventing FSM cross-contamination while preserving `from.id` authorization for callback queries.
6. **Fast-Track Callback Queries (`callbacks.rs`)**: `answerCallbackQuery` is acknowledged immediately prior to DB locking operations, avoiding `query is too old` Telegram client timeouts.
7. **Task Bounded Execution & Poller Retries (`polling.rs`)**: Polling update handles execute within a 30-second `tokio::time::timeout`. Failed tasks retry DB error logging up to 3 times before monotonically advancing update offsets.

## Security Audit Results

6/6 prompt injection tests passed:
- Jailbreak attack (system prompt override)
- Manager impersonation
- Secret key extraction
- Daily limit bypass
- Fake payment confirmation injection
- Squads v4 direct transfer bypass

Full audit log: 6/6 prompt injection tests passed (SEC-01 through SEC-06).
