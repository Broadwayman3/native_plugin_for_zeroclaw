use once_cell::sync::Lazy;
use regex::Regex;

static RE_CURRENCY: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)(\d+(?:\.\d+)?)\s*(₴|\$|€|R\$|ZŁ|TL|[a-zA-Z]{3})").unwrap());

static RE_CURRENCY_PREFIX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)(₴|\$|€|R\$|ZŁ|TL)\s*(\d+(?:\.\d+)?)").unwrap());

static RE_PLAIN_NUMBER: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*(\d+(?:\.\d+)?)\s*$").unwrap());

static RE_QTY: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?i)(\d+(?:\.\d+)?)\s*[xX*]\s*(.+)$").unwrap());

/// Parsed POS order input.
#[derive(Debug, Clone)]
pub struct ParsedOrder {
    pub has_price: bool,
    pub amount: Option<f64>,
    pub currency: Option<String>,
    pub items: String,
}

/// Parses POS order input text to extract amount, currency, and items.
/// Supports multi-segment orders ("2x Latte 50 UAH + 1x Muffin 30 UAH")
/// and fractional quantity multipliers ("1.5x Espresso 30 UAH").
///
/// NOTE: This function does NOT call sanitize_external_input().
/// The caller (handle_create_order in pos_flow.rs) is responsible for sanitization.
pub fn parse_pos_order_input(
    text: &str,
    default_item_label: &str,
    draft_items: Option<&str>,
) -> ParsedOrder {
    let text_clean = text.trim();

    // Check if input contains multi-segment splitters (+ or newline)
    let segments: Vec<&str> = if text_clean.contains('+') || text_clean.contains('\n') {
        text_clean
            .split(['+', '\n'])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        vec![text_clean]
    };

    if segments.len() > 1 {
        let mut total_amt = 0.0;
        let mut main_curr: Option<String> = None;
        let mut parsed_items = Vec::new();
        let mut valid_count = 0;
        let mut has_mixed_currency = false;

        for seg in segments {
            let seg_res = parse_single_segment(seg, default_item_label, draft_items);
            if seg_res.has_price {
                if let (Some(amt), Some(curr)) = (seg_res.amount, seg_res.currency) {
                    if let Some(ref existing) = main_curr {
                        if existing != &curr {
                            has_mixed_currency = true;
                            break;
                        }
                    } else {
                        main_curr = Some(curr);
                    }
                    total_amt += amt;
                    valid_count += 1;
                    parsed_items.push(seg_res.items);
                }
            } else {
                parsed_items.push(seg.to_string());
            }
        }

        if has_mixed_currency {
            return ParsedOrder {
                has_price: false,
                amount: None,
                currency: None,
                items: text_clean.to_string(),
            };
        }

        if valid_count > 0 && total_amt <= 999_999.99 {
            return ParsedOrder {
                has_price: true,
                amount: Some(total_amt),
                currency: main_curr,
                items: parsed_items.join(" + "),
            };
        }
    }

    parse_single_segment(text_clean, default_item_label, draft_items)
}

