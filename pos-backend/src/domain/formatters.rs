use crate::domain::constants::BASE58_ALPHABET;

/// Truncates long Base58 pubkeys/signatures for clean display (e.g. 8xAZ...mQ11).
pub fn format_pubkey_short(pubkey: &str) -> String {
    if pubkey.len() < 12 {
        return pubkey.to_string();
    }
    format!("{}...{}", &pubkey[..4], &pubkey[pubkey.len() - 4..])
}

/// Generates direct transaction link to Solscan Explorer.
/// Auto-detects network from SOLANA_RPC_URL env var if not specified.
pub fn get_solscan_tx_url(signature: &str, network: Option<&str>) -> String {
    let net = network.unwrap_or_else(|| {
        let rpc_url = std::env::var("SOLANA_RPC_URL").unwrap_or_default();
        if rpc_url.contains("mainnet") || rpc_url.contains("helius") {
            "mainnet"
        } else {
            "devnet"
        }
    });
    let cluster_param = if net == "devnet" || net == "testnet" {
        format!("?cluster={}", net)
    } else {
        String::new()
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
