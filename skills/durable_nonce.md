---
name: durable_nonce
description: Skill for constructing Durable Nonce transactions to solve Blockhash Expiry during Human Approval Checkpoints.
---

# Durable Nonce Account Skill (Solana Craft Solution)

## Problem Solved: Blockhash Expiry in Human Approval Workflows

In standard Solana transactions, blockhashes expire in ~90 seconds (~150 blocks). 
When an agent initiates a **Refund Request**, it pauses execution at a **Human Approval Checkpoint** waiting for the store manager to review and reply "Yes" in Telegram.
If the manager takes 5 minutes to respond, a standard transaction fails with `BlockhashNotFound`.

## Solution: Durable Nonce Accounts

Durable Nonces replace the recent blockhash with a stored nonce value from an on-chain Nonce Account. As long as `AdvanceNonceAccount` is the **first instruction** (Instruction #0) in the transaction, the transaction never expires regardless of how long the human approval takes.

### On-Chain Account Requirements:
- **Account Type**: SystemProgram Nonce Account (80 bytes data).
- **Rent Exemption**: ~0.0014472 SOL (pre-funded and rent-exempt).
- **Authority**: Controlled by `REFUND_SESSION_KEY`.

### Workflow for Refunds:

1. **Step 1 (Agent Pre-build)**:
   - Client requests refund: *"Refund 5 USDC for invoice #102"*.
   - Agent queries the pre-created Nonce Account address (`${NONCE_ACCOUNT_PUBKEY}`).
   - Fetches stored `nonce_hash` from RPC (`getAccountInfo`).
   - Builds transaction:
     - `Instruction #0`: `SystemProgram.advanceNonceAccount({ noncePubkey, authorizedPubkey })`
     - `Instruction #1`: `TokenProgram.transfer({ source: REFUND_SESSION_WALLET, destination: CLIENT_WALLET, amount: 5_000_000 })`
   - Sets `recentBlockhash = nonce_hash`.

2. **Step 2 (Human Checkpoint Pause)**:
   - Agent sends Telegram message to Manager ID:
     > ⚠️ **Запит на повернення коштів**
     > • Сума: 5.00 USDC
     > • Клієнт: `9xK2...mQ11`
     > • Nonce status: Prepared (Durable)
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
  "status": "valid_indefinitely_until_advanced"
}
```
