#!/usr/bin/env python3
"""ZeroClaw Solana POS Agent - Solana Transaction Verification Module"""

import sys
from typing import Dict, List, Optional, Any
from pos_core.constants import USDC_MINT


def _extract_token_balance_deltas(meta: Dict[str, Any], expected_mint: str, debug_log: bool = False) -> Dict[int, int]:
    """Extracts balance differences (post - pre) for specified mint indexed by account index."""
    pre_balances: Dict[int, int] = {}
    post_balances: Dict[int, int] = {}

    for b in meta.get("preTokenBalances") or []:
        if isinstance(b, dict) and b.get("mint") == expected_mint:
            try:
                ui_amt = b.get("uiTokenAmount") or {}
                amt_val = int((ui_amt.get("amount") if isinstance(ui_amt, dict) else None) or "0")
                idx_val = b.get("accountIndex")
                idx = int(idx_val) if idx_val is not None else 0
                pre_balances[idx] = amt_val
            except (ValueError, TypeError, AttributeError) as e:
                if debug_log:
                    print(f"[DEBUG][verification] Suppressed preTokenBalance parsing error: {e}", file=sys.stderr)

    for b in meta.get("postTokenBalances") or []:
        if isinstance(b, dict) and b.get("mint") == expected_mint:
            try:
                ui_amt = b.get("uiTokenAmount") or {}
                amt_val = int((ui_amt.get("amount") if isinstance(ui_amt, dict) else None) or "0")
                idx_val = b.get("accountIndex")
                idx = int(idx_val) if idx_val is not None else 0
                post_balances[idx] = amt_val
            except (ValueError, TypeError, AttributeError) as e:
                if debug_log:
                    print(f"[DEBUG][verification] Suppressed postTokenBalance parsing error: {e}", file=sys.stderr)

    deltas: Dict[int, int] = {}
    all_indices = set(pre_balances.keys()) | set(post_balances.keys())
    for idx in all_indices:
        deltas[idx] = post_balances.get(idx, 0) - pre_balances.get(idx, 0)
    return deltas


def _inspect_instructions_for_transfer(
    instructions: Optional[List[Any]], expected_merchant_ata: str, expected_usdc_atomic: int, debug_log: bool = False
) -> Optional[int]:
    """Recursively inspects instructions for token transfers matching Merchant ATA and expected amount."""
    for inst in instructions or []:
        if not isinstance(inst, dict):
            continue
        parsed = inst.get("parsed")
        if isinstance(parsed, dict) and parsed.get("type") in ["transfer", "transferChecked"]:
            info = parsed.get("info") or {}
            if isinstance(info, dict) and info.get("destination") == expected_merchant_ata:
                token_amt = info.get("tokenAmount") or {}
                amt_str = info.get("amount") or (token_amt.get("amount") if isinstance(token_amt, dict) else None)
                if amt_str:
                    try:
                        amt_val = int(amt_str)
                        if amt_val >= expected_usdc_atomic:
                            return amt_val
                    except (ValueError, TypeError, AttributeError) as e:
                        if debug_log:
                            print(f"[DEBUG][verification] Suppressed transfer amount parsing error: {e}", file=sys.stderr)
        nested_ixs = inst.get("instructions")
        if isinstance(nested_ixs, list):
            res = _inspect_instructions_for_transfer(nested_ixs, expected_merchant_ata, expected_usdc_atomic, debug_log=debug_log)
            if res is not None:
                return res
    return None


def verify_solana_transaction_payload(
    tx_json: Any, expected_merchant_ata: str, expected_usdc_atomic: int, expected_mint: str = USDC_MINT, debug_log: bool = False
) -> Dict[str, Any]:
    """Triple Payment Protection: Reverted Tx Guard, Balance Delta Verification, Recursive Instruction Inspection."""
    if not tx_json or not isinstance(tx_json, dict):
        return {"is_valid": False, "error": "Invalid transaction JSON payload"}
    meta = tx_json.get("meta")
    if not meta or not isinstance(meta, dict) or meta.get("err") is not None:
        return {"is_valid": False, "error": "Transaction failed or reverted on-chain (meta.err != null)"}

    deltas = _extract_token_balance_deltas(meta, expected_mint, debug_log=debug_log)
    transaction = tx_json.get("transaction") or {}
    message = (transaction.get("message") if isinstance(transaction, dict) else {}) or {}
    account_keys = (message.get("accountKeys") or message.get("staticAccountKeys") if isinstance(message, dict) else []) or []

    merchant_idx = next((i for i, k in enumerate(account_keys) if (k.get("pubkey") if isinstance(k, dict) else k) == expected_merchant_ata), None)

    if merchant_idx is not None:
        delta = deltas.get(merchant_idx, 0)
        if delta >= expected_usdc_atomic:
            return {"is_valid": True, "paid_atomic": delta, "verification_method": "balance_delta"}

    top_ixs = (message.get("instructions") if isinstance(message, dict) else []) or []
    paid_top = _inspect_instructions_for_transfer(top_ixs, expected_merchant_ata, expected_usdc_atomic, debug_log=debug_log)
    if paid_top is not None:
        return {"is_valid": True, "paid_atomic": paid_top, "verification_method": "top_level_instruction"}

    inner_ixs = meta.get("innerInstructions") or []
    for group in (inner_ixs if isinstance(inner_ixs, list) else []):
        if isinstance(group, dict):
            paid_inner = _inspect_instructions_for_transfer(group.get("instructions"), expected_merchant_ata, expected_usdc_atomic, debug_log=debug_log)
            if paid_inner is not None:
                return {"is_valid": True, "paid_atomic": paid_inner, "verification_method": "inner_instruction"}

    return {"is_valid": False, "error": "No valid token transfer or positive balance delta found for Merchant ATA"}
