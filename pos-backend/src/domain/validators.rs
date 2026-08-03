use std::collections::HashMap;

/// JSON Schema for Solana Pay response validation.
pub fn solana_pay_response_schema() -> HashMap<String, serde_json::Value> {
    let mut schema = HashMap::new();
    schema.insert("type".to_string(), serde_json::json!("object"));
    schema.insert(
        "required".to_string(),
        serde_json::json!(["status", "usdc_amount", "reference_pubkey"]),
    );
    schema.insert(
        "properties".to_string(),
        serde_json::json!({
            "status": {"type": "string", "enum": ["pending", "confirmed", "failed"]},
            "usdc_amount": {"type": "number", "minimum": 0.01, "maximum": 5000.0},
            "reference_pubkey": {"type": "string", "minLength": 32, "maxLength": 44}
        }),
    );
    schema
}

/// JSON Schema for Squads proposal validation.
pub fn squads_proposal_schema() -> HashMap<String, serde_json::Value> {
    let mut schema = HashMap::new();
    schema.insert("type".to_string(), serde_json::json!("object"));
    schema.insert(
        "required".to_string(),
        serde_json::json!(["status", "proposal_index", "amount_usdc", "multisig_pubkey"]),
    );
    schema.insert(
        "properties".to_string(),
        serde_json::json!({
            "status": {"type": "string", "enum": ["created", "rejected", "approved"]},
            "proposal_index": {"type": "integer", "minimum": 1},
            "amount_usdc": {"type": "number", "minimum": 0.01, "maximum": 50.0},
            "multisig_pubkey": {"type": "string", "minLength": 32, "maxLength": 44}
        }),
    );
    schema
}

/// Validates raw JSON output against a strict schema.
/// Raises error on schema violation for fail-closed behavior.
pub fn validate_llm_json_output(
    raw_output: &str,
    schema: &HashMap<String, serde_json::Value>,
) -> Result<serde_json::Value, String> {
    let data: serde_json::Value =
        serde_json::from_str(raw_output).map_err(|e| format!("Invalid JSON: {}", e))?;

    // Check required fields
    if let Some(required) = schema.get("required").and_then(|v| v.as_array()) {
        for field in required {
            if let Some(field_name) = field.as_str() {
                if data.get(field_name).is_none() {
                    return Err(format!("Missing required field: {}", field_name));
                }
            }
        }
    }

    // Validate property types and constraints
    if let Some(properties) = schema.get("properties").and_then(|v| v.as_object()) {
        for (key, prop_schema) in properties {
            if let Some(value) = data.get(key) {
                if let Some(prop_type) = prop_schema.get("type").and_then(|v| v.as_str()) {
                    match prop_type {
                        "number" | "integer" => {
                            if !value.is_number() {
                                return Err(format!("Invalid type for {}: expected number", key));
                            }
                            if let Some(min) = prop_schema.get("minimum").and_then(|v| v.as_f64()) {
                                if value.as_f64().unwrap_or(0.0) < min {
                                    return Err(format!("{} must be >= {}", key, min));
                                }
                            }
                            if let Some(max) = prop_schema.get("maximum").and_then(|v| v.as_f64()) {
                                if value.as_f64().unwrap_or(0.0) > max {
                                    return Err(format!("{} must be <= {}", key, max));
                                }
                            }
                        }
                        "string" => {
                            if !value.is_string() {
                                return Err(format!("Invalid type for {}: expected string", key));
                            }
                            let s = value.as_str().unwrap_or("");
                            if let Some(min_len) = prop_schema.get("minLength").and_then(|v| v.as_u64()) {
                                if s.len() < min_len as usize {
                                    return Err(format!("{} length must be >= {}", key, min_len));
                                }
                            }
                            if let Some(max_len) = prop_schema.get("maxLength").and_then(|v| v.as_u64()) {
                                if s.len() > max_len as usize {
                                    return Err(format!("{} length must be <= {}", key, max_len));
                                }
                            }
                            if let Some(enum_values) = prop_schema.get("enum").and_then(|v| v.as_array()) {
                                let valid: Vec<&str> = enum_values.iter().filter_map(|v| v.as_str()).collect();
                                if !valid.contains(&s) {
                                    return Err(format!("Invalid enum value for {}: {}", key, s));
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(data)
}

/// Trims non-essential metadata fields to keep tokens under ~150.
pub fn truncate_for_context(data: &serde_json::Value, max_tokens: usize) -> serde_json::Value {
    let json_str = serde_json::to_string(data).unwrap_or_default();
    let max_chars = max_tokens * 4;

    if json_str.len() <= max_chars {
        return data.clone();
    }

    let essential_keys = [
        "status", "verified", "usdc_amount", "paid_amount",
        "reference_pubkey", "signature", "proposal_index",
    ];

    let mut pruned = serde_json::Map::new();
    if let Some(obj) = data.as_object() {
        for key in &essential_keys {
            if let Some(value) = obj.get(*key) {
                if let Some(s) = value.as_str() {
                    if s.len() > 44 {
                        pruned.insert(
                            key.to_string(),
                            serde_json::Value::String(format!("{}...", &s[..41])),
                        );
                    } else {
                        pruned.insert(key.to_string(), value.clone());
                    }
                } else {
                    pruned.insert(key.to_string(), value.clone());
                }
            }
        }
    }

    serde_json::Value::Object(pruned)
}
