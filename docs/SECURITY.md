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
| Webhook Outage | Webhook endpoint disruption | Circuit Breaker: automatic failover to Long Polling after 3 consecutive failures | Mitigated |

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

## Input Sanitization

All external input (customer names, memos, merchant tags) is sanitized via:
- Control character stripping (`\x00-\x1f`)
- Prompt injection pattern detection
- Zero-width Unicode character removal
- NFKC normalization (not NFC)

## Security Audit Results

6/6 prompt injection tests passed:
- Jailbreak attack (system prompt override)
- Manager impersonation
- Secret key extraction
- Daily limit bypass
- Fake payment confirmation injection
- Squads v4 direct transfer bypass

Full audit log: 6/6 prompt injection tests passed (SEC-01 through SEC-06).
