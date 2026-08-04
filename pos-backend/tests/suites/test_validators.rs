use pos_backend::domain::validators::*;

#[test]
fn test_311_validate_solana_pay_response_schema() {
    let schema = solana_pay_response_schema();
    assert_eq!(
        schema.get("type").and_then(|v| v.as_str()),
        Some("object"),
        "311: missing type"
    );
    assert!(
        schema.get("required").and_then(|v| v.as_array()).is_some(),
        "311: missing required"
    );
    assert!(
        schema
            .get("properties")
            .and_then(|v| v.as_object())
            .is_some(),
        "311: missing properties"
    );
}

#[test]
fn test_312_validate_squads_proposal_schema() {
    let schema = squads_proposal_schema();
    assert_eq!(
        schema.get("type").and_then(|v| v.as_str()),
        Some("object"),
        "312: missing type"
    );
    assert!(
        schema.get("required").and_then(|v| v.as_array()).is_some(),
        "312: missing required"
    );
    assert!(
        schema
            .get("properties")
            .and_then(|v| v.as_object())
            .is_some(),
        "312: missing properties"
    );
}

#[test]
fn test_313_validate_llm_json_valid() {
    let schema = solana_pay_response_schema();
    let valid = r#"{"status":"confirmed","usdc_amount":1.5,"reference_pubkey":"Abcdefghijklmnopqrstuvwxyz1234567890abcd"}"#;
    validate_llm_json_output(valid, &schema).expect("313: valid JSON should pass");
}

#[test]
fn test_314_validate_llm_json_missing_required() {
    let schema = solana_pay_response_schema();
    let missing = r#"{"status":"confirmed","usdc_amount":1.5}"#;
    assert!(
        validate_llm_json_output(missing, &schema).is_err(),
        "314: missing required should fail"
    );
}

#[test]
fn test_315_validate_llm_json_wrong_type() {
    let schema = solana_pay_response_schema();
    let wrong = r#"{"status":"confirmed","usdc_amount":"not_a_number","reference_pubkey":"Abcdefghijklmnopqrstuvwxyz1234567890abcd"}"#;
    assert!(
        validate_llm_json_output(wrong, &schema).is_err(),
        "315: wrong type should fail"
    );
}

#[test]
fn test_316_validate_llm_json_min_violation() {
    let schema = solana_pay_response_schema();
    let below_min = r#"{"status":"confirmed","usdc_amount":0.001,"reference_pubkey":"Abcdefghijklmnopqrstuvwxyz1234567890abcd"}"#;
    assert!(
        validate_llm_json_output(below_min, &schema).is_err(),
        "316: below min should fail"
    );
}

#[test]
fn test_317_validate_llm_json_max_violation() {
    let schema = solana_pay_response_schema();
    let above_max = r#"{"status":"confirmed","usdc_amount":99999.0,"reference_pubkey":"Abcdefghijklmnopqrstuvwxyz1234567890abcd"}"#;
    assert!(
        validate_llm_json_output(above_max, &schema).is_err(),
        "317: above max should fail"
    );
}

#[test]
fn test_318_validate_llm_json_minlength() {
    let schema = solana_pay_response_schema();
    let short = r#"{"status":"confirmed","usdc_amount":1.0,"reference_pubkey":"short"}"#;
    assert!(
        validate_llm_json_output(short, &schema).is_err(),
        "318: minLength violation should fail"
    );
}

#[test]
fn test_319_validate_llm_json_maxlength() {
    let schema = solana_pay_response_schema();
    let long_key: String = "A".repeat(100);
    let too_long = format!(
        r#"{{"status":"confirmed","usdc_amount":1.0,"reference_pubkey":"{}"}}"#,
        long_key
    );
    assert!(
        validate_llm_json_output(&too_long, &schema).is_err(),
        "319: maxLength violation should fail"
    );
}

#[test]
fn test_320_validate_llm_json_enum_violation() {
    let schema = solana_pay_response_schema();
    let bad_enum = r#"{"status":"unknown","usdc_amount":1.0,"reference_pubkey":"Abcdefghijklmnopqrstuvwxyz1234567890abcd"}"#;
    assert!(
        validate_llm_json_output(bad_enum, &schema).is_err(),
        "320: enum violation should fail"
    );
}

#[test]
fn test_321_validate_llm_json_invalid_json() {
    let schema = solana_pay_response_schema();
    assert!(
        validate_llm_json_output("not json {{{", &schema).is_err(),
        "321: invalid JSON should fail"
    );
}

#[test]
fn test_322_truncate_under_limit() {
    let data = serde_json::json!({"status": "confirmed", "usdc_amount": 1.0});
    let result = truncate_for_context(&data, 1000);
    assert_eq!(result, data, "322: under limit should return unchanged");
}

#[test]
fn test_323_truncate_over_limit() {
    let mut obj = serde_json::Map::new();
    obj.insert("status".to_string(), serde_json::json!("confirmed"));
    obj.insert("usdc_amount".to_string(), serde_json::json!(1.0));
    obj.insert(
        "extra_metadata".to_string(),
        serde_json::json!("x".repeat(500)),
    );
    let data = serde_json::Value::Object(obj);
    let result = truncate_for_context(&data, 10);
    let result_str = serde_json::to_string(&result).unwrap();
    assert!(
        result_str.len() < serde_json::to_string(&data).unwrap().len(),
        "323: over limit should be pruned"
    );
}

#[test]
fn test_324_truncate_multibyte_utf8() {
    let mut obj = serde_json::Map::new();
    obj.insert("status".to_string(), serde_json::json!("confirmed"));
    obj.insert(
        "reference_pubkey".to_string(),
        serde_json::json!("日本語テスト文字列で很长的字符串用于测试截断功能安全性"),
    );
    let data = serde_json::Value::Object(obj);
    let result = truncate_for_context(&data, 10);
    if let Some(val) = result.get("reference_pubkey") {
        if let Some(s) = val.as_str() {
            assert!(
                s.starts_with("日本語テスト文字列で很长的字符串用于测试截断功能安全性")
                    || s.len() <= 50,
                "324: multi-byte UTF-8 truncated unsafely"
            );
            return;
        }
    }
}

#[test]
fn test_325_truncate_empty() {
    let data = serde_json::json!({});
    let result = truncate_for_context(&data, 1000);
    assert!(
        result.is_object() && result.as_object().unwrap().is_empty(),
        "325: empty should return empty object"
    );
}
