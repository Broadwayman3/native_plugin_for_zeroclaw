use super::refund::{handle_refund_command, is_manager_authorized};
use crate::api::telegram::client::send_telegram_request;
use crate::api::telegram::fsm::FsmStore;
use crate::api::telegram::orders::handle_pos_order;
use crate::api::telegram::state::get_user_lang;
use crate::config::AppConfig;
use crate::domain::i18n::{get_main_reply_keyboard, t};
use crate::domain::keyboards::{
    build_send_message_payload, generate_lang_inline_keyboard, is_btn_click,
};
use crate::domain::sanitizer::{
    escape_telegram_markdown_v2, sanitize_external_input, strip_bot_mention,
};

/// Handles incoming user text messages.
#[allow(clippy::too_many_arguments)]
pub async fn handle_user_message(
    client: &reqwest::Client,
    base_url: &str,
    config: &AppConfig,
    fsm: &FsmStore,
    chat_id: i64,
    user_id: i64,
    chat_type: &str,
    raw_text: &str,
    reply_to_text: Option<&str>,
) -> Result<(), String> {
    let stripped = strip_bot_mention(raw_text);
    let sanitized = sanitize_external_input(&stripped, 500);
    let text = sanitized.trim();
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
        return Ok(());
    }

    if text == "/cancel" {
        fsm.clear(chat_id, user_id).await;
        let payload = serde_json::json!({
            "chat_id": chat_id,
            "text": "❌ Action cancelled. Current session reset.",
        });
        let _ = send_telegram_request(client, &format!("{}/sendMessage", base_url), &payload).await;
        return Ok(());
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
        return Ok(());
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
        return Ok(());
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
            return Ok(());
        }

        let help = "♻️ *Squads v4 Multisig Refund*\n─────────────────\nPlease enter the refund command:\n`/refund <invoice_id> <amount_usdc>`\n\nExample:\n`/refund INV-a6f49762 1.80`";
        let esc = escape_telegram_markdown_v2(help);
        let payload = build_send_message_payload(chat_id, &esc, Some("MarkdownV2"), None);
        let _ = send_telegram_request(client, &format!("{}/sendMessage", base_url), &payload).await;
        return Ok(());
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
            return Ok(());
        }

        let summary_text = format!(
            "📊 *Sales Summary*\nMerchant: `{}`",
            &config.merchant_wallet_pubkey[..8]
        );
        let escaped = escape_telegram_markdown_v2(&summary_text);
        let payload = build_send_message_payload(chat_id, &escaped, Some("MarkdownV2"), None);
        let _ = send_telegram_request(client, &format!("{}/sendMessage", base_url), &payload).await;
        return Ok(());
    }

    // Refund command routing
    if text.starts_with("/refund") || (text.starts_with("INV-") && text.contains(' ')) {
        return handle_refund_command(client, base_url, config, chat_id, user_id, &lang, text)
            .await;
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
        return Ok(());
    }

    // Process POS Order Creation with FSM Context
    handle_pos_order(
        client,
        base_url,
        config,
        fsm,
        chat_id,
        user_id,
        chat_type,
        &lang,
        text,
        reply_to_text,
    )
    .await
}
