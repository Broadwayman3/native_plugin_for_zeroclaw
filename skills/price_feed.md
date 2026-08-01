---
name: price_feed
description: Skill for converting local fiat currencies (UAH, EUR, USD, BRL) to USDC using Jupiter REST API and Switchboard Crossbar endpoints.
---

# Currency Conversion & Price Feed Skill (Multinational & BRL/PIX Ready)

This skill enables the agent to dynamically calculate USDC amounts from local currency prices (UAH, USD, EUR, BRL) specified by cashiers or customers.

## Hard System Constraints (LLM Non-Determinism Defense)

> **CRITICAL INSTRUCTION FOR ALL LLMs (Claude, GPT-4o, Llama 3)**:
> 1. Output strictly valid JSON or the exact markdown template specified.
> 2. Do NOT add conversational preambles, introductory text, or concluding notes.
> 3. Never alter numeric precision or currency symbol definitions.

## Supported Endpoints & Fallbacks

1. **Primary Feed - Jupiter Price API**:
   - Endpoint: `https://api.jup.ag/price/v2?ids=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v`
   - Returns real-time USDC price relative to USD/SOL.

2. **Secondary / Fiat Feed - Switchboard Crossbar API**:
   - Endpoints:
     - `https://crossbar.switchboard.xyz/fiat/UAH_USD` (UAH -> USD)
     - `https://crossbar.switchboard.xyz/fiat/BRL_USD` (BRL -> USD for Brazil flows)

3. **Brazil Local PIX QR Code Fallback**:
   - Generates PIX reconciliation payload alongside Solana Pay URL for Brazilian merchants (`br.gov.bcb.pix`).

4. **Fallback Static Protection**:
   - If REST APIs are unreachable, use cached exchange rates with a warning log to prevent downtime during network blips.

## Usage Example (BRL Flow)

When a user inputs:
> *"Charge table 2, 54.50 BRL"*

The agent calls `price_feed`:
1. Fetches BRL/USD rate via Switchboard -> `1 USD = 5.45 BRL`.
2. Calculates: `54.50 BRL / 5.45 = 10.00 USDC`.
3. Passes `amount=10.00` to `skills/solana_pay.md` and generates optional PIX payload.

## Response Format for LLM Context (<100 tokens)
```json
{
  "fiat_currency": "BRL",
  "fiat_amount": 54.50,
  "usdc_amount": 10.00,
  "rate": 5.45,
  "provider": "Switchboard/Crossbar",
  "pix_reconciliation_active": true
}
```
