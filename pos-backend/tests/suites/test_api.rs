#[test]
fn test_131_formatters_pubkey_short() {
    let r = pos_backend::domain::formatters::format_pubkey_short(
        "8xAZmQ1111111111111111111111111111111111111",
    );
    assert!(
        r.starts_with("8xAZ") && r.ends_with("1111"),
        "131: pubkey formatted correctly, result: {}",
        r
    );
}

#[test]
fn test_132_formatters_solscan_url() {
    let r = pos_backend::domain::formatters::get_solscan_tx_url("sig123", Some("devnet"));
    assert!(
        r.contains("solscan.io/tx/sig123") && r.contains("cluster=devnet"),
        "132: Solscan URL generated, url: {}",
        r
    );
}

#[test]
fn test_133_formatters_base58_valid() {
    let r = pos_backend::domain::formatters::is_valid_base58(
        "8xAZmQ1111111111111111111111111111111111111",
    );
    assert!(r, "133: valid Base58 accepted");
}

#[test]
fn test_134_formatters_base58_invalid() {
    let r = pos_backend::domain::formatters::is_valid_base58("short");
    assert!(!r, "134: short key rejected");
}

#[test]
fn test_135_order_parser_with_currency() {
    let result = pos_backend::domain::order_parser::parse_pos_order_input(
        "2x Cappuccino 200 UAH",
        "Order",
        None,
    );
    assert!(
        result.has_price,
        "135: order with currency parsed, result: {:?}",
        result
    );
}

#[test]
fn test_136_order_parser_bare_number() {
    let result = pos_backend::domain::order_parser::parse_pos_order_input("150", "Order", None);
    assert_eq!(
        result.amount,
        Some(150.0),
        "136: bare number parsed as UAH, result: {:?}",
        result
    );
}

#[test]
fn test_137_order_parser_no_price() {
    let result =
        pos_backend::domain::order_parser::parse_pos_order_input("Coffee please", "Order", None);
    assert!(!result.has_price, "137: text without price detected");
}

#[test]
fn test_140_build_solana_pay_url_params() {
    let url = pos_core_logic::build_solana_pay_url(
        "merchant_key",
        10.0,
        "ref_key",
        None,
        "My Label",
        "My Message",
    );
    assert!(
        url.contains("label=My%20Label") && url.contains("message=My%20Message"),
        "140: custom label/message in URL, url: {}",
        url
    );
}

#[test]
fn test_141_order_parser_symbol_uah() {
    let r = pos_backend::domain::order_parser::parse_pos_order_input("150 ₴", "Order", None);
    assert!(
        r.has_price && r.currency.as_deref() == Some("UAH") && r.amount == Some(150.0),
        "141: ₴ symbol maps to UAH, result: {:?}",
        r
    );
}

#[test]
fn test_142_order_parser_symbol_usd() {
    let r = pos_backend::domain::order_parser::parse_pos_order_input("$50", "Order", None);
    assert!(
        r.has_price && r.currency.as_deref() == Some("USD") && r.amount == Some(50.0),
        "142: $ symbol maps to USD, result: {:?}",
        r
    );
}

#[test]
fn test_143_order_parser_symbol_eur() {
    let r = pos_backend::domain::order_parser::parse_pos_order_input("€25.50", "Order", None);
    assert!(
        r.has_price && r.currency.as_deref() == Some("EUR") && r.amount == Some(25.50),
        "143: € symbol maps to EUR, result: {:?}",
        r
    );
}

#[test]
fn test_144_order_parser_symbol_brl() {
    let r = pos_backend::domain::order_parser::parse_pos_order_input("R$100", "Order", None);
    assert!(
        r.has_price && r.currency.as_deref() == Some("BRL") && r.amount == Some(100.0),
        "144: R$ symbol maps to BRL, result: {:?}",
        r
    );
}

#[test]
fn test_145_order_parser_symbol_pln() {
    let r = pos_backend::domain::order_parser::parse_pos_order_input("ZŁ50", "Order", None);
    assert!(
        r.has_price && r.currency.as_deref() == Some("PLN") && r.amount == Some(50.0),
        "145: ZŁ symbol maps to PLN, result: {:?}",
        r
    );
}

#[test]
fn test_146_order_parser_draft_items_fallback() {
    let r = pos_backend::domain::order_parser::parse_pos_order_input(
        "150 UAH",
        "Default",
        Some("Draft Item"),
    );
    assert!(
        r.has_price && r.items == "Draft Item",
        "146: draft_items used when no items in text, items: {}",
        r.items
    );
}

#[test]
fn test_147_order_parser_negative_amount() {
    let r = pos_backend::domain::order_parser::parse_pos_order_input("-50 UAH", "Order", None);
    assert!(
        !r.has_price,
        "147: negative amount rejected, result: {:?}",
        r
    );
}

#[test]
fn test_148_order_parser_overflow_amount() {
    let r =
        pos_backend::domain::order_parser::parse_pos_order_input("9999999.99 USD", "Order", None);
    assert!(
        !r.has_price,
        "148: amount > 999999.99 rejected, result: {:?}",
        r
    );
}

#[test]
fn test_149_order_parser_items_with_currency() {
    let r = pos_backend::domain::order_parser::parse_pos_order_input(
        "2x Coffee 300 UAH",
        "Order",
        None,
    );
    assert!(
        r.has_price && r.items.contains("Coffee") && r.amount == Some(300.0),
        "149: items with currency parsed correctly, result: {:?}",
        r
    );
}

