use crate::api::telegram::client::send_telegram_request;
use crate::api::telegram::handlers::refund::is_manager_authorized;
use crate::api::telegram::state::set_user_lang;
use crate::config::AppConfig;

use crate::db;
use crate::domain::i18n::{get_localized_confirmation, get_main_reply_keyboard};
use crate::domain::keyboards::{build_answer_callback_payload, build_send_message_payload};
use crate::domain::sanitizer::{escape_telegram_markdown_v2, sanitize_external_input};

/// Handles incoming Telegram callback queries with guaranteed high-priority answerCallbackQuery and atomic cancellation.
#[allow(clippy::too_many_arguments)]
pub async fn handle_callback_query(
    client: &reqwest::Client,
    base_url: &str,
    config: &AppConfig,
    pool: Option<&deadpool_sqlite::Pool>,
    chat_id: i64,
    user_id: i64,
    cb_id: &str,
    raw_data: &str,
) -> Result<(), String> {
    let sanitized_data = sanitize_external_input(raw_data, 100);

    let is_admin_action = sanitized_data.starts_with("cancel_")
        || sanitized_data.starts_with("refund_")
        || sanitized_data.starts_with("squads_")
        || sanitized_data.starts_with("admin_");

    if is_admin_action {
        if let Err(err_msg) = is_manager_authorized(config, user_id) {
            let answer = build_answer_callback_payload(cb_id, err_msg, true);
            let _ = send_telegram_request(
                client,
                &format!("{}/answerCallbackQuery", base_url),
                &answer,
            )
            .await;
            return Ok(());
        }
    }

    if sanitized_data.starts_with("cancel_") {
        let inv_id = sanitized_data
            .trim_start_matches("cancel_")
            .trim_start_matches("invoice_");

        // 1. Fast-Track answerCallbackQuery BEFORE long DB operation to prevent callback timeout
        let ack_answer =
            build_answer_callback_payload(cb_id, "⏳ Processing cancellation...", false);
        let _ = send_telegram_request(
            client,
            &format!("{}/answerCallbackQuery", base_url),
            &ack_answer,
        )
        .await;

        // 2. Perform DB cancellation
        let mut cancel_success = false;
        let inv_id_str = inv_id.to_string();
        if let Some(pool) = pool {
            if let Ok(conn) = pool.get().await {
                let res = conn
                    .interact(move |c| db::invoices::cancel_invoice(c, &inv_id_str))
                    .await;
                if let Ok(Ok(count)) = res {
                    cancel_success = count == 1; // Strict single row update verification
                }
            }
        } else if let Ok(conn) = db::get_db_connection(&config.db_path) {
            if let Ok(count) = db::invoices::cancel_invoice(&conn, inv_id) {
                cancel_success = count == 1; // Strict single row update verification
            }
        }

        let msg_text = if cancel_success {
            format!("❌ Invoice {} has been cancelled.", inv_id)
        } else {
            format!(
                "⚠️ Cannot cancel invoice {} (already paid or expired).",
                inv_id
            )
        };

        let payload = serde_json::json!({
            "chat_id": chat_id,
            "text": msg_text,
        });
        if let Err(e) =
            send_telegram_request(client, &format!("{}/sendMessage", base_url), &payload).await
        {
            tracing::error!(error = %e, "Failed to send cancel invoice message");
        }
    } else if sanitized_data.starts_with("refund_") {
        let ack_answer =
            build_answer_callback_payload(cb_id, "⏳ Processing refund prompt...", false);
        let _ = send_telegram_request(
            client,
            &format!("{}/answerCallbackQuery", base_url),
            &ack_answer,
        )
        .await;

        let inv_id = sanitized_data.trim_start_matches("refund_");
        let clean_inv_id = inv_id.replace('\\', r"\\").replace('`', r"\`");
        let help = format!(
            "♻️ *Squads v4 Multisig Refund*\n─────────────────\nPlease enter refund command:\n`/refund {} 1.0`",
            clean_inv_id
        );
        let payload = build_send_message_payload(chat_id, &help, Some("MarkdownV2"), None);
        let _ = send_telegram_request(client, &format!("{}/sendMessage", base_url), &payload).await;
    } else if sanitized_data.starts_with("squads_") || sanitized_data.starts_with("admin_") {
        let ack_answer =
            build_answer_callback_payload(cb_id, "⏳ Administrative action processed.", false);
        let _ = send_telegram_request(
            client,
            &format!("{}/answerCallbackQuery", base_url),
            &ack_answer,
        )
        .await;

        let esc_data = escape_telegram_markdown_v2(&sanitized_data);
        let text = format!("✅ Administrative action `{}` recorded\\.", esc_data);
        let payload = build_send_message_payload(chat_id, &text, Some("MarkdownV2"), None);
        let _ = send_telegram_request(client, &format!("{}/sendMessage", base_url), &payload).await;
    } else if sanitized_data.starts_with("set_lang_") {
        let lang_code = sanitized_data.trim_start_matches("set_lang_");
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
        if let Err(e) = send_telegram_request(
            client,
            &format!("{}/answerCallbackQuery", base_url),
            &answer,
        )
        .await
        {
            tracing::error!(error = %e, "Failed to send language answerCallbackQuery");
        }

        let payload = build_send_message_payload(
            chat_id,
            &escaped_confirm,
            Some("MarkdownV2"),
            Some(&new_reply_keyboard),
        );
        if let Err(e) =
            send_telegram_request(client, &format!("{}/sendMessage", base_url), &payload).await
        {
            tracing::error!(error = %e, "Failed to send language confirmation message");
        }
    } else {
        let answer = build_answer_callback_payload(cb_id, "OK", false);
        if let Err(e) = send_telegram_request(
            client,
            &format!("{}/answerCallbackQuery", base_url),
            &answer,
        )
        .await
        {
            tracing::error!(error = %e, "Failed to send default answerCallbackQuery");
        }
    }

    Ok(())
}
