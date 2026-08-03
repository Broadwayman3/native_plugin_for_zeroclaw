use axum::extract::State;
use axum::Json;

use crate::db;
use crate::domain;
use crate::error::AppError;

/// POST /api/v1/pos/create-order - Create an order from parsed POS input
/// This replaces the handle_text_message flow from the Python bot.
#[derive(serde::Deserialize)]
pub struct CreateOrderRequest {
    pub chat_id: i64,
    pub text: String,
    pub lang: Option<String>,
    pub draft_items: Option<String>,
}

pub async fn handle_create_order(
    State(state): State<crate::api::AppState>,
    Json(req): Json<CreateOrderRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let lang = req.lang.as_deref().unwrap_or("en");

    // Sanitize input
    let sanitized = domain::sanitizer::sanitize_external_input(&req.text, 100);
    if sanitized.is_empty() {
        return Err(AppError::BadRequest("Empty input".to_string()));
    }

    // Parse order
    let def_label = domain::i18n::t_raw("default_item", Some(lang), &[]);
    let parsed = domain::order_parser::parse_pos_order_input(
        &sanitized,
        &def_label,
        req.draft_items.as_deref(),
    );

    let has_price = parsed
        .get("has_price")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if !has_price {
        // Need price - return prompt
        let items = parsed.get("items").and_then(|v| v.as_str()).unwrap_or("");
        let prompt_text = domain::i18n::t("price_needed", Some(lang), &[("items", items)]);

        return Ok(Json(serde_json::json!({
            "action": "prompt_price",
            "message": prompt_text,
            "items": items,
            "parse_mode": "MarkdownV2"
        })));
    }

    let fiat_amt = parsed.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let fiat_curr = parsed
        .get("currency")
        .and_then(|v| v.as_str())
        .unwrap_or("UAH");
    let item_desc = parsed.get("items").and_then(|v| v.as_str()).unwrap_or("");

    // Get fiat rate
    let rate_info =
        domain::price_feed::get_multitier_fiat_rate(fiat_curr, None, None, None, None, true)
            .unwrap_or_else(|_| serde_json::json!({"rate": 1.0}));
    let rate = rate_info
        .get("rate")
        .and_then(|v| v.as_f64())
        .unwrap_or(1.0);

    let usdc_amt = (fiat_amt / rate * 100.0).round() / 100.0;

    // Generate invoice
    let inv_id = format!("INV-{}", rand::random::<u32>() % 800 + 200);
    let ref_key = pos_core_logic::generate_secure_reference_key();

    let conn = db::get_db_connection(&state.config.db_path)?;
    db::invoices::create_invoice(
        &conn,
        &db::invoices::CreateInvoiceRequest {
            id: inv_id.clone(),
            reference_pubkey: ref_key.clone(),
            fiat_currency: Some(fiat_curr.to_string()),
            fiat_amount: Some(fiat_amt),
            usdc_amount: usdc_amt,
        },
    )?;

    // Generate Solana Pay URL
    let solana_url = pos_core_logic::build_solana_pay_url(
        &state.config.merchant_wallet_pubkey,
        usdc_amt,
        &ref_key,
        Some(&state.config.usdc_mint_address),
        "ZeroClaw POS",
        "POS Payment",
    );

    let qr_url = domain::formatters::generate_solana_pay_qr_image_url(&solana_url, 300);
    let receipt = domain::i18n::format_itemized_receipt(
        &inv_id,
        item_desc,
        0.0,
        usdc_amt,
        lang,
        Some(fiat_curr),
        Some(fiat_amt),
        Some(rate),
    );

    let keyboard = domain::i18n::get_cancel_invoice_inline_keyboard(&inv_id, lang);

    Ok(Json(serde_json::json!({
        "action": "invoice_created",
        "invoice_id": inv_id,
        "receipt": receipt,
        "qr_url": qr_url,
        "solana_pay_url": solana_url,
        "reply_markup": keyboard,
        "parse_mode": "MarkdownV2"
    })))
}
