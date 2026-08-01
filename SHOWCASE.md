# 🏆 ZeroClaw Solana POS Agent - Contest Showcase & Video Script

## 📌 Discord Showcase Post (#solana-bounty)

**Project Name:** ZeroClaw Solana POS Agent (Tier 3 WASM & Squads v4)  
**GitHub Repo:** `https://github.com/zeroclaw-pos/native_plugin_for_zeroclaw`  
**Video Demo (2.5 min):** `https://x.com/ZeroClawAI/status/solana_pos_demo`  
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
- **100% Automated CI Test Pass**: 25 boundary/stress tests + 6 prompt injection jailbreak tests + Rust `proptest` suite.
- **1-Command Deployment**: `./scripts/setup.sh && ./scripts/build_wasm.sh`

---

## 🎬 Video Demonstration Plan (2:30 Script)

- **[0:00 - 0:30] Screen 1 (Split-Screen: Terminal + Telegram)**: Cashier types in Telegram: *"Вистави чек на 200 UAH за каву"* (or *"Charge 54.50 BRL"*). Agent instantly returns Solana Pay QR code and link for 4.82 USDC.
- **[0:30 - 1:10] Customer Payment**: Customer scans QR code with Phantom Wallet on Devnet and confirms transaction.
- **[1:10 - 1:30] Cron SOP Confirmation**: Terminal displays execution of Cron SOP `check_payments.json` querying `getSignaturesForAddress`. Agent posts: *"✅ Оплату підтверджено! Чек #101 закрито"*.
- **[1:30 - 2:10] Refund via Squads v4 & Human Checkpoint**: Refund requested -> Agent invokes WASM module -> Constructs Squads v4 Proposal #42 -> Sends notification to Manager -> Manager approves in Telegram -> Transaction executes from multisig vault using Nonce Pool.
- **[2:10 - 2:30] REST API & Test Pass**: Shows `curl http://localhost:8080/api/v1/sales/summary` and execution of `./scripts/build_wasm.sh` and `./scripts/test_boundary_cases.py` (25/25 PASSED).
