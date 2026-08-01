---
name: solana_pay
description: Skill for generating Solana Pay URLs, QR codes, and Blinks with unique reference public keys for POS invoicing.
---

# Solana Pay Invoicing & QR Code Skill

This skill allows the agent to parse customer/cashier invoice requests and generate valid, non-custodial Solana Pay URLs and QR code representations.

## Key Rules & Workflow

1. **Unique Reference Pubkey Generation**:
   - For EVERY new invoice request, generate a fresh random 32-byte Ed25519 Solana Public Key to serve as the `reference` parameter.
   - This `reference` key is registered in active invoices state and passed to the SOP Cron monitoring task (`sops/check_payments.json`).

2. **Solana Pay Spec Format**:
   - Protocol: `solana:`
   - Recipient: Merchant Wallet Pubkey (`${MERCHANT_WALLET_PUBKEY}`)
   - Amount: Calculated USDC amount (e.g. `5.00`)
   - SPL Token: USDC Mint (`EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v`)
   - Reference: Newly generated unique `reference` Pubkey
   - Label: Business Name (e.g. `ZeroClaw Coffee POS`)
   - Message: Invoice ID / Item Description (e.g. `Invoice #102: 2x Cappuccino`)

```
solana:<MERCHANT_PUBKEY>?amount=<AMOUNT>&spl-token=<USDC_MINT>&reference=<UNIQUE_REFERENCE_PUBKEY>&label=<LABEL>&message=<MESSAGE>
```

3. **Output Formatting for LLM (Token-Optimized)**:
   - Always produce a concise response (<200 tokens).
   - Provide the direct Solana Pay deep link.
   - Include a clickable QR code image link via standard QR rendering.
   - Output example:
     ```
     🧾 **Рахунок #102 сформовано**
     • Сума: **5.00 USDC** (≈ 207.50 UAH)
     • Reference Key: `7xWz...9qKP`
     
     📱 **Оплатіть через Phantom / Solflare:**
     [Сканувати QR Код](https://api.qrserver.com/v1/create-qr-code/?size=250x250&data=solana:8xAZ...%3Famount=5.00...)
     
     *Очікую підтвердження з мережі Solana...*
     ```
