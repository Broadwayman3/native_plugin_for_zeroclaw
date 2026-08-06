# REST API & Webhook Specification

Base URL: `http://localhost:8080`

Total Endpoints: **18 REST API routes (19 handlers)**

---

## 1. System & Health

### Health Check (GET)
```http
GET /healthz
```
- **Description**: Lightweight health check endpoint for container orchestrators (Kubernetes / Docker).
- **Response**: `200 OK`

### Get Settings (GET)
```http
GET /api/v1/settings
```
- **Description**: Returns current merchant configuration, quick receipt defaults, and accepted currency.
- **Response**:
```json
{
  "quick_receipt_amount": 200.0,
  "quick_receipt_currency": "UAH",
  "merchant_wallet_pubkey": "8xAZnR2pMQR3Qv5xK8c7mQ11rF4eG7hJ9kL2nP4s"
}
```

### Update Settings (POST) - Manager Auth
```http
GET /api/v1/settings/update
```
- **Headers**: `X-Telegram-User-Id: <manager_id>`
- **Description**: Updates quick receipt configuration. Requires `X-Telegram-User-Id` matching `MANAGER_TELEGRAM_ID`.
- **Request Payload**:
```json
{
  "quick_receipt_amount": 250.0,
  "quick_receipt_currency": "UAH"
}
```
- **Response**: `200 OK`

---

## 2. Telegram Webhook & Long Polling Listener

### Receive Webhook Update (POST)
```http
POST /api/v1/telegram/webhook
```
- **Headers**: `X-Telegram-Bot-Api-Secret-Token: <token>` (validated via constant-time string comparison `subtle::ConstantTimeEq`)
- **Body Limit**: 128 KB maximum payload limit
- **Description**: Enqueues incoming Telegram update synchronously into SQLite WAL database (`webhook.rs`) with a `4500ms` connection acquire timeout and triggers async worker wakeup (`webhook_notify`).
- **Response**:
  - `200 OK`: Update successfully stored in SQLite queue.
  - `401 Unauthorized`: Missing or invalid secret token header.
  - `500 Internal Server Error`: Connection pool acquisition timed out (>4.5s) or DB write failed. Signals Telegram gateway to retry delivery.

### Dual Mode & Circuit Breaker Failover Spec
- **Webhook Mode**: Used when `TELEGRAM_WEBHOOK_URL` is set in configuration. Registers webhook with Telegram API on startup.
- **Long Polling Mode**: Used when `TELEGRAM_WEBHOOK_URL` is omitted OR when Webhook registration fails. Calls `deleteWebhook` and initiates `getUpdates?offset={low_watermark}&timeout=20` long-poll loop.
- **Circuit Breaker Failover**: Webhook registration failures or network issues trip a 5-minute circuit breaker (`WEBHOOK_COOLDOWN_SECS = 300`). Pending DB updates are drained before falling back to Long Polling.
- **Rate-Limiting & Queue Backpressure**:
  - Per-chat bounded MPSC channels (capacity 64) enforce strict FIFO order without head-of-line blocking across chats.
  - When a chat queue reaches full capacity (64 items), the system records a DLQ failure (`"Per-chat queue capacity full (64)"`) and returns an automated Telegram rate-limit notice:
    `"⚠️ Too many commands in progress. Please wait a few seconds."`
- **Stale Update Filtering**: `STALE_UPDATE_TTL_SECS` (default 300s) rejects old top-level `message` and `edited_message` payloads with clock-skew tolerance (`msg_date >= now`). `callback_query` inline menu actions and system updates are exempted.

---

## 3. Invoices

### List Invoices (GET)
```http
GET /api/v1/invoices?id=INV-a6f49762&status=pending
```
- **Query Params**: `id` (optional), `status` (optional: `pending`, `paid`, `cancelled`, `expired`)
- **Response**:
```json
[
  {
    "id": "INV-a6f49762",
    "reference_pubkey": "RefKey1111111111111111111111111111111111111",
    "fiat_currency": "USD",
    "fiat_amount": 10.0,
    "usdc_amount": 10.0,
    "status": "pending",
    "tx_signature": null,
    "created_at": "2026-08-06T19:00:00Z"
  }
]
```

### Create Invoice (POST)
```http
POST /api/v1/invoices/create
```
- **Headers**: `X-Api-Key: <key>`
- **Request Payload**:
```json
{
  "id": "INV-a6f49762",
  "reference_pubkey": "RefKey1111111111111111111111111111111111111",
  "fiat_currency": "USD",
  "fiat_amount": 10.0,
  "usdc_amount": 10.0,
  "telegram_chat_id": 123456789,
  "telegram_msg_id": 42
}
```
- **Response**:
```json
{
  "id": "INV-a6f49762",
  "reference_pubkey": "RefKey1111111111111111111111111111111111111",
  "fiat_currency": "USD",
  "fiat_amount": 10.0,
  "usdc_amount": 10.0,
  "status": "pending",
  "solana_pay_url": "solana:8xAZnR2pMQR3Qv5xK8c7mQ11rF4eG7hJ9kL2nP4s?amount=10.00&spl-token=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v&reference=RefKey1111111111111111111111111111111111111",
  "tx_signature": null,
  "created_at": "2026-08-06T19:00:00Z"
}
```

