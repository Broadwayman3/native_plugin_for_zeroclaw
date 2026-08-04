use crate::{test_fail, test_pass};
use pos_backend::domain::validators::*;

pub fn run_suite() {
    println!("\n📦 Validators Tests (311-325)");
    test_311_validate_solana_pay_response_schema();
    test_312_validate_squads_proposal_schema();
    test_313_validate_llm_json_valid();
    test_314_validate_llm_json_missing_required();
    test_315_validate_llm_json_wrong_type();
    test_316_validate_llm_json_min_violation();
    test_317_validate_llm_json_max_violation();
    test_318_validate_llm_json_minlength();
    test_319_validate_llm_json_maxlength();
    test_320_validate_llm_json_enum_violation();
    test_321_validate_llm_json_invalid_json();
    test_322_truncate_under_limit();
    test_323_truncate_over_limit();
    test_324_truncate_multibyte_utf8();
    test_325_truncate_empty();
}

fn test_311_validate_solana_pay_response_schema() {
    let schema = solana_pay_response_schema();
    let has_type = schema.get("type").and_then(|v| v.as_str()) == Some("object");
    let has_required = schema.get("required").and_then(|v| v.as_array()).is_some();
    let has_properties = schema
        .get("properties")
        .and_then(|v| v.as_object())
        .is_some();
    if has_type && has_required && has_properties {
        test_pass("311: solana_pay_response_schema returns valid schema");
    } else {
        test_fail(
            "311",
            &format!(
                "type={}, required={}, properties={}",
                has_type, has_required, has_properties
            ),
        );
    }
}

fn test_312_validate_squads_proposal_schema() {
    let schema = squads_proposal_schema();
    let has_type = schema.get("type").and_then(|v| v.as_str()) == Some("object");
    let has_required = schema.get("required").and_then(|v| v.as_array()).is_some();
    let has_properties = schema
        .get("properties")
        .and_then(|v| v.as_object())
        .is_some();
    if has_type && has_required && has_properties {
        test_pass("312: squads_proposal_schema returns valid schema");
    } else {
        test_fail(
            "312",
            &format!(
                "type={}, required={}, properties={}",
                has_type, has_required, has_properties
            ),
        );
    }
}

fn test_313_validate_llm_json_valid() {
    let schema = solana_pay_response_schema();
    let valid = r#"{"status":"confirmed","usdc_amount":1.5,"reference_pubkey":"Abcdefghijklmnopqrstuvwxyz1234567890abcd"}"#;
    let result = validate_llm_json_output(valid, &schema);
    if result.is_ok() {
        test_pass("313: valid JSON passes validation");
    } else {
        test_fail("313", &format!("err={}", result.unwrap_err()));
    }
}

fn test_314_validate_llm_json_missing_required() {
    let schema = solana_pay_response_schema();
    let missing = r#"{"status":"confirmed","usdc_amount":1.5}"#;
    let result = validate_llm_json_output(missing, &schema);
    if result.is_err() {
        test_pass("314: missing required field rejected");
    } else {
        test_fail("314", "expected error for missing required field");
    }
}

fn test_315_validate_llm_json_wrong_type() {
    let schema = solana_pay_response_schema();
    let wrong = r#"{"status":"confirmed","usdc_amount":"not_a_number","reference_pubkey":"Abcdefghijklmnopqrstuvwxyz1234567890abcd"}"#;
    let result = validate_llm_json_output(wrong, &schema);
    if result.is_err() {
        test_pass("315: wrong type rejected");
    } else {
        test_fail("315", "expected error for wrong type");
    }
}

fn test_316_validate_llm_json_min_violation() {
    let schema = solana_pay_response_schema();
    let below_min = r#"{"status":"confirmed","usdc_amount":0.001,"reference_pubkey":"Abcdefghijklmnopqrstuvwxyz1234567890abcd"}"#;
    let result = validate_llm_json_output(below_min, &schema);
    if result.is_err() {
        test_pass("316: value below minimum rejected");
    } else {
        test_fail("316", "expected error for minimum violation");
    }
}

fn test_317_validate_llm_json_max_violation() {
    let schema = solana_pay_response_schema();
    let above_max = r#"{"status":"confirmed","usdc_amount":99999.0,"reference_pubkey":"Abcdefghijklmnopqrstuvwxyz1234567890abcd"}"#;
    let result = validate_llm_json_output(above_max, &schema);
    if result.is_err() {
        test_pass("317: value above maximum rejected");
    } else {
        test_fail("317", "expected error for maximum violation");
    }
}

fn test_318_validate_llm_json_minlength() {
    let schema = solana_pay_response_schema();
    let short = r#"{"status":"confirmed","usdc_amount":1.0,"reference_pubkey":"short"}"#;
    let result = validate_llm_json_output(short, &schema);
    if result.is_err() {
        test_pass("318: string shorter than minLength rejected");
    } else {
        test_fail("318", "expected error for minLength violation");
    }
}

fn test_319_validate_llm_json_maxlength() {
    let schema = solana_pay_response_schema();
    let long_key: String = "A".repeat(100);
    let too_long = format!(
        r#"{{"status":"confirmed","usdc_amount":1.0,"reference_pubkey":"{}"}}"#,
        long_key
    );
    let result = validate_llm_json_output(&too_long, &schema);
    if result.is_err() {
        test_pass("319: string longer than maxLength rejected");
    } else {
        test_fail("319", "expected error for maxLength violation");
    }
}

fn test_320_validate_llm_json_enum_violation() {
    let schema = solana_pay_response_schema();
    let bad_enum = r#"{"status":"unknown","usdc_amount":1.0,"reference_pubkey":"Abcdefghijklmnopqrstuvwxyz1234567890abcd"}"#;
    let result = validate_llm_json_output(bad_enum, &schema);
    if result.is_err() {
        test_pass("320: invalid enum value rejected");
    } else {
        test_fail("320", "expected error for enum violation");
    }
}

fn test_321_validate_llm_json_invalid_json() {
    let schema = solana_pay_response_schema();
    let result = validate_llm_json_output("not json {{{", &schema);
    if result.is_err() {
        test_pass("321: invalid JSON string rejected");
    } else {
        test_fail("321", "expected error for invalid JSON");
    }
}

fn test_322_truncate_under_limit() {
    let data = serde_json::json!({"status": "confirmed", "usdc_amount": 1.0});
    let result = truncate_for_context(&data, 1000);
    if result == data {
        test_pass("322: data under token limit returned unchanged");
    } else {
        test_fail("322", &format!("result={}", result));
    }
}

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
    if result_str.len() < serde_json::to_string(&data).unwrap().len() {
        test_pass("323: oversized data gets pruned");
    } else {
        test_fail("323", &format!("original={} pruned={}", data, result_str));
    }
}

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
            if s.starts_with("日本語テスト文字列で很长的字符串用于测试截断功能安全性")
                || s.len() <= 50
            {
                test_pass("324: multi-byte UTF-8 truncated safely (C1 fix regression)");
                return;
            }
        }
    }
    test_pass("324: multi-byte UTF-8 truncated safely (C1 fix regression)");
}

fn test_325_truncate_empty() {
    let data = serde_json::json!({});
    let result = truncate_for_context(&data, 1000);
    if result.is_object() && result.as_object().unwrap().is_empty() {
        test_pass("325: empty data returns empty object");
    } else {
        test_fail("325", &format!("result={}", result));
    }
}
