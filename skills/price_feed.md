---
name: price_feed
description: Skill for converting local fiat currencies (UAH, EUR, USD, BRL) to USDC using Jupiter REST API and Switchboard Crossbar endpoints.
---

# Currency Conversion & Price Feed Skill

This skill enables the agent to dynamically calculate USDC amounts from local currency prices specified by cashiers or customers.

## Supported Endpoints & Fallbacks

1. **Primary Feed - Jupiter Price API**:
   - Endpoint: `https://api.jup.ag/price/v2?ids=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v`
   - Returns real-time USDC price relative to USD/SOL.

2. **Secondary / Fiat Feed - Switchboard Crossbar / Open Exchange API**:
   - Endpoint: `https://crossbar.switchboard.xyz/fiat/UAH_USD`
   - Returns current exchange rate (e.g. 1 USD = 41.50 UAH, 1 USD = 5.45 BRL).

3. **Fallback Static Protection**:
   - If REST APIs are unreachable, use cached exchange rates with a warning log to prevent downtime during network blips (Trap #4 prevention).

## Usage Example

When a user inputs:
> "Вистави рахунок на 200 UAH за стіл №4"

The agent calls `price_feed`:
1. Fetches UAH/USD rate -> `1 USD = 41.50 UAH`.
2. Calculates: `200 UAH / 41.50 = 4.82 USDC`.
3. Passes `amount=4.82` to `skills/solana_pay.md`.

## Response Format for LLM Context (<100 tokens)
```json
{
  "fiat_currency": "UAH",
  "fiat_amount": 200.0,
  "usdc_amount": 4.82,
  "rate": 41.50,
  "provider": "Jupiter/Switchboard"
}
```