### Update Invoice Status (POST)
```http
POST /api/v1/invoices/update_status
```
- **Headers**: `X-Api-Key: <key>`
- **Description**: Atomically transitions invoice status using `UPDATE invoices SET status=? WHERE id=? AND status='pending'`.
- **Request Payload**:
```json
{
  "id": "INV-a6f49762",
  "status": "paid",
  "tx_signature": "5K8c7mQ11rF4eG7hJ9kL2nP4s..."
}
```
- **Response**: `200 OK`

### Cancel Invoice (POST)
```http
POST /api/v1/invoices/cancel
```
- **Headers**: `X-Api-Key: <key>`
- **Request Payload**:
```json
{
  "id": "INV-a6f49762"
}
```
- **Response**: `200 OK`

### Verify Transaction (POST)
```http
POST /api/v1/invoices/verify-transaction
```
- **Headers**: `X-Api-Key: <key>`
- **Description**: Runs Triple Payment Verification against Solana RPC transaction meta.
- **Request Payload**:
```json
{
  "invoice_id": "INV-a6f49762",
  "transaction_meta": { ... }
}
```
- **Response**:
```json
{
  "is_valid": true,
  "verified_amount": 10.0
}
```

---

## 4. Refund & Governance (Manager Auth)

### Approve Refund (POST)
```http
POST /api/v1/refund/approve
```
- **Headers**: `X-Telegram-User-Id: <manager_id>`
- **Request Payload**:
```json
{
  "invoice_id": "INV-a6f49762",
  "amount_usdc": 10.0
}
```
- **Response**: `200 OK`

### Reject Refund (POST)
```http
POST /api/v1/refund/reject
```
- **Headers**: `X-Telegram-User-Id: <manager_id>`
- **Request Payload**:
```json
{
  "invoice_id": "INV-a6f49762",
  "reason": "Customer request expired"
}
```
- **Response**: `200 OK`

---

## 5. Nonce Pool

### Allocate Nonce (POST)
```http
POST /api/v1/nonce/allocate
```
- **Headers**: `X-Api-Key: <key>`
- **Description**: Atomically allocates a free durable nonce account for transaction building.
- **Response**:
```json
{
  "pubkey": "NoncePubkey11111111111111111111111111111111"
}
```

### Release Nonce (POST)
```http
POST /api/v1/nonce/release
```
- **Headers**: `X-Api-Key: <key>`
- **Request Payload**:
```json
{
  "pubkey": "NoncePubkey11111111111111111111111111111111"
}
```
- **Response**: `200 OK`

### Sync Nonce Pool (POST)
```http
POST /api/v1/nonce/sync
```
- **Headers**: `X-Api-Key: <key>`
- **Description**: Refreshes nonce account states and releases stale locks.
- **Response**: `200 OK`

---

## 6. POS Order Creation & Sales

### Create POS Order (POST)
```http
POST /api/v1/pos/create-order
```
- **Headers**: `X-Api-Key: <key>`
- **Request Payload**:
```json
{
  "chat_id": 123456789,
  "raw_text": "2x Cappuccino 200 UAH"
}
```
- **Response**: `200 OK`

### Sales Summary (GET)
```http
GET /api/v1/sales/summary
```
- **Response**:
```json
{
  "total_sales_usdc": 1500.50,
  "completed_invoices_count": 42
}
```

### Premium Analytics (GET) - x402 Machine Commerce Gated
```http
GET /api/v1/sales/premium_analytics
```
- **Headers**: `X-ACCEPT-PAYMENT: <tx_signature>` (Optional on first call, mandatory for challenge resolution)
- **Response**:
  - `402 Payment Required`: If `X-ACCEPT-PAYMENT` header is missing or payment signature is unverified.
  - `200 OK`: Returns detailed analytics payload when valid micropayment transaction signature is provided.

---

## 7. Solana Actions / Blinks (Dialect v2 Spec)

### Actions Discovery Spec (GET)
```http
GET /actions.json
```
- **Headers**: `X-Action-Version: 2.4`, `X-Blockchain-Ids: solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp`
- **Response**:
```json
{
  "rules": [
    {
      "pathPattern": "/api/v1/actions/**",
      "apiPath": "/api/v1/actions/**"
    }
  ]
}
```

### Pay Invoice Action (GET)
```http
GET /api/v1/actions/pay_invoice?amount=10.0&reference=RefKey...
```
- **Headers**: `X-Action-Version: 2.4`, `X-Blockchain-Ids: solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp`
- **Description**: Returns Solana Action metadata for rendering Action Blinks in Twitter/Telegram/Dialect clients.
- **Response**:
```json
{
  "icon": "https://zeroclaw.io/logo.png",
  "title": "ZeroClaw POS Payment",
  "description": "Pay 10.00 USDC for POS Receipt #102",
  "label": "Pay 10.00 USDC",
  "links": {
    "actions": [
      {
        "label": "Pay 10.00 USDC",
        "href": "/api/v1/actions/pay_invoice?amount=10.0"
      }
    ]
  }
}
```

### Pay Invoice Action Execution (POST)
```http
POST /api/v1/actions/pay_invoice?amount=10.0
```
- **Headers**: `X-Action-Version: 2.4`, `X-Blockchain-Ids: solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp`
- **Request Payload**:
```json
{
  "account": "CustomerSolanaWalletPubkey111111111111111111"
}
```
- **Response**:
```json
{
  "transaction": "base64_encoded_serialized_solana_transaction",
  "message": "Payment transaction created successfully"
}
```
