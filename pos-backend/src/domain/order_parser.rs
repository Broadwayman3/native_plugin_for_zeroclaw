use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;

static RE_CURRENCY: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)(\d+(?:\.\d+)?)\s*([a-zA-Z]{3}|₴|\$|€|R\$|zł|TL)\b").unwrap());

static RE_PLAIN_NUMBER: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*(\d+(?:\.\d+)?)\s*$").unwrap());

/// Parses POS order input text to extract amount, currency, and items.
///
/// NOTE: This function does NOT call sanitize_external_input().
/// The caller (handle_create_order in pos_flow.rs) is responsible for sanitization.
/// This is intentional — avoids double-sanitization and keeps this function pure.
pub fn parse_pos_order_input(
    text: &str,
    default_item_label: &str,
    draft_items: Option<&str>,
) -> HashMap<String, serde_json::Value> {
    let text_clean = text.trim();
    let mut result = HashMap::new();

    // Try to match currency patterns: "150 UAH", "35.50 BRL", "12 USD", "$100", "€50", etc.
    if let Some(caps) = RE_CURRENCY.captures(text_clean) {
        let amt: f64 = caps.get(1).unwrap().as_str().parse().unwrap_or(0.0);

        if amt <= 0.0 || !amt.is_finite() || amt > 999_999.99 {
            let mut r = HashMap::new();
            r.insert("has_price".to_string(), serde_json::Value::Bool(false));
            r.insert(
                "items".to_string(),
                serde_json::Value::String(text_clean.to_string()),
            );
            r.insert("amount".to_string(), serde_json::Value::Null);
            r.insert("currency".to_string(), serde_json::Value::Null);
            return r;
        }

        let curr_str = caps.get(2).unwrap().as_str();

        let curr = match curr_str {
            "₴" => "UAH",
            "$" => "USD",
            "€" => "EUR",
            "R$" | "REAL" => "BRL",
            "ZŁ" => "PLN",
            _ => curr_str,
        }
        .to_uppercase();

        let matched_str = caps.get(0).unwrap().as_str();
        let items_part = text_clean.replace(matched_str, "").trim().to_string();

        let final_item = if !items_part.is_empty() {
            items_part
        } else if let Some(draft) = draft_items {
            draft.to_string()
        } else {
            format!("{} {} {}", default_item_label, amt, curr)
        };

        result.insert("has_price".to_string(), serde_json::Value::Bool(true));
        result.insert("items".to_string(), serde_json::Value::String(final_item));
        result.insert("amount".to_string(), serde_json::json!(amt));
        result.insert("currency".to_string(), serde_json::Value::String(curr));
        return result;
    }

    // Try bare number (defaults to UAH)
    if let Some(caps) = RE_PLAIN_NUMBER.captures(text_clean) {
        let amt: f64 = caps.get(1).unwrap().as_str().parse().unwrap_or(0.0);

        if amt <= 0.0 || !amt.is_finite() || amt > 999_999.99 {
            let mut r = HashMap::new();
            r.insert("has_price".to_string(), serde_json::Value::Bool(false));
            r.insert(
                "items".to_string(),
                serde_json::Value::String(text_clean.to_string()),
            );
            r.insert("amount".to_string(), serde_json::Value::Null);
            r.insert("currency".to_string(), serde_json::Value::Null);
            return r;
        }

        let final_item = if let Some(draft) = draft_items {
            draft.to_string()
        } else {
            format!("{} {} UAH", default_item_label, amt)
        };

        result.insert("has_price".to_string(), serde_json::Value::Bool(true));
        result.insert("items".to_string(), serde_json::Value::String(final_item));
        result.insert("amount".to_string(), serde_json::json!(amt));
        result.insert(
            "currency".to_string(),
            serde_json::Value::String("UAH".to_string()),
        );
        return result;
    }

    // No price found
    result.insert("has_price".to_string(), serde_json::Value::Bool(false));
    result.insert(
        "items".to_string(),
        serde_json::Value::String(text_clean.to_string()),
    );
    result.insert("amount".to_string(), serde_json::Value::Null);
    result.insert("currency".to_string(), serde_json::Value::Null);
    result
}
