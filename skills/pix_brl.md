---
name: pix_brl
description: Skill for Brazil-first BRL invoicing and dual settlement reconciliation via Switchboard Crossbar and EMV QRCPS PIX payloads. Implemented in Rust (pos-backend/src/domain/pix_brl.rs).
---

# Brazil-First BRL & PIX Reconciliation Skill

Enables the ZeroClaw POS Agent to process payments in Brazilian Real (BRL), fetch live BRL/USD exchange rates via Switchboard Crossbar API, construct dual Solana Pay USDC links and EMV QRCPS PIX QR payloads with Tag 6304 CRC16 CCITT-FALSE checksums.

## Workflow & Constraints

1. **BRL to USDC Conversion**:
   - Query Switchboard Crossbar API: `https://crossbar.switchboard.xyz/fiat/BRL_USD`.
   - Calculate USDC amount: `amount_usdc = amount_brl / rate_brl_usd`.

2. **Dual QR Response**:
   - Return Solana Pay USDC link.
   - Return EMV QRCPS PIX reconciliation payload (`br.gov.bcb.pix`) with valid CRC16 CCITT-FALSE checksum for instant local fiat fallback.