fn parse_single_segment(
    seg_text: &str,
    default_item_label: &str,
    draft_items: Option<&str>,
) -> ParsedOrder {
    let seg_clean = seg_text.trim();

    // Check for quantity multiplier at the start of segment (e.g. "2x Espresso 40 UAH", "1.5x Cake 50 UAH")
    let (qty, core_text) = if let Some(caps) = RE_QTY.captures(seg_clean) {
        let q: f64 = caps
            .get(1)
            .map_or(1.0, |m| m.as_str().parse().unwrap_or(1.0));
        if q <= 0.0 || !q.is_finite() {
            return ParsedOrder {
                has_price: false,
                amount: None,
                currency: None,
                items: seg_clean.to_string(),
            };
        }
        let rest = caps.get(2).map_or(seg_clean, |m| m.as_str().trim());
        (q, rest)
    } else {
        (1.0, seg_clean)
    };

    // Try to match currency postfix pattern: "150 UAH", "35.50 BRL", "12 USD"
    if let Some(caps) = RE_CURRENCY.captures(core_text) {
        let matched_amt: f64 = caps.get(1).unwrap().as_str().parse().unwrap_or_default();

        let match_start = caps.get(0).unwrap().start();
        if match_start > 0 {
            let prev_char = core_text.chars().nth(match_start - 1);
            if prev_char == Some('-') {
                return ParsedOrder {
                    has_price: false,
                    amount: None,
                    currency: None,
                    items: seg_clean.to_string(),
                };
            }
        }

        if matched_amt <= 0.0 || !matched_amt.is_finite() || matched_amt > 999_999.99 {
            return ParsedOrder {
                has_price: false,
                amount: None,
                currency: None,
                items: seg_clean.to_string(),
            };
        }

        let curr_str = caps.get(2).unwrap().as_str();
        let curr_upper = curr_str.to_uppercase();
        let curr = match curr_upper.as_str() {
            "₴" | "UAH" => "UAH",
            "$" | "USD" => "USD",
            "€" | "EUR" => "EUR",
            "R$" | "REAL" | "BRL" => "BRL",
            "ZŁ" | "PLN" => "PLN",
            _ => curr_str,
        }
        .to_uppercase();

        let matched_str = caps.get(0).unwrap().as_str();
        let items_part = core_text.replace(matched_str, "").trim().to_string();

        let (final_item, total_amt) = if !items_part.is_empty() {
            let item_str = if RE_QTY.is_match(seg_clean) && !seg_clean.starts_with(&items_part) {
                format!("{}x {}", qty, items_part)
            } else {
                items_part
            };
            (item_str, matched_amt)
        } else if let Some(draft) = draft_items {
            (draft.to_string(), matched_amt * qty)
        } else {
            let item_str = if qty != 1.0 {
                format!("{}x {}", qty, default_item_label)
            } else {
                format!("{} {} {}", default_item_label, matched_amt, curr)
            };
            (item_str, matched_amt * qty)
        };

        return ParsedOrder {
            has_price: true,
            amount: Some(total_amt),
            currency: Some(curr),
            items: final_item,
        };
    }

    // Try currency prefix patterns: "$50", "€25.50", "R$100"
    if let Some(caps) = RE_CURRENCY_PREFIX.captures(core_text) {
        let curr_str = caps.get(1).unwrap().as_str();
        let curr_upper = curr_str.to_uppercase();
        let matched_amt: f64 = caps.get(2).unwrap().as_str().parse().unwrap_or_default();

        if matched_amt <= 0.0 || !matched_amt.is_finite() || matched_amt > 999_999.99 {
            return ParsedOrder {
                has_price: false,
                amount: None,
                currency: None,
                items: seg_clean.to_string(),
            };
        }

        let curr = match curr_upper.as_str() {
            "₴" | "UAH" => "UAH",
            "$" | "USD" => "USD",
            "€" | "EUR" => "EUR",
            "R$" | "REAL" | "BRL" => "BRL",
            "ZŁ" | "PLN" => "PLN",
            _ => curr_str,
        }
        .to_uppercase();

        let matched_str = caps.get(0).unwrap().as_str();
        let items_part = core_text.replace(matched_str, "").trim().to_string();

        let (final_item, total_amt) = if !items_part.is_empty() {
            let item_str = if RE_QTY.is_match(seg_clean) && !seg_clean.starts_with(&items_part) {
                format!("{}x {}", qty, items_part)
            } else {
                items_part
            };
            (item_str, matched_amt)
        } else if let Some(draft) = draft_items {
            (draft.to_string(), matched_amt * qty)
        } else {
            let item_str = if qty != 1.0 {
                format!("{}x {}", qty, default_item_label)
            } else {
                format!("{} {} {}", default_item_label, matched_amt, curr)
            };
            (item_str, matched_amt * qty)
        };

        return ParsedOrder {
            has_price: true,
            amount: Some(total_amt),
            currency: Some(curr),
            items: final_item,
        };
    }

    // Try bare number (defaults to UAH)
    if let Some(caps) = RE_PLAIN_NUMBER.captures(core_text) {
        let matched_amt: f64 = caps.get(1).unwrap().as_str().parse().unwrap_or_default();

        if matched_amt <= 0.0 || !matched_amt.is_finite() || matched_amt > 999_999.99 {
            return ParsedOrder {
                has_price: false,
                amount: None,
                currency: None,
                items: seg_clean.to_string(),
            };
        }

        let (final_item, total_amt) = if let Some(draft) = draft_items {
            (draft.to_string(), matched_amt * qty)
        } else {
            let item_str = if qty != 1.0 {
                format!("{}x {}", qty, default_item_label)
            } else {
                format!("{} {} UAH", default_item_label, matched_amt)
            };
            (item_str, matched_amt * qty)
        };

        return ParsedOrder {
            has_price: true,
            amount: Some(total_amt),
            currency: Some("UAH".to_string()),
            items: final_item,
        };
    }

    // No price found
    ParsedOrder {
        has_price: false,
        amount: None,
        currency: None,
        items: seg_clean.to_string(),
    }
}
