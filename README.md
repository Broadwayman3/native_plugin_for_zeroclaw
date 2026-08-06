# ZeroClaw Solana POS Payment Terminal Agent

> **Tier 3 WASM Native Plugin + Squads v4 Multisig Governance**

![Tests](https://img.shields.io/badge/Tests-526%20PASSED-brightgreen?style=for-the-badge&logo=pytest)
![WASM Sandbox](https://img.shields.io/badge/Sandbox-Tier%203%20WASM%20wasip2-orange?style=for-the-badge&logo=webassembly)
![Custody](https://img.shields.io/badge/Custody-T1%20Invoicing%20%2B%20Squads%20v4-blue?style=for-the-badge&logo=solana)
![License](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)

## 10-Second Verification

```bash
# Complete automated verification (526 tests, WASM build, security checks)
./scripts/verify_all.sh
```

## Telegram Cashier Interface

```text
┌────────────────────────────────────────────────────────────────────────┐
│ Telegram POS Bot (@ZeroClawPOSBot)                                     │
├────────────────────────────────────────────────────────────────────────┤
│ Cashier: "Bill 2x Cappuccino ($8.00) and 1x Croissant ($2.00)"        │
│                                                                        │
│ Agent: *ZeroClaw POS Receipt #102*                                     │
│ • 2x Cappuccino ($8.00)                                                │
│ • 1x Croissant ($2.00)                                                 │
│ *TOTAL: $10.00 USDC*                                                   │
│                                                                        │
│ Pay URL: solana:8xAZ...mQ11?amount=10.00&spl-token=EPjF...t1v          │
│                                                                        │
│ [Customer scans QR & signs on Solana Devnet]                           │
│                                                                        │
│ Agent: *Payment Confirmed!* Invoice #102                                │
└────────────────────────────────────────────────────────────────────────┘
```

## Quickstart (15 min)

1. **Create Telegram Bot**: Chat with [@BotFather](https://t.me/BotFather), copy token to `.env` (`TELEGRAM_BOT_TOKEN`).
2. **Set Merchant Wallet & Webhook (Optional)**: Paste Solana wallet address and optional `TELEGRAM_WEBHOOK_URL` / `TELEGRAM_BOT_SECRET_TOKEN` in `.env`.
3. **Launch**: `docker-compose up -d` or `cargo run --manifest-path pos-backend/Cargo.toml`
4. **Start**: Send `/start` to your Telegram bot

## Architecture

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

## Components

| Component | Path | Description |
|-----------|------|-------------|
| WASM Plugin | [`plugins/solana-pos-core`](./plugins/solana-pos-core) | Rust crate → `wasm32-wasip2` via WIT |
| Core Logic | [`pos-core-logic`](./plugins/solana-pos-core/pos-core-logic) | Shared business logic |
| REST Backend | [`pos-backend`](./pos-backend) | Axum HTTP server (18 REST API routes), SQLite WAL, domain logic |
| Telegram Listener | [`pos-backend/src/api/telegram`](./pos-backend/src/api/telegram) | Webhook DB queue (`webhook.rs`), Long Polling with panic isolation (`polling.rs`), Atomic CAS invoice updates, Canonical `ChatSession` locks (`locks.rs`), Failover worker restart (`lifecycle.rs`), 12-factor stdout logging (`RUST_LOG`) |
| Skills | [`skills/`](./skills) | LLM skill definitions (Solana Pay, Nonce, PIX, etc.) |
| SOPs | [`sops/`](./sops) | Standard Operating Procedures (JSON) |
| WIT Interface | [`wit/v0/pos_core.wit`](./wit/v0/pos_core.wit) | WASI Component Model contract |

## Documentation

| Document | Description |
|----------|-------------|
| [Architecture](./docs/ARCHITECTURE.md) | Crate structure, WIT ABI, data flow & resilience |
| [API Reference](./docs/API.md) | 18 REST API routes, Webhook, Solana Actions v2 & x402 specification |
| [Security](./docs/SECURITY.md) | Threat model, defense matrix, DLQ mechanics |
| [Database](./docs/DATABASE.md) | Schema, migrations, pragmas & atomic CAS |
| [Deployment](./docs/DEPLOYMENT.md) | Docker, local dev, env vars & RUST_LOG filtering |
| [Testing](./docs/TESTING.md) | Test strategy, module breakdown (526 tests) |

## Security

- **Tier 1 Non-Custodial**: Direct customer-to-merchant settlement via Solana Pay
- **Tier 3 WASM Sandbox**: Rust plugin in isolated `wasm32-wasip2` environment
- **Squads v4 Multisig**: Agent = Proposer only; managers hold threshold signers
- **Triple Payment Verification**: Reference key + token mint + amount check
- **526 automated tests** (481 in pos-backend, 31 in pos-core-logic, 8 in solana-pos-core WASM) including prompt injection defense

## License

MIT
