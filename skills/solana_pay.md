---
name: solana_pay
description: Skill for generating Solana Pay URLs, QR codes, and Blinks with unique reference public keys and Triple Payment Verification.
---

# Solana Pay Invoicing & QR Code Skill

This skill allows the agent to parse customer/cashier invoice requests, generate valid non-custodial Solana Pay URLs, and enforce **Triple Payment Verification**.

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

3. **Triple Payment Verification Engine (Prevents Forgery & Dusting)**:
   - **Condition 1 (Reference Matching)**: Transaction must include the invoice's unique `reference` Ed25519 public key.
   - **Condition 2 (Token Mint Enforcement)**: Token transfer mint must EXACTLY match USDC Mint (`EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v`). Rejects fake/custom SPL tokens.
   - **Condition 3 (Amount Sufficiency)**: `paid_amount_atomic_units` >= `expected_amount_atomic_units`. Rejects 1-lamport micro-dusting attacks!

4. **Output Formatting for LLM (Token-Optimized)**:
   - Always produce a concise response (<200 tokens).
   - Provide the direct Solana Pay deep link.
   - Include a clickable QR code image link via standard QR rendering.
