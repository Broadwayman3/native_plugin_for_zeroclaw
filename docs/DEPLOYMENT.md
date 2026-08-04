# Deployment

## Docker (Recommended)

```bash
docker-compose up -d
```

This starts two services:
- `pos-backend` (Axum on port 8080)
- `zeroclaw-agent` (ZeroClaw host, depends on pos-backend)

## Local Development

### Prerequisites

- Rust 1.77+ with `wasm32-wasip2` target
- `wasm-tools` (optional, for WASI validation)

### Setup

```bash
./scripts/setup.sh
```

Copies `.env.example` → `.env` and `config.example.toml` → `config.toml` if not present.

### Build WASM Plugin

```bash
./scripts/build_wasm.sh
```

### Start Backend

```bash
cargo run --manifest-path pos-backend/Cargo.toml
```

Server starts on `http://localhost:8080`.

### Run Verification

```bash
./scripts/verify_all.sh
```

## Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `TELEGRAM_BOT_TOKEN` | Telegram bot token from @BotFather | `8861640052:AAH...` |
| `MANAGER_TELEGRAM_ID` | Telegram user ID for manager approvals | `123456789` |
| `SOLANA_RPC_URL` | Solana RPC endpoint | `https://devnet.helius-rpc.com/?api-key=...` |
| `MERCHANT_WALLET_PUBKEY` | Store wallet public key | `8xAZ...mQ11` |
| `SQUADS_MULTISIG_PUBKEY` | Squads v4 multisig account | `SQDS4ep65...` |
| `SQUADS_VAULT_PUBKEY` | Squads v4 vault account | `9xK2...` |
| `USDC_MINT_PUBKEY` | USDC token mint | `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v` |

## Configuration

`config.toml` contains:
- Agent metadata (name, description)
- Telegram channel settings
- Solana RPC/commitment configuration
- Squads v4 program ID
- Security limits (max_single_refund_usdc)
- Skills/SOPS/Plugins directory paths

## Troubleshooting

### Database locked

Increase `busy_timeout` in `schema.rs` or check for long-running transactions.

### WASM build fails

Ensure `wasm32-wasip2` target is installed:
```bash
rustup target add wasm32-wasip2
```

### Nonce pool exhausted

Allocate more nonce accounts in the seed data or release locked accounts.
