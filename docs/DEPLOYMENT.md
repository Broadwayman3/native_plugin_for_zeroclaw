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

## Reverse Proxy & Webhook Configuration

When deploying behind Nginx, Cloudflare, or Caddy reverse proxies:

```nginx
location /api/v1/telegram/webhook {
    proxy_pass http://127.0.0.1:8080;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Telegram-Bot-Api-Secret-Token $http_x_telegram_bot_api_secret_token;
    
    # Must be > 65 seconds to accommodate the 60s backend Solana RPC execution wrapper timeout
    proxy_read_timeout 70s;
    proxy_connect_timeout 10s;
}
```

> [!IMPORTANT]
> **Production Reverse Proxy Ops Note**: The reverse proxy `proxy_read_timeout` **MUST** be set to at least **65–70 seconds**. Both Poller and Webhook workers wrap update dispatches in a 60-second `tokio::time::timeout` to accommodate Solana RPC node latency during on-chain invoice/Squads proposal creation. A smaller timeout (e.g. 15s or 30s) will cause Nginx to drop HTTP connections before Axum returns `200 OK`.

> [!NOTE]
> Webhook endpoints return `HTTP 200 OK` for duplicate or ignored updates during mode transitions to prevent Head-of-Line (HoL) delivery blocking in Telegram API. `HTTP 503 Service Unavailable` is returned strictly on unrecoverable SQLite connection pool failures.

## Environment Variables

| Variable | Description | Example / Default |
|----------|-------------|-------------------|
| `TELEGRAM_BOT_TOKEN` | Telegram bot token from @BotFather | `8861640052:AAH...` |
| `TELEGRAM_WEBHOOK_URL` | Optional Telegram webhook endpoint URL (enables Webhook mode) | `https://your-domain.com/api/v1/telegram/webhook` |
| `TELEGRAM_BOT_SECRET_TOKEN` | Secret token for constant-time webhook header validation | `sec_token_8861640052...` |
| `STALE_UPDATE_TTL_SECS` | Stale update TTL threshold in seconds (0 = disabled) | `300` (5 minutes) |
| `MANAGER_TELEGRAM_ID` | Telegram user ID for manager approvals and settings mutations | `123456789` |
| `SOLANA_RPC_URL` | Solana RPC endpoint (Helius / QuickNode / Devnet) | `https://api.devnet.solana.com` |
| `SOLANA_FALLBACK_RPC_URL` | Secondary fallback RPC URL | `https://devnet.helius-rpc.com/?api-key=...` |
| `MERCHANT_WALLET_PUBKEY` | Store wallet public key (Tier 1 destination for funds) | `8xAZ...mQ11` |
| `REFUND_SESSION_KEY` | Tier 2 Session key secret array for non-custodial refunds | `[12,34,56,...]` |
| `NONCE_ACCOUNT_PUBKEY` | Pre-funded Solana Nonce Account pubkey | `Nonce111111111111111111111111111111111111111` |
| `SQUADS_MULTISIG_PUBKEY` | Squads v4 multisig account | `SQDS4ep65...` |
| `SQUADS_VAULT_PUBKEY` | Squads v4 vault account | `9xK2...` |
| `USDC_MINT_ADDRESS` | USDC token mint address | `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v` |
| `QUICK_RECEIPT_AMOUNT` | Default amount for quick receipt POS button | `200.0` |
| `QUICK_RECEIPT_CURRENCY` | Default currency code for quick receipts | `"UAH"` |
| `ALLOW_LOCAL_RPC` | Allow local HTTP RPC endpoints for dev testing | `false` |
| `API_KEYS` | API keys for authenticated REST API routes | `"key1,key2"` (Comma-separated string) |
| `RUST_LOG` | Log filtering spec (reduces 3rd-party crate noise) | `pos_backend=info,pos_core_logic=info,reqwest=warn,hyper=warn,h2=warn,tower_http=warn,deadpool_sqlite=warn` |
| `NO_COLOR` | Optional: Disable ANSI color codes for log aggregators | `1` |

### Configuration Parsing Details (`API_KEYS`)

> [!IMPORTANT]
> - In environment variables (`.env`), `API_KEYS` is parsed as a **comma-separated string** (e.g. `API_KEYS="key1,key2"`).
> - In TOML configuration (`config.toml`), `api_keys` is specified as a **native TOML array of strings** (e.g. `api_keys = ["key1", "key2"]`).

## Configuration File (`config.toml`)

`config.toml` contains:
- Agent metadata (name, description, version)
- Telegram channel settings (`bot_token`, `webhook_url`, `stale_update_ttl_secs`, `manager_chat_id`)
- POS quick receipt defaults (`quick_receipt_amount`, `quick_receipt_currency`)
- Solana RPC/commitment configuration (`rpc_url`, `fallback_rpc_url`, `merchant_wallet`, `allow_local_rpc`)
- Squads v4 program ID and vault settings
- Security limits (`max_single_refund_usdc`, `daily_refund_limit_usdc`, `api_keys`)
- Skills/SOPs/Plugins directory paths

## Troubleshooting

### Database locked

SQLite connections are initialized with `PRAGMA busy_timeout = 5000;` and `PRAGMA journal_mode = WAL;`. If database locks occur under heavy load, verify connection pool acquisition limits in `DbPool`.

### WASM build fails

Ensure `wasm32-wasip2` target is installed:
```bash
rustup target add wasm32-wasip2
```

### Nonce pool exhausted

Allocate more nonce accounts in the seed data or release locked accounts via `POST /api/v1/nonce/sync`.
