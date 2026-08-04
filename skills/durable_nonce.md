---
name: durable_nonce
description: Skill for constructing Durable Nonce transactions with live RPC state querying and Nonce Account Pools to solve Blockhash Expiry and Parallel Approval Collisions (Bounty Trap #1). Nonce pool managed in Rust (pos-backend/src/db/nonce.rs).
---

# Durable Nonce Account Skill (Solana Craft Solution)

## Problem Solved: Blockhash Expiry & Parallel Approval Collisions (Bounty Trap #1)

1. **Blockhash Expiry**: Standard Solana blockhashes expire in ~90 seconds (~150 blocks). When a refund pauses at a **Human Approval Checkpoint**, standard transactions fail with `BlockhashNotFound`.
2. **Parallel Nonce Collisions (Bounty Trap #1)**: One nonce account serializes to ONE in-flight transaction! If multiple customers request refunds simultaneously and approvals are delayed, using the same Nonce Account causes `AdvanceNonceAccount` collisions, invalidating all other pending transactions.

## Solution: Live RPC Querying & Nonce Account Pool Allocation

### Nonce Account Pool Configuration
The agent maintains a pool of initialized Nonce Accounts (`nonce_accounts = ["Nonce111...", "Nonce222...", "Nonce333..."]`). When building parallel proposals:
- Dynamically allocates a distinct, free Nonce Account from the pool for each active invoice/refund.
- Immediately prior to transaction construction, queries live on-chain account state via `getAccountInfo(NONCE_ACCOUNT_PUBKEY)`.

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
   - **CRITICAL STEP**: Agent allocates free nonce from pool and queries `getAccountInfo(NONCE_ACCOUNT_PUBKEY)` with `commitment: "confirmed"`.
   - Parses fresh `nonce_hash` from live on-chain account data.
   - Builds transaction:
     - `Instruction #0`: `SystemProgram.advanceNonceAccount({ noncePubkey, authorizedPubkey })`
     - `Instruction #1`: `TokenProgram.transfer({ source: REFUND_SESSION_WALLET, destination: CLIENT_WALLET, amount: 5_000_000 })`
   - Sets `recentBlockhash = live_nonce_hash`.

2. **Step 2 (Human Checkpoint Pause)**:
   - Agent sends Telegram message to Manager ID with Nonce Pool details.

3. **Step 3 (Execution after Manager Approval)**:
   - Manager approves proposal.
   - ZeroClaw submits transaction cleanly without blockhash expiry or parallel nonce collisions.

### RPC Inspection Payload (<150 tokens)
```json
{
  "durable_nonce_active": true,
  "nonce_pool_size": 3,
  "allocated_nonce_account": "Nonce111111111111111111111111111111111111111",
  "stored_nonce": "4uQeVj5t...9xKb",
  "live_rpc_verified": true,
  "status": "valid_indefinitely_until_advanced"
}
```
