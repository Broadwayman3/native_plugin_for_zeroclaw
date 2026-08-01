#!/usr/bin/env python3
"""
ZeroClaw Solana POS Agent - Fail-Closed JSON Schema Validator & LLM Context Truncator
Enforces structural schema constraints on LLM and RPC outputs.
Prevents context window flooding by trimming payloads to ~150 tokens.
"""

import json
import jsonschema

SOLANA_PAY_RESPONSE_SCHEMA = {
    "type": "object",
    "properties": {
        "status": {"type": "string", "enum": ["pending", "confirmed", "failed"]},
        "usdc_amount": {"type": "number", "minimum": 0.01, "maximum": 5000.0},
        "reference_pubkey": {"type": "string", "minLength": 32, "maxLength": 44}
    },
    "required": ["status", "usdc_amount", "reference_pubkey"]
}

SQUADS_PROPOSAL_SCHEMA = {
    "type": "object",
    "properties": {
        "status": {"type": "string", "enum": ["created", "rejected", "approved"]},
        "proposal_index": {"type": "integer", "minimum": 1},
        "amount_usdc": {"type": "number", "minimum": 0.01, "maximum": 50.0},
        "multisig_pubkey": {"type": "string", "minLength": 32, "maxLength": 44}
    },
    "required": ["status", "proposal_index", "amount_usdc", "multisig_pubkey"]
}

PAYMENT_VERIFICATION_SCHEMA = {
    "type": "object",
    "properties": {
        "verified": {"type": "boolean"},
        "signature": {"type": "string", "minLength": 32, "maxLength": 90},
        "paid_amount": {"type": "number", "minimum": 0.0}
    },
    "required": ["verified", "signature", "paid_amount"]
}

def validate_llm_json_output(raw_output: str, schema: dict = SOLANA_PAY_RESPONSE_SCHEMA) -> dict:
    """
    Evaluates raw LLM or API string against a strict JSON Schema.
    Raises ValueError on schema violation for Fail-Closed halting.
    """
    try:
        data = json.loads(raw_output)
        jsonschema.validate(instance=data, schema=schema)
        return data
    except (json.JSONDecodeError, jsonschema.ValidationError) as e:
        raise ValueError(f"🚨 FAIL-CLOSED: Output violated structural schema constraints: {e}")

def truncate_for_context(data_dict: dict, max_tokens: int = 150) -> dict:
    """
    Trims non-essential metadata fields from dictionaries to keep tokens < max_tokens (~200 limit).
    Bounty Trap Guard #3: Prevents context window flooding in Agent Runtime.
    """
    json_str = json.dumps(data_dict)
    max_chars = max_tokens * 4
    
    if len(json_str) <= max_chars:
        return data_dict
    
    essential_keys = {"status", "verified", "usdc_amount", "paid_amount", "reference_pubkey", "signature", "proposal_index"}
    pruned = {k: v for k, v in data_dict.items() if k in essential_keys}
    
    for k, v in pruned.items():
        if isinstance(v, str) and len(v) > 44:
            pruned[k] = v[:41] + "..."
            
    return pruned

if __name__ == "__main__":
    sample_valid = '{"status": "confirmed", "usdc_amount": 10.5, "reference_pubkey": "8xAZmQ1111111111111111111111111111111111111"}'
    res = validate_llm_json_output(sample_valid)
    assert res["status"] == "confirmed"
    print("✅ validators.py self-test passed successfully.")
