use crate::domain::constants::BASE58_ALPHABET;

/// Truncates long Base58 pubkeys/signatures for clean display (e.g. 8xAZ...mQ11).
/// UTF-8 safe: uses character iterator instead of byte slicing.
pub fn format_pubkey_short(pubkey: &str) -> String {
    let char_count = pubkey.chars().count();
    if char_count < 12 {
        return pubkey.to_string();
    }
    let head: String = pubkey.chars().take(4).collect();
    let tail: String = pubkey.chars().skip(char_count - 4).collect();
    format!("{}...{}", head, tail)
}

/// Generates direct transaction link to Solscan Explorer.
pub fn get_solscan_tx_url(signature: &str, network: Option<&str>) -> String {
    let cluster_param = match network {
        Some(net @ ("devnet" | "testnet")) => format!("?cluster={}", net),
        _ => String::new(),
    };
    format!("https://solscan.io/tx/{}{}", signature, cluster_param)
}

/// Validates Solana Base58 public key format (32-44 chars, valid alphabet).
pub fn is_valid_base58(pubkey_str: &str) -> bool {
    if pubkey_str.len() < 32 || pubkey_str.len() > 44 {
        return false;
    }
    pubkey_str.chars().all(|c| BASE58_ALPHABET.contains(c))
}

/// Generates QR code image URL for Solana Pay.
/// Default size is 300x300 pixels.
pub fn generate_solana_pay_qr_image_url(solana_pay_url: &str, size: u32) -> String {
    let encoded = urlencoding::encode(solana_pay_url);
    format!(
        "https://api.qrserver.com/v1/create-qr-code/?size={}x{}&data={}",
        size, size, encoded
    )
}

/// Generates Telegram Bot API sendPhoto JSON payload.
pub fn generate_telegram_photo_payload(
    chat_id: &str,
    qr_image_url: &str,
    caption_text: &str,
    reply_markup: Option<&serde_json::Value>,
) -> serde_json::Value {
    let mut payload = serde_json::json!({
        "chat_id": chat_id,
        "photo": qr_image_url,
        "caption": caption_text,
        "parse_mode": "MarkdownV2"
    });

    if let Some(markup) = reply_markup {
        payload["reply_markup"] = markup.clone();
    }

    payload
}

/// Formats updated Telegram photo/message caption when an invoice transitions to PAID status.
/// All user-supplied and dynamic values are escaped via escape_telegram_markdown_v2().
pub fn format_paid_receipt_caption(
    invoice_id: &str,
    items: &str,
    usdc_amount: f64,
    signature: &str,
    network: Option<&str>,
) -> String {
    use crate::domain::sanitizer::escape_telegram_markdown_v2;

    let esc_id = escape_telegram_markdown_v2(invoice_id);
    let esc_items = escape_telegram_markdown_v2(items);
    let esc_usdc = escape_telegram_markdown_v2(&format!("{:.2}", usdc_amount));
    let esc_sig_short = escape_telegram_markdown_v2(&format_pubkey_short(signature));
    let solscan_url = get_solscan_tx_url(signature, network);

    format!(
        "✅ *ОПЛАЧЕНО* \\| Чек \\#{}\n• Позиції: {}\n• Сума: *{} USDC*\n• Транзакція: [{}]({})",
        esc_id, esc_items, esc_usdc, esc_sig_short, solscan_url
    )
}

/// Formats initial cashier receipt text supporting dual payment options (Solana Pay USDC + PIX BRL).
pub fn format_dual_payment_receipt_caption(
    invoice_id: &str,
    items: &str,
    usdc_amount: f64,
    brl_amount: f64,
    pix_emv_payload: &str,
) -> String {
    use crate::domain::sanitizer::escape_telegram_markdown_v2;

    let esc_id = escape_telegram_markdown_v2(invoice_id);
    let esc_items = escape_telegram_markdown_v2(items);
    let esc_usdc = escape_telegram_markdown_v2(&format!("{:.2}", usdc_amount));
    let esc_brl = escape_telegram_markdown_v2(&format!("{:.2}", brl_amount));
    let esc_pix = escape_telegram_markdown_v2(pix_emv_payload);

    format!(
        "🧾 *РАХУНОК \\#{}*\n• Позиції: {}\n\n💳 *Оплата Solana Pay \\(USDC\\):*\n• Сума: *{} USDC*\n\n🇧🇷 *Оплата PIX \\(BRL\\):*\n• Сума: *R$ {}*\n• Код PIX: `{}`",
        esc_id, esc_items, esc_usdc, esc_brl, esc_pix
    )
}
