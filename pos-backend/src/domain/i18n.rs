use crate::domain::i18n_strings::{LANG_META, TRANSLATIONS};
use crate::domain::sanitizer::escape_telegram_markdown_v2;

/// Normalizes language code (e.g., "pt-BR" -> "pt", "zh-CN" -> "zh").
fn normalize_lang(lang: &str) -> String {
    let clean = lang.to_lowercase();
    let clean = clean.split('-').next().unwrap_or("en");
    clean.split('_').next().unwrap_or("en").to_string()
}

/// Retrieves (flag_emoji, native_name) tuple for a language code.
pub fn get_lang_meta(lang_code: &str) -> (&'static str, &'static str) {
    let clean = normalize_lang(lang_code);
    LANG_META
        .get(clean.as_str())
        .copied()
        .unwrap_or(("\u{1F1FA}\u{1F1F3}", "English"))
}

/// Returns localized language change confirmation message.
pub fn get_localized_confirmation(lang_code: &str) -> String {
    let (flag, name) = get_lang_meta(lang_code);
    let clean = normalize_lang(lang_code);
    let template = TRANSLATIONS
        .get(clean.as_str())
        .and_then(|d| d.get("lang_confirm"))
        .unwrap_or(&"🌐 Interface language successfully changed to {flag} {lang_name}!");

    template
        .replace("{flag}", flag)
        .replace("{lang_name}", name)
}

/// Retrieves localized message with MarkdownV2 escaping (default for Telegram output).
pub fn t(key: &str, lang: Option<&str>, kwargs: &[(&str, &str)]) -> String {
    t_impl(key, lang, true, kwargs)
}

/// Retrieves localized message WITHOUT escaping (for internal use, keyboards, etc.).
pub fn t_raw(key: &str, lang: Option<&str>, kwargs: &[(&str, &str)]) -> String {
    t_impl(key, lang, false, kwargs)
}

/// Backward-compatible alias for t() (Python's get_localized_message).
/// Returns escaped message by default (safe for Telegram output).
pub fn get_localized_message(key: &str, lang: &str, kwargs: &[(&str, &str)]) -> String {
    t(key, Some(lang), kwargs)
}

/// Internal implementation for localized message retrieval.
fn t_impl(key: &str, lang: Option<&str>, escape_markdown: bool, kwargs: &[(&str, &str)]) -> String {
    let clean = normalize_lang(lang.unwrap_or("en"));
    let template = TRANSLATIONS
        .get(clean.as_str())
        .and_then(|d| d.get(key))
        .or_else(|| TRANSLATIONS["en"].get(key))
        .copied()
        .unwrap_or(key);

    let mut result = template.to_string();
    for (k, v) in kwargs {
        result = result.replace(&format!("{{{}}}", k), v);
    }

    if escape_markdown {
        escape_telegram_markdown_v2(&result)
    } else {
        result
    }
}

/// Generates localized cashier persistent reply keyboard.
pub fn get_main_reply_keyboard(lang: &str) -> serde_json::Value {
    let clean = normalize_lang(lang);
    serde_json::json!({
        "keyboard": [
            [{"text": t_raw("btn_custom", Some(&clean), &[])}, {"text": t_raw("btn_quick_uah", Some(&clean), &[])}],
            [{"text": t_raw("btn_sales", Some(&clean), &[])}, {"text": t_raw("btn_refund", Some(&clean), &[])}],
            [{"text": t_raw("btn_lang", Some(&clean), &[])}]
        ],
        "resize_keyboard": true
    })
}

/// Generates inline keyboard for invoice cancellation.
pub fn get_cancel_invoice_inline_keyboard(invoice_id: &str, lang: &str) -> serde_json::Value {
    let clean = normalize_lang(lang);
    let btn_label = t_raw("cancel_btn_text", Some(&clean), &[]);
    serde_json::json!({
        "inline_keyboard": [[
            {"text": btn_label, "callback_data": format!("cancel_invoice_{}", invoice_id)}
        ]]
    })
}

/// Builds inline keyboard for Squads v4 refund approve/reject.
pub fn get_refund_checkpoint_inline_keyboard(refund_id: i64, lang: &str) -> serde_json::Value {
    let approve_label = t_raw("btn_approve", Some(lang), &[]);
    let reject_label = t_raw("btn_reject", Some(lang), &[]);
    serde_json::json!({
        "inline_keyboard": [[
            {"text": approve_label, "callback_data": format!("approve_refund_{}", refund_id)},
            {"text": reject_label, "callback_data": format!("reject_refund_{}", refund_id)}
        ]]
    })
}

/// Formats an itemized POS receipt with MarkdownV2 escaping.
#[allow(clippy::too_many_arguments)]
pub fn format_itemized_receipt(
    invoice_id: &str,
    items: &str,
    tax_rate_pct: f64,
    amount_usdc: f64,
    lang: &str,
    fiat_currency: Option<&str>,
    fiat_amount: Option<f64>,
    exchange_rate: Option<f64>,
) -> String {
    let tax_amount = (amount_usdc * (tax_rate_pct / 100.0) * 100.0).round() / 100.0;
    let default_item = t_raw("default_item", Some(lang), &[]);

    let title_escaped = t("receipt_title", Some(lang), &[("invoice_id", invoice_id)]);
    let tax_escaped = t(
        "receipt_tax",
        Some(lang),
        &[
            ("tax_rate_pct", &format!("{:.0}", tax_rate_pct)),
            ("tax_amount", &format!("{:.2}", tax_amount)),
        ],
    );
    let total_escaped = t(
        "receipt_total",
        Some(lang),
        &[("amount_usdc", &format!("{:.2}", amount_usdc))],
    );

    let raw_items = if items.is_empty() {
        default_item.as_str()
    } else {
        items
    };
    let items_escaped = escape_telegram_markdown_v2(raw_items);
    let items_formatted = items_escaped.replace("; ", "\n• ").replace(";", "\n• ");
    let items_formatted = if items_formatted.starts_with("• ") {
        items_formatted
    } else {
        format!("• {}", items_formatted)
    };

    let fiat_conversion_line =
        if let (Some(curr), Some(amt), Some(rate)) = (fiat_currency, fiat_amount, exchange_rate) {
            if rate > 0.0 {
                let curr_escaped = escape_telegram_markdown_v2(curr);
                let amt_escaped = escape_telegram_markdown_v2(&format!("{:.2}", amt));
                let rate_escaped = escape_telegram_markdown_v2(&format!("{:.2}", rate));
                format!(
                    "• Charged: {} {} \\(Rate: {}\\)\n",
                    amt_escaped, curr_escaped, rate_escaped
                )
            } else {
                String::new()
            }
        } else {
            String::new()
        };

    format!(
        "*{}*\n\
         ───────────────────────────\n\
         {}\n\
         ───────────────────────────\n\
         • {}\n\
         {}\
         • *{}*",
        title_escaped, items_formatted, tax_escaped, fiat_conversion_line, total_escaped
    )
}
