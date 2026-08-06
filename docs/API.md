# REST API & Webhook Specification

Base URL: `http://localhost:8080`

Total Endpoints: **13 REST API routes**

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
POST /api/v1/settings/update
```
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

## 2. Telegram Webhook

### Receive Webhook Update (POST)
```http
POST /api/v1/telegram/webhook
```
- **Headers**: `X-Telegram-Bot-Api-Secret-Token: <token>` (validated via constant-time string comparison)
- **Body Limit**: 128 KB maximum payload limit
- **Description**: Enqueues incoming Telegram update synchronously into SQLite WAL database (`webhook.rs`) with a `4500ms` `deadpool` connection acquire timeout and triggers async worker wakeup.
- **Response**:
  - `200 OK`: Update successfully stored in SQLite queue.
  - `500 Internal Server Error`: Connection pool acquisition timed out (>4.5s) or DB write failed. Returns 500 to signal Telegram gateway to retry delivery.

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
  "status": "pending",
  "solana_pay_url": "solana:8xAZnR2pMQR3Qv5xK8c7mQ11rF4eG7hJ9kL2nP4s?amount=10.00&spl-token=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v&reference=RefKey1111111111111111111111111111111111111"
}
```

### Update Invoice Status (POST)
```http
POST /api/v1/invoices/update_status
```
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

---

## 5. Nonce Pool

### Allocate Nonce (POST)
```http
POST /api/v1/nonce/allocate
```
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
- **Request Payload**:
```json
{
  "pubkey": "NoncePubkey11111111111111111111111111111111"
}
```

### Sync Nonce Pool (POST)
```http
POST /api/v1/nonce/sync
```
- **Description**: Refreshes nonce account states and releases stale locks.

---

## 6. POS Order Creation & Sales

### Create POS Order (POST)
```http
POST /api/v1/pos/create-order
```
- **Request Payload**:
```json
{
  "chat_id": 123456789,
  "raw_text": "2x Cappuccino 200 UAH"
}
```

### Sales Summary (GET)
```http
GET /api/v1/sales/summary
```

### Premium Analytics (GET) - x402 Gated
```http
GET /api/v1/sales/premium_analytics
```
- **Headers**: `X-ACCEPT-PAYMENT: <tx_signature>`
- **Response**: `402 Payment Required` if header missing, or `200 OK` with analytics payload.

---

## 7. Solana Actions / Blinks

### Actions Discovery Spec (GET)
```http
GET /actions.json
```

### Pay Invoice Action (GET & POST)
```http
GET /api/v1/actions/pay_invoice
POST /api/v1/actions/pay_invoice
```
