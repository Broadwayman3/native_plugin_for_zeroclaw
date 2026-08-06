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

    if sanitized_data.starts_with("cancel_") {
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

        let inv_id = sanitized_data
            .trim_start_matches("cancel_")
            .trim_start_matches("invoice_");

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
        if let Err(e) = send_telegram_request(
            client,
            &format!("{}/answerCallbackQuery", base_url),
            &answer,
        )
        .await
        {
            tracing::error!(error = %e, "Failed to send answerCallbackQuery");
        }

        let payload = serde_json::json!({
            "chat_id": chat_id,
            "text": msg_text,
        });
        if let Err(e) =
            send_telegram_request(client, &format!("{}/sendMessage", base_url), &payload).await
        {
            tracing::error!(error = %e, "Failed to send cancel invoice message");
        }
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
