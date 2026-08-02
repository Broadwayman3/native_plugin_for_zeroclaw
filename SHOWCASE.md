# 🏆 ZeroClaw Solana POS Agent - Contest Showcase & Video Script

## 📌 Discord Showcase Post (#solana-bounty)

**Project Name:** ZeroClaw Solana POS Agent (Tier 3 WASM & Squads v4)  
**GitHub Repo:** `[ВСТАВТЕ_ПОСИЛАННЯ_НА_GITHUB_РЕПОЗИТОРІЙ]`  
**Video Demo (2.5 min):** `[ВСТАВТЕ_ПОСИЛАННЯ_НА_ДЕМО_ВІДЕО]`  
**Custody Tier:** Tier 1 Invoicing + Tier 3 WASM Native Sandbox + Squads v4 Multisig Proposals (Tier 2 Guarded)  

### ☕ What it does:
An autonomous AI POS Payment Terminal operating in Telegram/WhatsApp for local businesses.

- **Solana Pay Invoicing**: Instant QR code generation with unique Ed25519 reference keys.
- **Multi-currency Pricing**: Supports USD, UAH, and Brazil BRL via Switchboard Crossbar.
- **Triple Payment Verification**: Rejects fake tokens & 1-lamport dusting attacks.
- **Squads v4 Multisig Governance**: Refunds are created as Squads v4 proposals (Proposer role), requiring store owner threshold approvals.
- **Durable Nonce Pools**: Solves blockhash expiry during delayed human approval checkpoints and eliminates parallel approval collisions (Bounty Trap #1).
- **SQLite POS REST Backend**: Local WAL-mode DB & reporting API (`GET /api/v1/sales/summary`).

### 🛡️ Security & Reproducibility:
- **100% Automated CI Test Pass**: 305 boundary/stress tests (including 10,000 WASM String Heap Stress Test) + 6 prompt injection jailbreak tests + Rust `proptest` suite.
- **Fail-Closed Security**: Invalid LLM JSON, missing wallet configs, or unknown RPC hosts immediately halt payment verification without making arbitrary state mutations.

---

## 📽️ Split-Screen Video Demo Script (2:30 Min)

- **[0:00 - 0:30] Screen 1 (Split-Screen: Terminal + Telegram)**: Cashier types in Telegram: *"Вистави чек на 200 UAH за каву"* (or *"Charge 54.50 BRL"*). Agent instantly returns Solana Pay QR code and link for 4.82 USDC.
- **[0:30 - 1:15] Screen 2 (Devnet Transaction Execution)**: Solflare mobile wallet scans QR code, confirms payment. POS Agent receives transaction, parses Associated Token Account balance deltas, and marks invoice as `PAID`.
- **[1:15 - 1:50] Refund Flow & Squads v4 Proposal**: Cashier requests refund. POS Agent requests Human Manager Approval via Telegram button. Upon approval, POS Agent initiates a keyless Squads v4 Multisig proposal (Proposer role, no withdrawal key access).
- **[1:50 - 2:10] Token-2022 Transfer Fee & Offline Fallback**: Demonstration of Token-2022 Transfer Fee capping and offline static fallback price feed when Pyth/Switchboard endpoints are simulated offline.
- **[2:10 - 2:30] REST API & Test Pass**: Shows `curl http://localhost:8080/api/v1/sales/summary` and execution of `./scripts/build_wasm.sh` and `./scripts/test_boundary_cases.py` (305/305 PASSED).



