use super::client::send_telegram_request;
use super::fsm::FsmStore;
use super::orders::handle_pos_order;
use super::state::{get_user_lang, set_user_lang};
use crate::config::AppConfig;
use crate::db;
use crate::domain::i18n::{get_localized_confirmation, get_main_reply_keyboard, t};
use crate::domain::keyboards::{
    build_answer_callback_payload, build_send_message_payload, generate_lang_inline_keyboard,
    is_btn_click,
};
use crate::domain::sanitizer::{escape_telegram_markdown_v2, strip_bot_mention};

/// Validates whether the user ID matches the configured store manager ID.
fn is_manager_authorized(config: &AppConfig, user_id: i64) -> Result<(), &'static str> {
    if user_id == 1087788105 {
        return Err("⛔ Anonymous group admin authorization is not supported. Please use your personal Telegram account.");
    }
    if config.manager_telegram_id == 0 {
        return Err("⛔ Forbidden. MANAGER_TELEGRAM_ID is not configured in server settings.");
    }
    if user_id <= 0 || user_id != config.manager_telegram_id {
        return Err("⛔ Forbidden. Action requires store manager authorization.");
    }
    Ok(())
}

/// Handles incoming user text messages.
#[allow(clippy::too_many_arguments)]
pub async fn handle_user_message(
    client: &reqwest::Client,
    base_url: &str,
    config: &AppConfig,
    fsm: &FsmStore,
    chat_id: i64,
    user_id: i64,
    raw_text: &str,
    reply_to_text: Option<&str>,
) {
    let normalized = strip_bot_mention(raw_text);
    let text = normalized.trim();
    let lang = get_user_lang(&config.db_path, chat_id);

    if text == "/start" {
        let welcome_text = t("welcome", Some(&lang), &[]);
        let lang_keyboard = generate_lang_inline_keyboard();
        let main_keyboard = get_main_reply_keyboard(
            &lang,
            config.quick_receipt_amount,
            &config.quick_receipt_currency,
        );

        let payload = build_send_message_payload(
            chat_id,
            &welcome_text,
            Some("MarkdownV2"),
            Some(&main_keyboard),
        );
        let _ = send_telegram_request(client, &format!("{}/sendMessage", base_url), &payload).await;

        let select_msg = t("select_lang", Some(&lang), &[]);
        let lang_payload = build_send_message_payload(
            chat_id,
            &select_msg,
            Some("MarkdownV2"),
            Some(&lang_keyboard),
        );
        let _ = send_telegram_request(client, &format!("{}/sendMessage", base_url), &lang_payload)
            .await;
        return;
    }

    if text == "/cancel" {
        fsm.clear(chat_id, user_id).await;
        let payload = serde_json::json!({
            "chat_id": chat_id,
            "text": "❌ Action cancelled. Current session reset.",
        });
        let _ = send_telegram_request(client, &format!("{}/sendMessage", base_url), &payload).await;
        return;
    }

    // Handle button clicks across all 13 languages
    if is_btn_click(text, "btn_lang")
        || text.contains("Idiomas")
        || text.contains("Languages")
        || text.contains("Мови")
        || text.contains("Sprachen")
    {
        let select_msg = t("select_lang", Some(&lang), &[]);
        let lang_keyboard = generate_lang_inline_keyboard();
        let payload = build_send_message_payload(
            chat_id,
            &select_msg,
            Some("MarkdownV2"),
            Some(&lang_keyboard),
        );
        let _ = send_telegram_request(client, &format!("{}/sendMessage", base_url), &payload).await;
        return;
    }

    if is_btn_click(text, "btn_custom")
        || text.contains("Digitar valor")
        || text.contains("custom amount")
        || text.contains("довільну суму")
        || text.contains("Betrag eingeben")
    {
        let help_text = t("custom_help", Some(&lang), &[]);
        let payload = build_send_message_payload(chat_id, &help_text, Some("MarkdownV2"), None);
        let _ = send_telegram_request(client, &format!("{}/sendMessage", base_url), &payload).await;
        return;
    }

    if is_btn_click(text, "btn_refund")
        || text.contains("Reembolso")
        || text.contains("Refund")
        || text.contains("Повернення")
        || text.contains("Rückerstattung")
    {
        if let Err(err_msg) = is_manager_authorized(config, user_id) {
            let payload = serde_json::json!({
                "chat_id": chat_id,
                "text": err_msg,
            });
            let _ =
                send_telegram_request(client, &format!("{}/sendMessage", base_url), &payload).await;
            return;
        }

        let help = "♻️ *Squads v4 Multisig Refund*\n─────────────────\nPlease enter the refund command:\n`/refund <invoice_id> <amount_usdc>`\n\nExample:\n`/refund INV-a6f49762 1.80`";
        let esc = escape_telegram_markdown_v2(help);
        let payload = build_send_message_payload(chat_id, &esc, Some("MarkdownV2"), None);
        let _ = send_telegram_request(client, &format!("{}/sendMessage", base_url), &payload).await;
        return;
    }

    if is_btn_click(text, "btn_sales")
        || text.contains("Resumo de vendas")
        || text.contains("Sales Summary")
        || text.contains("Звіт")
        || text.contains("Verkaufsübersicht")
    {
        if let Err(err_msg) = is_manager_authorized(config, user_id) {
            let payload = serde_json::json!({
                "chat_id": chat_id,
                "text": err_msg,
            });
            let _ =
                send_telegram_request(client, &format!("{}/sendMessage", base_url), &payload).await;
            return;
        }

        let summary_text = format!(
            "📊 *Sales Summary*\nMerchant: `{}`",
            &config.merchant_wallet_pubkey[..8]
        );
        let escaped = escape_telegram_markdown_v2(&summary_text);
        let payload = build_send_message_payload(chat_id, &escaped, Some("MarkdownV2"), None);
        let _ = send_telegram_request(client, &format!("{}/sendMessage", base_url), &payload).await;
        return;
    }

    // Refund command: /refund INV-101 20.0 (Strict Manager Auth)
    if text.starts_with("/refund") || (text.starts_with("INV-") && text.contains(" ")) {
        if let Err(err_msg) = is_manager_authorized(config, user_id) {
            let payload = serde_json::json!({
                "chat_id": chat_id,
                "text": err_msg,
            });
            let _ =
                send_telegram_request(client, &format!("{}/sendMessage", base_url), &payload).await;
            return;
        }

        let clean_text = if text.starts_with("/refund") {
            text.to_string()
        } else {
            format!("/refund {}", text)
        };

        let parts: Vec<&str> = clean_text.split_whitespace().collect();
        if parts.len() < 3 {
            let payload = serde_json::json!({
                "chat_id": chat_id,
                "text": "Usage: /refund <invoice_id> <amount_usdc>",
            });
            let _ =
                send_telegram_request(client, &format!("{}/sendMessage", base_url), &payload).await;
            return;
        }

        let inv_id = parts[1];
        let raw_amt = parts[2].to_lowercase();
        let amt: f64 = if raw_amt.ends_with("sol") {
            let sol_val: f64 = raw_amt.trim_end_matches("sol").parse().unwrap_or(0.0);
            sol_val * 180.0
        } else {
            raw_amt.parse().unwrap_or(0.0)
        };

        if amt > 50.0 {
            let payload = serde_json::json!({
                "chat_id": chat_id,
                "text": "❌ Error: Refund amount exceeds max allowable threshold ($50.00 USDC).",
            });
            let _ =
                send_telegram_request(client, &format!("{}/sendMessage", base_url), &payload).await;
            return;
        }

        let proposal_idx = if let Ok(conn) = db::get_db_connection(&config.db_path) {
            db::squads::create_proposal(&conn, inv_id, &config.merchant_wallet_pubkey, amt)
                .unwrap_or(1)
        } else {
            1
        };

        let esc_inv = escape_telegram_markdown_v2(inv_id);
        let esc_amt = escape_telegram_markdown_v2(&format!("{:.2}", amt));
        let esc_idx = escape_telegram_markdown_v2(&proposal_idx.to_string());

        let resp_msg = format!("✅ *Squads v4 Refund Proposal \\#{} Created*\n─────────────────\n• Invoice: `{}`\n• Amount: *{} USDC*\n• Status: *Pending Threshold Approval*", esc_idx, esc_inv, esc_amt);
        let payload = build_send_message_payload(chat_id, &resp_msg, Some("MarkdownV2"), None);
        let _ = send_telegram_request(client, &format!("{}/sendMessage", base_url), &payload).await;
        return;
    }

    let has_digit = text.chars().any(|c| c.is_ascii_digit());
    if !has_digit
        && (text.eq_ignore_ascii_case("hello")
            || text.eq_ignore_ascii_case("hi")
            || text.contains("привіт")
            || text.contains("ола"))
    {
        let help_text = t("custom_help", Some(&lang), &[]);
        let payload = build_send_message_payload(chat_id, &help_text, Some("MarkdownV2"), None);
        let _ = send_telegram_request(client, &format!("{}/sendMessage", base_url), &payload).await;
        return;
    }

    // Process POS Order Creation with FSM Context
    handle_pos_order(
        client,
        base_url,
        config,
        fsm,
        chat_id,
        user_id,
        &lang,
        text,
        reply_to_text,
    )
    .await;
}

