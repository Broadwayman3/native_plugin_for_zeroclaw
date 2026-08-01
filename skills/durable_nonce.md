---
name: durable_nonce
description: Skill for constructing Durable Nonce transactions with live RPC state querying to solve Blockhash Expiry and Nonce Advance Desynchronization.
---

# Durable Nonce Account Skill (Solana Craft Solution)

## Problem Solved: Blockhash Expiry & Async Nonce Advance Desynchronization

1. **Blockhash Expiry**: Standard Solana blockhashes expire in ~90 seconds (~150 blocks). When a refund pauses at a **Human Approval Checkpoint**, standard transactions fail with `BlockhashNotFound`.
2. **Async Nonce Advance Risk (Blind Spot #3)**: If a transaction containing `AdvanceNonceAccount` is sent to the network but fails on a subsequent instruction, the Nonce Account **still advances on-chain**. Storing or caching Nonce values locally causes desynchronization and transaction failures!

## Solution: Live RPC Nonce State Querying

The agent MUST query live on-chain account state via `getAccountInfo(NONCE_ACCOUNT_PUBKEY)` **immediately prior** to building every single transaction proposal. Never cache nonce hashes locally!

### On-Chain Account Requirements:
- **Account Type**: SystemProgram Nonce Account (80 bytes data).
- **Rent Exemption**: ~0.0014472 SOL (1,447,200 lamports rent-exempt).
- **Authority**: Controlled by `REFUND_SESSION_KEY`.

### Uninitialized Nonce Account Edge Case Handling:

If `getAccountInfo(NONCE_ACCOUNT_PUBKEY)` returns an uninitialized account (space == 0):

1. **Auto-Initialization Instructions**:
   - Instruction #0: `SystemProgram.createAccount({ fromPubkey: REFUND_SESSION_KEY, newAccountPubkey: NONCE_ACCOUNT_PUBKEY, lamports: 1447200, space: 80, programId: SYSTEM_PROGRAM_ID })`
   - Instruction #1: `SystemProgram.initializeNonceAccount({ noncePubkey: NONCE_ACCOUNT_PUBKEY, authorizedPubkey: REFUND_SESSION_KEY })`

### Execution Workflow for Refunds:

1. **Step 1 (Live RPC Query & Pre-build)**:
   - Client requests refund: *"Refund 5 USDC for invoice #102"*.
   - **CRITICAL STEP**: Agent performs live RPC call `getAccountInfo(NONCE_ACCOUNT_PUBKEY)` with `commitment: "confirmed"`.
   - Parses fresh `nonce_hash` from live on-chain account data.
   - Builds transaction:
     - `Instruction #0`: `SystemProgram.advanceNonceAccount({ noncePubkey, authorizedPubkey })`
     - `Instruction #1`: `TokenProgram.transfer({ source: REFUND_SESSION_WALLET, destination: CLIENT_WALLET, amount: 5_000_000 })`
   - Sets `recentBlockhash = live_nonce_hash`.

2. **Step 2 (Human Checkpoint Pause)**:
   - Agent sends Telegram message to Manager ID:
     > ⚠️ **Запит на повернення коштів**
     > • Сума: 5.00 USDC
     > • Клієнт: `9xK2...mQ11`
     > • Nonce status: Prepared (Durable Live)
     > 
     > [Схвалити] / [Відхилити]

3. **Step 3 (Execution after Manager Approval)**:
   - Manager replies "Yes" 15 minutes later.
   - ZeroClaw resumes execution, signs transaction with `REFUND_SESSION_KEY`, and submits via `sendTransaction`.
   - Transaction succeeds cleanly despite the 15-minute delay!

### RPC Inspection Payload (<150 tokens)
```json
{
  "durable_nonce_active": true,
  "nonce_account": "Nonce111111111111111111111111111111111111111",
  "stored_nonce": "4uQeVj5t...9xKb",
  "live_rpc_verified": true,
  "status": "valid_indefinitely_until_advanced"
}
```
