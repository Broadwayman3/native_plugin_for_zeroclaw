use super::client::{
    generate_qr_code_png_bytes, send_telegram_photo_bytes, send_telegram_request,
    start_chat_action_loop,
};

use super::fsm::FsmStore;
use crate::api::telegram::handlers::refund::is_manager_authorized;
use crate::config::AppConfig;
use crate::db;
use crate::domain::i18n::{format_itemized_receipt, get_cancel_invoice_inline_keyboard, t, t_raw};
use crate::domain::sanitizer::sanitize_item_name;

/// Processes an incoming POS order text message with FSM context & reply_to_message support.
#[allow(clippy::too_many_arguments)]
pub async fn handle_pos_order(
    client: &reqwest::Client,
    base_url: &str,
    config: &AppConfig,
    fsm: &FsmStore,
    chat_id: i64,
    user_id: i64,
    chat_type: &str,
    lang: &str,
    text: &str,
    _reply_to_text: Option<&str>,
) -> Result<(), String> {
    // In group/supergroup chats, ignore unauthenticated random numeric input without slash command
    if chat_type != "private"
        && !text.starts_with('/')
        && is_manager_authorized(config, user_id).is_err()
    {
        return Ok(());
    }

    let sanitized = sanitize_item_name(text);
    let def_label = t_raw("default_item", Some(lang), &[]);

    // Start background chat action typing indicator loop (automatically aborted on Drop)
    let _action_task = start_chat_action_loop(
        client.clone(),
        base_url.to_string(),
        chat_id,
        "upload_photo",
    );

    // 1. Check FSM for existing pending item for this (chat_id, user_id)
    let pending_session = if user_id > 0 {
        fsm.get_pending(chat_id, user_id).await
    } else {
        None
    };

    let mut parsed =
        crate::domain::order_parser::parse_pos_order_input(&sanitized, &def_label, None);

    // Merge pending item name ONLY if user input supplied price only (e.g. "50 UAH")
    // and contains no explicit custom item name.
    if parsed.has_price && parsed.items == def_label {
        if let Some(ref pending) = pending_session {
            parsed.items = pending.item_name.clone();
            if parsed.currency.is_none() {
                parsed.currency = pending.currency.clone();
            }
        }
    }

    if !parsed.has_price {
        // Save pending item name in FSM store (only for valid user_id > 0)
        if user_id > 0 {
            fsm.set_pending(
                chat_id,
                user_id,
                parsed.items.clone(),
                parsed.currency.clone(),
            )
            .await;
        }

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
        let _ = send_telegram_request(client, &format!("{}/sendMessage", base_url), &payload).await;
        return Ok(());
    }

    let fiat_amt = parsed.amount.unwrap_or_default();
    let fiat_curr = parsed.currency.as_deref().unwrap_or("UAH");

    // Validate amount BEFORE clearing FSM state so user can re-enter price on error!
    if fiat_amt <= 0.0 || !fiat_amt.is_finite() {
        let payload = serde_json::json!({
            "chat_id": chat_id,
            "text": "Error: Amount must be positive.",
        });
        let _ = send_telegram_request(client, &format!("{}/sendMessage", base_url), &payload).await;
        return Ok(());
    }

    let rate_info = match crate::domain::price_feed::get_multitier_fiat_rate(
        fiat_curr, None, None, None, None, true,
    ) {
        Ok(r) => r,
        Err(e) => {
            let payload = serde_json::json!({
                "chat_id": chat_id,
                "text": "Error: Price feed unavailable for this currency.",
            });
            let _ =
                send_telegram_request(client, &format!("{}/sendMessage", base_url), &payload).await;
            return Err(format!("Price feed unavailable: {}", e));
        }
    };

    // Clear FSM state ONLY AFTER price validation and price feed lookup succeed
    if user_id > 0 {
        fsm.clear(chat_id, user_id).await;
    }

    let rate = rate_info
        .get("rate")
        .and_then(|v| v.as_f64())
        .unwrap_or(1.0);
    let usdc_atomic = pos_core_logic::safe_f64_to_u64_atomic(fiat_amt / rate, 6);
    let usdc_amt = usdc_atomic as f64 / 1_000_000.0;

    let inv_id = format!("INV-{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let ref_key = pos_core_logic::generate_secure_reference_key();

    let req = db::invoices::CreateInvoiceRequest {
        id: inv_id.clone(),
        reference_pubkey: ref_key.clone(),
        fiat_currency: Some(fiat_curr.to_string()),
        fiat_amount: Some(fiat_amt),
        usdc_amount: usdc_amt,
        telegram_chat_id: Some(chat_id),
        telegram_msg_id: None,
    };

    // Persist invoice record in SQLite via deadpool-sqlite interact
    let db_saved = if let Some(pool) = fsm.pool() {
        if let Ok(conn) = pool.get().await {
            let req_clone = req.clone();
            let res = conn
                .interact(move |c| db::invoices::create_invoice(c, &req_clone))
                .await;
            matches!(res, Ok(Ok(_)))
        } else {
            false
        }
    } else if let Ok(conn) = db::get_db_connection(&config.db_path) {
        db::invoices::create_invoice(&conn, &req).is_ok()
    } else {
        false
    };

    if !db_saved {
        tracing::error!(inv_id = %inv_id, "Failed to persist invoice to database");
        return Err(format!("Failed to persist invoice {} to database", inv_id));
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

    let mut msg_sent = false;
    if let Ok(photo_bytes) = generate_qr_code_png_bytes(&solana_url) {
        if let Ok(json) = send_telegram_photo_bytes(
            client,
            base_url,
            chat_id,
            photo_bytes,
            "qr_code.png",
            "image/png",
            &receipt,
            Some(&keyboard),
        )
        .await
        {
            msg_sent = true;
            if let Some(msg_id) = json
                .get("result")
                .and_then(|r| r.get("message_id"))
                .and_then(|v| v.as_i64())
            {
                let inv_id_clone = inv_id.clone();
                if let Some(pool) = fsm.pool() {
                    if let Ok(conn) = pool.get().await {
                        let _ = conn
                            .interact(move |c| {
                                db::invoices::update_invoice_telegram_context(
                                    c,
                                    &inv_id_clone,
                                    chat_id,
                                    msg_id,
                                )
                            })
                            .await;
                    }
                } else if let Ok(conn) = db::get_db_connection(&config.db_path) {
                    let _ = db::invoices::update_invoice_telegram_context(
                        &conn, &inv_id, chat_id, msg_id,
                    );
                }
            }
        }
    }

    // Fallback: If sending photo failed, send text message receipt with Phantom link
    if !msg_sent {
        let msg_payload = serde_json::json!({
            "chat_id": chat_id,
            "text": receipt,
            "parse_mode": "MarkdownV2",
            "reply_markup": keyboard
        });

        if let Ok(json) =
            send_telegram_request(client, &format!("{}/sendMessage", base_url), &msg_payload).await
        {
            msg_sent = true;
            if let Some(msg_id) = json
                .get("result")
                .and_then(|r| r.get("message_id"))
                .and_then(|v| v.as_i64())
            {
                let inv_id_clone = inv_id.clone();
                if let Some(pool) = fsm.pool() {
                    if let Ok(conn) = pool.get().await {
                        let _ = conn
                            .interact(move |c| {
                                db::invoices::update_invoice_telegram_context(
                                    c,
                                    &inv_id_clone,
                                    chat_id,
                                    msg_id,
                                )
                            })
                            .await;
                    }
                } else if let Ok(conn) = db::get_db_connection(&config.db_path) {
                    let _ = db::invoices::update_invoice_telegram_context(
                        &conn, &inv_id, chat_id, msg_id,
                    );
                }
            }
        }
    }

    if !msg_sent {
        tracing::error!(inv_id = %inv_id, "Failed to deliver invoice receipt to Telegram API");
        return Err(format!(
            "Failed to deliver invoice {} receipt to Telegram API",
            inv_id
        ));
    }

    Ok(())
}
