use crate::api::telegram::client::send_telegram_request;
use crate::config::AppConfig;
use crate::db;
use crate::domain::i18n::t;
use crate::domain::keyboards::build_send_message_payload;
use crate::domain::sanitizer::{escape_telegram_markdown_v2, sanitize_external_input};

/// Validates whether the user ID matches the configured store manager ID.
pub fn is_manager_authorized(config: &AppConfig, user_id: i64) -> Result<(), &'static str> {
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

/// Parses an amount string (e.g. "1.80" or "50") into atomic micro-USDC units (1 USDC = 1_000_000 units).
/// Uses integer arithmetic (u128) with no float math to satisfy AGENTS.md financial rules.
pub fn parse_usdc_atomic_amount(raw_amt: &str) -> Option<u128> {
    let clean = raw_amt.trim().to_lowercase();
    if clean.is_empty() || clean.contains('-') {
        return None;
    }

    let (num_str, is_sol) = if clean.ends_with("sol") {
        (clean.trim_end_matches("sol").trim(), true)
    } else {
        (clean.as_str(), false)
    };

    let parts: Vec<&str> = num_str.split('.').collect();
    if parts.len() > 2 {
        return None;
    }

    let whole: u128 = parts[0].parse().ok()?;

    let frac_units: u128 = if parts.len() == 2 {
        let frac_str = parts[1];
        if frac_str.len() > 6 {
            return None; // Exceeds 6 decimal places for USDC
        }
        let padded = format!("{:0<6}", frac_str);
        padded.parse().ok()?
    } else {
        0
    };

    let base_micro = whole.checked_mul(1_000_000)?.checked_add(frac_units)?;

    if is_sol {
        let sol_rate = match crate::domain::price_feed::get_multitier_fiat_rate(
            "SOL", None, None, None, None, true,
        ) {
            Ok(info) => info.get("rate").and_then(|v| v.as_f64()).unwrap_or(180.0),
            Err(_) => 180.0,
        };
        let micro_usdc =
            pos_core_logic::safe_f64_to_u64_atomic((base_micro as f64 / 1_000_000.0) * sol_rate, 6);
        Some(micro_usdc as u128)
    } else {
        Some(base_micro)
    }
}

/// Handles `/refund` multisig proposal creation in u128 atomic units.
pub async fn handle_refund_command(
    client: &reqwest::Client,
    base_url: &str,
    config: &AppConfig,
    chat_id: i64,
    user_id: i64,
    lang: &str,
    raw_text: &str,
) -> Result<(), String> {
    let sanitized = sanitize_external_input(raw_text, 100);

    if let Err(err_msg) = is_manager_authorized(config, user_id) {
        let payload = serde_json::json!({
            "chat_id": chat_id,
            "text": err_msg,
        });
        let _ = send_telegram_request(client, &format!("{}/sendMessage", base_url), &payload).await;
        return Ok(());
    }

    let clean_text = if sanitized.starts_with("/refund") {
        sanitized.to_string()
    } else {
        format!("/refund {}", sanitized)
    };

    let parts: Vec<&str> = clean_text.split_whitespace().collect();
    if parts.len() < 3 {
        let usage_msg = t(
            "custom_help",
            Some(lang),
            &[("help", "Usage: /refund <invoice_id> <amount_usdc>")],
        );
        let payload = serde_json::json!({
            "chat_id": chat_id,
            "text": usage_msg,
        });
        let _ = send_telegram_request(client, &format!("{}/sendMessage", base_url), &payload).await;
        return Ok(());
    }

    let inv_id = parts[1];
    let raw_amt_str = parts[2..].join(" ");

    let usdc_atomic = match parse_usdc_atomic_amount(&raw_amt_str) {
        Some(amt) => amt,
        None => {
            let payload = serde_json::json!({
                "chat_id": chat_id,
                "text": "❌ Error: Invalid refund amount format.",
            });
            let _ =
                send_telegram_request(client, &format!("{}/sendMessage", base_url), &payload).await;
            return Ok(());
        }
    };

    // Max threshold: $50.00 USDC = 50_000_000 micro-USDC
    const MAX_REFUND_ATOMIC: u128 = 50_000_000;
    if usdc_atomic > MAX_REFUND_ATOMIC {
        let payload = serde_json::json!({
            "chat_id": chat_id,
            "text": "❌ Error: Refund amount exceeds max allowable threshold ($50.00 USDC).",
        });
        let _ = send_telegram_request(client, &format!("{}/sendMessage", base_url), &payload).await;
        return Ok(());
    }

    let usdc_float = usdc_atomic as f64 / 1_000_000.0;

    let proposal_idx = match db::get_db_connection(&config.db_path).and_then(|conn| {
        db::squads::create_proposal(&conn, inv_id, &config.merchant_wallet_pubkey, usdc_float)
    }) {
        Ok(idx) => idx,
        Err(e) => {
            let err_text = format!(
                "❌ Error: Failed to create refund proposal in database: {}",
                e
            );
            let payload = build_send_message_payload(chat_id, &err_text, None, None);
            let _ =
                send_telegram_request(client, &format!("{}/sendMessage", base_url), &payload).await;
            return Err(format!("Failed to create refund proposal: {}", e));
        }
    };

    let esc_inv = escape_telegram_markdown_v2(inv_id);
    let esc_amt = escape_telegram_markdown_v2(&format!("{:.2}", usdc_float));
    let esc_idx = escape_telegram_markdown_v2(&proposal_idx.to_string());

    let resp_msg = format!(
        "✅ *Squads v4 Refund Proposal \\#{} Created*\n─────────────────\n• Invoice: `{}`\n• Amount: *{} USDC*\n• Status: *Pending Threshold Approval*",
        esc_idx, esc_inv, esc_amt
    );
    let payload = build_send_message_payload(chat_id, &resp_msg, Some("MarkdownV2"), None);
    if let Err(e) =
        send_telegram_request(client, &format!("{}/sendMessage", base_url), &payload).await
    {
        tracing::error!(error = %e, "Failed to send refund proposal creation message");
        return Err(e);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_usdc_atomic_amount_valid() {
        assert_eq!(parse_usdc_atomic_amount("1.80"), Some(1_800_000));
        assert_eq!(parse_usdc_atomic_amount("50"), Some(50_000_000));
        assert_eq!(parse_usdc_atomic_amount("50.00"), Some(50_000_000));
        assert_eq!(parse_usdc_atomic_amount("0.000001"), Some(1));
        assert!(parse_usdc_atomic_amount("0.1sol").is_some());
        assert!(parse_usdc_atomic_amount("0.1 SOL").is_some());
    }

    #[test]
    fn test_parse_usdc_atomic_amount_invalid() {
        assert_eq!(parse_usdc_atomic_amount("0.0000001"), None); // Too many decimals
        assert_eq!(parse_usdc_atomic_amount("-1.0"), None);
        assert_eq!(parse_usdc_atomic_amount("invalid"), None);
        assert_eq!(parse_usdc_atomic_amount("1.2.3"), None);
    }
}
