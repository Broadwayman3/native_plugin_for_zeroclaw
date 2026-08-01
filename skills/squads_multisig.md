---
name: squads_multisig
description: Skill for constructing Squads v4 Multisig Proposal transactions with live on-chain sequence index synchronization.
---

# Squads v4 Multisig Integration Skill (Tier 3 Institutional Security)

## Overview & Architecture

To eliminate single-point-of-failure key risks during refund processing, the agent integrates with **Squads v4** (Solana's premier smart contract multisig protocol).

Instead of signing and executing transfers directly from a hot wallet, the agent acts in a restricted **`Proposer`** role. The store owner/management team holds the **`Vault Authority / Threshold Signer`** keys.

```
+------------------+         +-----------------------+         +-------------------------+
| ZeroClaw Agent   | ------> | Squads v4 Program     | ------> | Store Manager (Phantom) |
| Role: Proposer   |         | Proposal #42 Created  |         | Role: Threshold Signer  |
+------------------+         +-----------------------+         +-------------------------+
                                                                             |
                                                                             v
                                                                 [Approve & Execute Tx]
```

## Squads v4 Parameters & Non-Custodial Key Roles

- **Program ID**: `SQDS4ep65T869rmQrGGsybZb26a6Uq3vig54W62pkhm`
- **Multisig Account**: `${SQUADS_MULTISIG_PUBKEY}`
- **Vault Account**: `${SQUADS_VAULT_PUBKEY}`
- **REFUND_SESSION_KEY Role**: `Payer & Proposer` (schedules proposal on-chain & pays ~0.002 SOL rent, but holds **zero Execution Authority**).
- **Store Manager Role**: `Threshold Signer & Execution Authority` (holds final authorization keys).

## Blind Spot #4 Countermeasure: Live On-Chain Proposal Index Synchronization

If proposals are created manually or via SquadsX app outside of ZeroClaw, local database counters desynchronize. 

**Rule**: The agent MUST perform a live RPC `getAccountInfo(SQUADS_MULTISIG_PUBKEY)` query to parse the `transaction_index` state directly from the on-chain account payload before generating a new proposal:
```
next_proposal_index = onchain_multisig_account.transaction_index + 1
```

## Workflow for Refund Proposals

1. **Refund Request Received**:
   - Cashier/Customer requests refund: *"Refund 15 USDC for invoice #104"*.

2. **On-Chain Sequence Sync & WASM Proposal Creation**:
   - Agent queries `getAccountInfo(SQUADS_MULTISIG_PUBKEY)` with `commitment: "confirmed"`.
   - Fetches live `transaction_index = 41`. Computes `next_proposal_index = 42`.
   - Calls `plugins/solana-pos-core` WASM crate to generate a `multisig_create_proposal` instruction payload.

3. **Human Approval Checkpoint Notification**:
   - ZeroClaw triggers a Telegram notification to the store manager:
     > 🏛️ **Squads v4 Multisig Proposal Created**
     > • Proposal Index: `#42` (On-Chain Verified)
     > • Multisig Vault: `${SQUADS_VAULT_PUBKEY}`
     > • Action: Transfer **15.00 USDC** -> `9xK2...mQ11`
     > 
     > [Схвалити у Squads App / Phantom]

4. **Execution**:
   - Manager reviews and signs the proposal via Phantom / Squads App. Once threshold is met, Squads v4 executes the transfer automatically on-chain.
