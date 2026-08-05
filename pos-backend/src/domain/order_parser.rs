use once_cell::sync::Lazy;
use regex::Regex;

static RE_CURRENCY: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)(\d+(?:\.\d+)?)\s*(₴|\$|€|R\$|ZŁ|TL|[a-zA-Z]{3})").unwrap());

static RE_CURRENCY_PREFIX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)(₴|\$|€|R\$|ZŁ|TL)\s*(\d+(?:\.\d+)?)").unwrap());

static RE_PLAIN_NUMBER: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*(\d+(?:\.\d+)?)\s*$").unwrap());

/// Parsed POS order input.
#[derive(Debug, Clone)]
pub struct ParsedOrder {
    pub has_price: bool,
    pub amount: Option<f64>,
    pub currency: Option<String>,
    pub items: String,
}

/// Parses POS order input text to extract amount, currency, and items.
///
/// NOTE: This function does NOT call sanitize_external_input().
/// The caller (handle_create_order in pos_flow.rs) is responsible for sanitization.
/// This is intentional — avoids double-sanitization and keeps this function pure.
pub fn parse_pos_order_input(
    text: &str,
    default_item_label: &str,
    draft_items: Option<&str>,
) -> ParsedOrder {
    let text_clean = text.trim();

    // Multiple price aggregation (e.g. "Latte 120 UAH + Croissant 80 UAH")
    let matches: Vec<_> = RE_CURRENCY.captures_iter(text_clean).collect();
    if matches.len() > 1 {
        let mut total_amt = 0.0;
        let mut main_curr: Option<String> = None;
        let mut valid_count = 0;
        let mut has_mixed_currency = false;

        for caps in &matches {
            let amt: f64 = caps.get(1).unwrap().as_str().parse().unwrap_or_default();

            let match_start = caps.get(0).unwrap().start();
            if match_start > 0 {
                let prev_char = text_clean.chars().nth(match_start - 1);
                if prev_char == Some('-') {
                    continue;
                }
            }

            if amt > 0.0 && amt.is_finite() && amt <= 999_999.99 {
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
            let items_clean = RE_CURRENCY.replace_all(text_clean, "").to_string();
            let final_item = items_clean
                .split('+')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join(" + ");

            return ParsedOrder {
                has_price: true,
                amount: Some(total_amt),
                currency: main_curr,
                items: if final_item.is_empty() {
                    text_clean.to_string()
                } else {
                    final_item
                },
            };
        }
    }

    // Try to match currency patterns: "150 UAH", "35.50 BRL", "12 USD", "$100", "€50", etc.
    if let Some(caps) = RE_CURRENCY.captures(text_clean) {
        let amt: f64 = caps.get(1).unwrap().as_str().parse().unwrap_or_default();

        // Reject negative amounts (check char before the match)
        let match_start = caps.get(0).unwrap().start();
        if match_start > 0 {
            let prev_char = text_clean.chars().nth(match_start - 1);
            if prev_char == Some('-') {
                return ParsedOrder {
                    has_price: false,
                    amount: None,
                    currency: None,
                    items: text_clean.to_string(),
                };
            }
        }

        if amt <= 0.0 || !amt.is_finite() || amt > 999_999.99 {
            return ParsedOrder {
                has_price: false,
                amount: None,
                currency: None,
                items: text_clean.to_string(),
            };
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

        return ParsedOrder {
            has_price: true,
            amount: Some(amt),
            currency: Some(curr),
            items: final_item,
        };
    }

    // Try currency prefix patterns: "$50", "€25.50", "R$100"
    if let Some(caps) = RE_CURRENCY_PREFIX.captures(text_clean) {
        let curr_str = caps.get(1).unwrap().as_str();
        let amt: f64 = caps.get(2).unwrap().as_str().parse().unwrap_or_default();

        if amt <= 0.0 || !amt.is_finite() || amt > 999_999.99 {
            return ParsedOrder {
                has_price: false,
                amount: None,
                currency: None,
                items: text_clean.to_string(),
            };
        }

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

        return ParsedOrder {
            has_price: true,
            amount: Some(amt),
            currency: Some(curr),
            items: final_item,
        };
    }

    // Try bare number (defaults to UAH)
    if let Some(caps) = RE_PLAIN_NUMBER.captures(text_clean) {
        let amt: f64 = caps.get(1).unwrap().as_str().parse().unwrap_or_default();

        if amt <= 0.0 || !amt.is_finite() || amt > 999_999.99 {
            return ParsedOrder {
                has_price: false,
                amount: None,
                currency: None,
                items: text_clean.to_string(),
            };
        }

        let final_item = if let Some(draft) = draft_items {
            draft.to_string()
        } else {
            format!("{} {} UAH", default_item_label, amt)
        };

        return ParsedOrder {
            has_price: true,
            amount: Some(amt),
            currency: Some("UAH".to_string()),
            items: final_item,
        };
    }

    // No price found
    ParsedOrder {
        has_price: false,
        amount: None,
        currency: None,
        items: text_clean.to_string(),
    }
}