/// Handles incoming Telegram callback queries with guaranteed answerCallbackQuery and atomic cancellation.
pub async fn handle_callback_query(
    client: &reqwest::Client,
    base_url: &str,
    config: &AppConfig,
    chat_id: i64,
    cb_id: &str,
    data: &str,
) {
    if data.starts_with("cancel_") {
        let inv_id = data
            .trim_start_matches("cancel_")
            .trim_start_matches("invoice_");

        let mut cancel_success = false;
        if let Ok(conn) = db::get_db_connection(&config.db_path) {
            if let Ok(count) = db::invoices::cancel_invoice(&conn, inv_id) {
                cancel_success = count > 0;
            }
        }

        let (toast_text, msg_text) = if cancel_success {
            (
                "Invoice Cancelled ❌",
                format!("❌ Invoice {} has been cancelled.", inv_id),
            )
        } else {
            (
                "Cancellation Failed ⚠️",
                format!(
                    "⚠️ Cannot cancel invoice {} (already paid or expired).",
                    inv_id
                ),
            )
        };

        let answer = build_answer_callback_payload(cb_id, toast_text, false);
        let _ = send_telegram_request(
            client,
            &format!("{}/answerCallbackQuery", base_url),
            &answer,
        )
        .await;

        let payload = serde_json::json!({
            "chat_id": chat_id,
            "text": msg_text,
        });
        let _ = send_telegram_request(client, &format!("{}/sendMessage", base_url), &payload).await;
    } else if data.starts_with("set_lang_") {
        let lang_code = data.trim_start_matches("set_lang_");
        set_user_lang(&config.db_path, chat_id, lang_code);

        let confirm_msg = get_localized_confirmation(lang_code);
        let escaped_confirm = escape_telegram_markdown_v2(&confirm_msg);
        let new_reply_keyboard = get_main_reply_keyboard(
            lang_code,
            config.quick_receipt_amount,
            &config.quick_receipt_currency,
        );

        let answer = build_answer_callback_payload(
            cb_id,
            &format!("Language set to {}", lang_code.to_uppercase()),
            false,
        );
        let _ = send_telegram_request(
            client,
            &format!("{}/answerCallbackQuery", base_url),
            &answer,
        )
        .await;

        let payload = build_send_message_payload(
            chat_id,
            &escaped_confirm,
            Some("MarkdownV2"),
            Some(&new_reply_keyboard),
        );
        let _ = send_telegram_request(client, &format!("{}/sendMessage", base_url), &payload).await;
    } else {
        let answer = build_answer_callback_payload(cb_id, "OK", false);
        let _ = send_telegram_request(
            client,
            &format!("{}/answerCallbackQuery", base_url),
            &answer,
        )
        .await;
    }
}