#[test]
fn test_150_order_parser_decimal_amount() {
    let r = pos_backend::domain::order_parser::parse_pos_order_input("35.75 BRL", "Order", None);
    assert!(
        r.has_price && r.amount == Some(35.75) && r.currency.as_deref() == Some("BRL"),
        "150: decimal amount parsed correctly, result: {:?}",
        r
    );
}

#[test]
fn test_151_order_parser_empty_text() {
    let r = pos_backend::domain::order_parser::parse_pos_order_input("", "Order", None);
    assert!(
        !r.has_price && r.items.is_empty(),
        "151: empty text returns no price, result: {:?}",
        r
    );
}

#[test]
fn test_152_order_parser_whitespace_only() {
    let r = pos_backend::domain::order_parser::parse_pos_order_input("   ", "Order", None);
    assert!(
        !r.has_price && r.items.is_empty(),
        "152: whitespace-only returns no price, result: {:?}",
        r
    );
}

#[test]
fn test_153_order_parser_mixed_language() {
    let r = pos_backend::domain::order_parser::parse_pos_order_input(
        "2x Капучіно 200 UAH",
        "Замовлення",
        None,
    );
    assert!(
        r.has_price && r.items.contains("Капучіно") && r.amount == Some(200.0),
        "153: mixed Ukrainian/English parsed, result: {:?}",
        r
    );
}

#[test]
fn test_154_order_parser_turkish_lira() {
    let r = pos_backend::domain::order_parser::parse_pos_order_input("500 TL", "Order", None);
    assert!(
        r.has_price && r.currency.as_deref() == Some("TL") && r.amount == Some(500.0),
        "154: TL currency parsed, result: {:?}",
        r
    );
}

#[test]
fn test_155_solscan_url_mainnet() {
    let r = pos_backend::domain::formatters::get_solscan_tx_url("sig123", None);
    assert!(
        r.contains("solscan.io/tx/sig123") && !r.contains("cluster="),
        "155: mainnet URL has no cluster param, url: {}",
        r
    );
}

#[test]
fn test_156_qr_image_url_format() {
    let url = pos_backend::domain::formatters::generate_solana_pay_qr_image_url(
        "solana:merchant?amount=10",
        300,
    );
    assert!(
        url.contains("api.qrserver.com") && url.contains("300x300"),
        "156: QR URL has correct format and size, url: {}",
        url
    );
}

#[test]
fn test_157_qr_image_url_encoded() {
    let url = pos_backend::domain::formatters::generate_solana_pay_qr_image_url(
        "solana:merchant?amount=10&label=My Shop",
        200,
    );
    assert!(
        url.contains("200x200") && url.contains("label%3DMy%20Shop"),
        "157: QR URL properly encodes special chars, url: {}",
        url
    );
}

#[test]
fn test_158_telegram_photo_payload_basic() {
    let payload = pos_backend::domain::formatters::generate_telegram_photo_payload(
        "12345",
        "https://example.com/qr.png",
        "Pay 10 USDC",
        None,
    );
    let has_chat = payload["chat_id"] == "12345";
    let has_photo = payload["photo"] == "https://example.com/qr.png";
    let has_caption = payload["caption"] == "Pay 10 USDC";
    let has_parse_mode = payload["parse_mode"] == "MarkdownV2";
    assert!(
        has_chat && has_photo && has_caption && has_parse_mode,
        "158: photo payload has all required fields, payload: {}",
        payload
    );
}

#[test]
fn test_159_telegram_photo_payload_with_markup() {
    let markup =
        serde_json::json!({"inline_keyboard": [[{"text": "Pay", "callback_data": "pay_123"}]]});
    let payload = pos_backend::domain::formatters::generate_telegram_photo_payload(
        "12345",
        "https://example.com/qr.png",
        "Pay",
        Some(&markup),
    );
    assert!(
        payload.get("reply_markup").is_some(),
        "159: photo payload includes reply_markup, payload: {}",
        payload
    );
}

#[test]
fn test_160_solscan_url_testnet() {
    let r = pos_backend::domain::formatters::get_solscan_tx_url("sig456", Some("testnet"));
    assert!(
        r.contains("cluster=testnet"),
        "160: testnet URL has cluster param, url: {}",
        r
    );
}

#[test]
fn test_161_pubkey_short_too_short() {
    let r = pos_backend::domain::formatters::format_pubkey_short("abc");
    assert_eq!(r, "abc", "161: short pubkey returned unchanged");
}

#[test]
fn test_162_base58_invalid_chars() {
    let r = pos_backend::domain::formatters::is_valid_base58(&"0OIl".repeat(10));
    assert!(!r, "162: Base58 with invalid chars rejected");
}

#[test]
fn test_163_order_parser_multi_items() {
    let r = pos_backend::domain::order_parser::parse_pos_order_input(
        "Latte 120 UAH + Croissant 80 UAH",
        "Order",
        None,
    );
    assert!(
        r.has_price && r.amount == Some(200.0) && r.currency.as_deref() == Some("UAH"),
        "163: multi items parsed & aggregated to 200 UAH, result: {:?}",
        r
    );
}
