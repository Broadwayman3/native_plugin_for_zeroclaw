# REST API & Webhook Reference

Base URL: `http://localhost:8080`

## Endpoints

### Telegram Webhook

#### Receive Update (POST)

```
POST /api/v1/telegram/webhook
```

Enqueues incoming Telegram updates into SQLite FIFO processing queue.
- **Header**: `X-Telegram-Bot-Api-Secret-Token` (validated via constant-time comparison `constant_time_eq`)
- **Body Limit**: 64 KB maximum request payload (requests exceeding 64 KB return HTTP 400)
- **Response**: Always returns `HTTP 200 OK` for valid secret tokens to prevent Telegram API disabling webhooks.

---

### Actions / Blinks

#### Get Actions Spec

```
GET /actions.json
```

Returns Solana Actions/Blinks discovery spec.

#### Pay Invoice (GET)

```
GET /api/v1/actions/pay_invoice
```

Returns Blink action card (invoice details).

#### Pay Invoice (POST)

```
POST /api/v1/actions/pay_invoice
```

Processes Blink action payment transaction.

---

### Sales

#### Sales Summary

```
GET /api/v1/sales/summary
```

Returns aggregated sales metrics with daily revenue.

#### Premium Analytics (x402)

```
GET /api/v1/sales/premium_analytics
```

Payment-gated premium analytics (x402 Machine Commerce). Returns HTTP 402 if payment not provided.

---

### Invoices

#### List Invoices

```
GET /api/v1/invoices?id=<invoice_id>&status=<status>
```

Filter by `id` or `status` query parameters.

#### Create Invoice

```
POST /api/v1/invoices/create
```

Creates a new pending invoice. Returns invoice details with Solana Pay URL.

#### Update Invoice Status

```
POST /api/v1/invoices/update_status
```

Atomically updates invoice status. Uses `UPDATE ... WHERE status = 'pending'` to prevent race conditions.

#### Cancel Invoice

```
POST /api/v1/invoices/cancel
```

Cancels/voids a pending invoice.

---

### Nonce Pool

#### Allocate Nonce

```
POST /api/v1/nonce/allocate
```

Allocates a free durable nonce account from the pool. Returns HTTP 503 if pool exhausted.

#### Release Nonce

```
POST /api/v1/nonce/release
```

Releases a locked nonce account back to the pool.

---

### POS Flow

#### Create Order

```
POST /api/v1/pos/create-order
```

Creates an order from parsed POS text input (replaces Telegram text message handler).

---

## Middleware

- **CORS**: Origin `Any`, methods GET/POST/PUT/DELETE/OPTIONS
- **Rate Limiting**: Sliding window rate limiter (`RateLimiter`), returns HTTP 429 on burst limit violations
- **Payload Limit**: 1MB maximum request body (64 KB for Telegram Webhook)
- **Headers**: Content-Type, Authorization, X-ACCEPT-PAYMENT, X-Telegram-Bot-Api-Secret-Token, Content-Encoding, Accept-Encoding

## Error Responses

| Code | Meaning |
|------|---------|
| 200 | Success |
| 400 | Bad Request (invalid input or payload size violation) |
| 401 | Unauthorized (invalid Telegram webhook secret token) |
| 402 | Payment Required (x402) |
| 404 | Not Found |
| 409 | Conflict (invoice already exists) |
| 429 | Too Many Requests (rate limit exceeded) |
| 500 | Internal Server Error |
| 503 | Service Unavailable (nonce pool exhausted) |
