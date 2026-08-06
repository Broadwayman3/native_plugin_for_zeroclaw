use serde_json::Value;

use crate::config::AppConfig;
use crate::db;
use crate::domain::i18n::{format_itemized_receipt, get_cancel_invoice_inline_keyboard, t, t_raw};
use crate::domain::sanitizer::sanitize_external_input;

/// Processes an incoming POS order text message.
pub async fn handle_pos_order(
    client: &reqwest::Client,
    base_url: &str,
    config: &AppConfig,
    chat_id: i64,
    lang: &str,
    text: &str,
) {
    let sanitized = sanitize_external_input(text, 100);
    let def_label = t_raw("default_item", Some(lang), &[]);
    let parsed = crate::domain::order_parser::parse_pos_order_input(&sanitized, &def_label, None);

    if !parsed.has_price {
        let prompt_text = t("price_needed", Some(lang), &[("items", &parsed.items)]);
        let payload = serde_json::json!({
            "chat_id": chat_id,
            "text": prompt_text,
            "parse_mode": "MarkdownV2",
            "reply_markup": {
                "force_reply": true,
                "selective": true
            }
        });
        let _ = client
            .post(format!("{}/sendMessage", base_url))
            .json(&payload)
            .send()
            .await;
        return;
    }

    let fiat_amt = parsed.amount.unwrap_or_default();
    let fiat_curr = parsed.currency.as_deref().unwrap_or("UAH");

    if fiat_amt <= 0.0 || !fiat_amt.is_finite() {
        let payload = serde_json::json!({
            "chat_id": chat_id,
            "text": "Error: Amount must be positive.",
        });
        let _ = client
            .post(format!("{}/sendMessage", base_url))
            .json(&payload)
            .send()
            .await;
        return;
    }

    let rate_info = match crate::domain::price_feed::get_multitier_fiat_rate(
        fiat_curr, None, None, None, None, true,
    ) {
        Ok(r) => r,
        Err(_) => {
            let payload = serde_json::json!({
                "chat_id": chat_id,
                "text": "Error: Price feed unavailable for this currency.",
            });
            let _ = client
                .post(format!("{}/sendMessage", base_url))
                .json(&payload)
                .send()
                .await;
            return;
        }
    };

    let rate = rate_info
        .get("rate")
        .and_then(|v| v.as_f64())
        .unwrap_or(1.0);
    let usdc_atomic = pos_core_logic::safe_f64_to_u64_atomic(fiat_amt / rate, 6);
    let usdc_amt = usdc_atomic as f64 / 1_000_000.0;

    let inv_id = format!("INV-{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let ref_key = pos_core_logic::generate_secure_reference_key();

    // 1. Create invoice record in SQLite with chat_id (telegram_msg_id populated after sendPhoto)
    if let Ok(conn) = db::get_db_connection(&config.db_path) {
        let _ = db::invoices::create_invoice(
            &conn,
            &db::invoices::CreateInvoiceRequest {
                id: inv_id.clone(),
                reference_pubkey: ref_key.clone(),
                fiat_currency: Some(fiat_curr.to_string()),
                fiat_amount: Some(fiat_amt),
                usdc_amount: usdc_amt,
                telegram_chat_id: Some(chat_id),
                telegram_msg_id: None,
            },
        );
    }

    let solana_url = pos_core_logic::build_solana_pay_url(
        &config.merchant_wallet_pubkey,
        usdc_amt,
        &ref_key,
        Some(&config.usdc_mint_address),
        "ZeroClaw POS",
        "POS Payment",
    );
    let phantom_link = pos_core_logic::solana_pay::generate_phantom_universal_link(&solana_url);
    let qr_url = crate::domain::formatters::generate_solana_pay_qr_image_url(&solana_url, 300);

    let receipt = format_itemized_receipt(
        &inv_id,
        &parsed.items,
        0.0,
        usdc_amt,
        lang,
        Some(fiat_curr),
        Some(fiat_amt),
        Some(rate),
    );

    let keyboard = get_cancel_invoice_inline_keyboard(&inv_id, Some(&phantom_link), lang);

    let photo_payload = serde_json::json!({
        "chat_id": chat_id,
        "photo": qr_url,
        "caption": receipt,
        "parse_mode": "MarkdownV2",
        "reply_markup": keyboard
    });

    // 2. Send photo and update telegram_msg_id in SQLite upon response
    if let Ok(resp) = client
        .post(format!("{}/sendPhoto", base_url))
        .json(&photo_payload)
        .send()
        .await
    {
        if let Ok(json) = resp.json::<Value>().await {
            if let Some(msg_id) = json
                .get("result")
                .and_then(|r| r.get("message_id"))
                .and_then(|v| v.as_i64())
            {
                if let Ok(conn) = db::get_db_connection(&config.db_path) {
                    let _ = db::invoices::update_invoice_telegram_context(
                        &conn, &inv_id, chat_id, msg_id,
                    );
                }
            }
        }
    }
}
