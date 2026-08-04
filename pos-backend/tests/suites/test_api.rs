use crate::{test_fail, test_pass};

pub fn run_suite() {
    println!("\n📦 API Tests (131-162)");
    test_131_formatters_pubkey_short();
    test_132_formatters_solscan_url();
    test_133_formatters_base58_valid();
    test_134_formatters_base58_invalid();
    test_135_order_parser_with_currency();
    test_136_order_parser_bare_number();
    test_137_order_parser_no_price();
    test_138_squads_instruction_data();
    test_139_squads_discriminator();
    test_140_build_solana_pay_url_params();
    test_141_order_parser_symbol_uah();
    test_142_order_parser_symbol_usd();
    test_143_order_parser_symbol_eur();
    test_144_order_parser_symbol_brl();
    test_145_order_parser_symbol_pln();
    test_146_order_parser_draft_items_fallback();
    test_147_order_parser_negative_amount();
    test_148_order_parser_overflow_amount();
    test_149_order_parser_items_with_currency();
    test_150_order_parser_decimal_amount();
    test_151_order_parser_empty_text();
    test_152_order_parser_whitespace_only();
    test_153_order_parser_mixed_language();
    test_154_order_parser_turkish_lira();
    test_155_solscan_url_mainnet();
    test_156_qr_image_url_format();
    test_157_qr_image_url_encoded();
    test_158_telegram_photo_payload_basic();
    test_159_telegram_photo_payload_with_markup();
    test_160_solscan_url_testnet();
    test_161_pubkey_short_too_short();
    test_162_base58_invalid_chars();
}

fn test_131_formatters_pubkey_short() {
    let r = pos_backend::domain::formatters::format_pubkey_short(
        "8xAZmQ1111111111111111111111111111111111111",
    );
    if r.starts_with("8xAZ") && r.ends_with("1111") {
        test_pass("131: pubkey formatted correctly");
    } else {
        test_fail("131", &format!("result: {}", r));
    }
}

fn test_132_formatters_solscan_url() {
    let r = pos_backend::domain::formatters::get_solscan_tx_url("sig123", Some("devnet"));
    if r.contains("solscan.io/tx/sig123") && r.contains("cluster=devnet") {
        test_pass("132: Solscan URL generated");
    } else {
        test_fail("132", &format!("url: {}", r));
    }
}

fn test_133_formatters_base58_valid() {
    let r = pos_backend::domain::formatters::is_valid_base58(
        "8xAZmQ1111111111111111111111111111111111111",
    );
    if r {
        test_pass("133: valid Base58 accepted");
    } else {
        test_fail("133", "expected true");
    }
}

fn test_134_formatters_base58_invalid() {
    let r = pos_backend::domain::formatters::is_valid_base58("short");
    if !r {
        test_pass("134: short key rejected");
    } else {
        test_fail("134", "expected false");
    }
}

fn test_135_order_parser_with_currency() {
    let result = pos_backend::domain::order_parser::parse_pos_order_input(
        "2x Cappuccino 200 UAH",
        "Order",
        None,
    );
    if result.has_price {
        test_pass("135: order with currency parsed");
    } else {
        test_fail("135", &format!("result: {:?}", result));
    }
}

fn test_136_order_parser_bare_number() {
    let result = pos_backend::domain::order_parser::parse_pos_order_input("150", "Order", None);
    if result.amount == Some(150.0) {
        test_pass("136: bare number parsed as UAH");
    } else {
        test_fail("136", &format!("result: {:?}", result));
    }
}

fn test_137_order_parser_no_price() {
    let result =
        pos_backend::domain::order_parser::parse_pos_order_input("Coffee please", "Order", None);
    if !result.has_price {
        test_pass("137: text without price detected");
    } else {
        test_fail("137", "expected has_price=false");
    }
}

fn test_138_squads_instruction_data() {
    let data = pos_core_logic::build_squads_v4_instruction_data(42, 0, false);
    if data.len() == 18 && data[8..16] == 42u64.to_le_bytes() {
        test_pass("138: Squads instruction data correct");
    } else {
        test_fail("138", &format!("len = {}", data.len()));
    }
}

fn test_139_squads_discriminator() {
    if pos_core_logic::ANCHOR_DISCRIMINATOR == [132, 116, 68, 174, 216, 160, 198, 22] {
        test_pass("139: Anchor discriminator correct");
    } else {
        test_fail("139", "discriminator mismatch");
    }
}

fn test_140_build_solana_pay_url_params() {
    let url = pos_core_logic::build_solana_pay_url(
        "merchant_key",
        10.0,
        "ref_key",
        None,
        "My Label",
        "My Message",
    );
    if url.contains("label=My%20Label") && url.contains("message=My%20Message") {
        test_pass("140: custom label/message in URL");
    } else {
        test_fail("140", &format!("url: {}", url));
    }
}

fn test_141_order_parser_symbol_uah() {
    let r = pos_backend::domain::order_parser::parse_pos_order_input("150 ₴", "Order", None);
    if r.has_price && r.currency.as_deref() == Some("UAH") && r.amount == Some(150.0) {
        test_pass("141: ₴ symbol maps to UAH");
    } else {
        test_fail("141", &format!("result: {:?}", r));
    }
}

fn test_142_order_parser_symbol_usd() {
    let r = pos_backend::domain::order_parser::parse_pos_order_input("$50", "Order", None);
    if r.has_price && r.currency.as_deref() == Some("USD") && r.amount == Some(50.0) {
        test_pass("142: $ symbol maps to USD");
    } else {
        test_fail("142", &format!("result: {:?}", r));
    }
}

fn test_143_order_parser_symbol_eur() {
    let r = pos_backend::domain::order_parser::parse_pos_order_input("€25.50", "Order", None);
    if r.has_price && r.currency.as_deref() == Some("EUR") && r.amount == Some(25.50) {
        test_pass("143: € symbol maps to EUR");
    } else {
        test_fail("143", &format!("result: {:?}", r));
    }
}

fn test_144_order_parser_symbol_brl() {
    let r = pos_backend::domain::order_parser::parse_pos_order_input("R$100", "Order", None);
    if r.has_price && r.currency.as_deref() == Some("BRL") && r.amount == Some(100.0) {
        test_pass("144: R$ symbol maps to BRL");
    } else {
        test_fail("144", &format!("result: {:?}", r));
    }
}

fn test_145_order_parser_symbol_pln() {
    let r = pos_backend::domain::order_parser::parse_pos_order_input("ZŁ50", "Order", None);
    if r.has_price && r.currency.as_deref() == Some("PLN") && r.amount == Some(50.0) {
        test_pass("145: ZŁ symbol maps to PLN");
    } else {
        test_fail("145", &format!("result: {:?}", r));
    }
}

fn test_146_order_parser_draft_items_fallback() {
    let r = pos_backend::domain::order_parser::parse_pos_order_input(
        "150 UAH",
        "Default",
        Some("Draft Item"),
    );
    if r.has_price && r.items == "Draft Item" {
        test_pass("146: draft_items used when no items in text");
    } else {
        test_fail("146", &format!("items: {}", r.items));
    }
}

fn test_147_order_parser_negative_amount() {
    let r = pos_backend::domain::order_parser::parse_pos_order_input("-50 UAH", "Order", None);
    if !r.has_price {
        test_pass("147: negative amount rejected");
    } else {
        test_fail("147", &format!("result: {:?}", r));
    }
}

fn test_148_order_parser_overflow_amount() {
    let r =
        pos_backend::domain::order_parser::parse_pos_order_input("9999999.99 USD", "Order", None);
    if !r.has_price {
        test_pass("148: amount > 999999.99 rejected");
    } else {
        test_fail("148", &format!("result: {:?}", r));
    }
}

fn test_149_order_parser_items_with_currency() {
    let r = pos_backend::domain::order_parser::parse_pos_order_input(
        "2x Coffee 300 UAH",
        "Order",
        None,
    );
    if r.has_price && r.items.contains("Coffee") && r.amount == Some(300.0) {
        test_pass("149: items with currency parsed correctly");
    } else {
        test_fail("149", &format!("result: {:?}", r));
    }
}

fn test_150_order_parser_decimal_amount() {
    let r = pos_backend::domain::order_parser::parse_pos_order_input("35.75 BRL", "Order", None);
    if r.has_price && r.amount == Some(35.75) && r.currency.as_deref() == Some("BRL") {
        test_pass("150: decimal amount parsed correctly");
    } else {
        test_fail("150", &format!("result: {:?}", r));
    }
}

fn test_151_order_parser_empty_text() {
    let r = pos_backend::domain::order_parser::parse_pos_order_input("", "Order", None);
    if !r.has_price && r.items.is_empty() {
        test_pass("151: empty text returns no price");
    } else {
        test_fail("151", &format!("result: {:?}", r));
    }
}

fn test_152_order_parser_whitespace_only() {
    let r = pos_backend::domain::order_parser::parse_pos_order_input("   ", "Order", None);
    if !r.has_price && r.items.is_empty() {
        test_pass("152: whitespace-only returns no price");
    } else {
        test_fail("152", &format!("result: {:?}", r));
    }
}

fn test_153_order_parser_mixed_language() {
    let r = pos_backend::domain::order_parser::parse_pos_order_input(
        "2x Капучіно 200 UAH",
        "Замовлення",
        None,
    );
    if r.has_price && r.items.contains("Капучіно") && r.amount == Some(200.0) {
        test_pass("153: mixed Ukrainian/English parsed");
    } else {
        test_fail("153", &format!("result: {:?}", r));
    }
}

fn test_154_order_parser_turkish_lira() {
    let r = pos_backend::domain::order_parser::parse_pos_order_input("500 TL", "Order", None);
    if r.has_price && r.currency.as_deref() == Some("TL") && r.amount == Some(500.0) {
        test_pass("154: TL currency parsed");
    } else {
        test_fail("154", &format!("result: {:?}", r));
    }
}

fn test_155_solscan_url_mainnet() {
    let r = pos_backend::domain::formatters::get_solscan_tx_url("sig123", None);
    if r.contains("solscan.io/tx/sig123") && !r.contains("cluster=") {
        test_pass("155: mainnet URL has no cluster param");
    } else {
        test_fail("155", &format!("url: {}", r));
    }
}

fn test_156_qr_image_url_format() {
    let url = pos_backend::domain::formatters::generate_solana_pay_qr_image_url(
        "solana:merchant?amount=10",
        300,
    );
    if url.contains("api.qrserver.com") && url.contains("300x300") {
        test_pass("156: QR URL has correct format and size");
    } else {
        test_fail("156", &format!("url: {}", url));
    }
}

fn test_157_qr_image_url_encoded() {
    let url = pos_backend::domain::formatters::generate_solana_pay_qr_image_url(
        "solana:merchant?amount=10&label=My Shop",
        200,
    );
    if url.contains("200x200") && url.contains("label%3DMy%20Shop") {
        test_pass("157: QR URL properly encodes special chars");
    } else {
        test_fail("157", &format!("url: {}", url));
    }
}

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
    if has_chat && has_photo && has_caption && has_parse_mode {
        test_pass("158: photo payload has all required fields");
    } else {
        test_fail("158", &format!("payload: {}", payload));
    }
}

fn test_159_telegram_photo_payload_with_markup() {
    let markup =
        serde_json::json!({"inline_keyboard": [[{"text": "Pay", "callback_data": "pay_123"}]]});
    let payload = pos_backend::domain::formatters::generate_telegram_photo_payload(
        "12345",
        "https://example.com/qr.png",
        "Pay",
        Some(&markup),
    );
    if payload.get("reply_markup").is_some() {
        test_pass("159: photo payload includes reply_markup");
    } else {
        test_fail("159", &format!("payload: {}", payload));
    }
}

fn test_160_solscan_url_testnet() {
    let r = pos_backend::domain::formatters::get_solscan_tx_url("sig456", Some("testnet"));
    if r.contains("cluster=testnet") {
        test_pass("160: testnet URL has cluster param");
    } else {
        test_fail("160", &format!("url: {}", r));
    }
}

fn test_161_pubkey_short_too_short() {
    let r = pos_backend::domain::formatters::format_pubkey_short("abc");
    if r == "abc" {
        test_pass("161: short pubkey returned unchanged");
    } else {
        test_fail("161", &format!("result: {}", r));
    }
}

fn test_162_base58_invalid_chars() {
    let r = pos_backend::domain::formatters::is_valid_base58(&"0OIl".repeat(10));
    if !r {
        test_pass("162: Base58 with invalid chars rejected");
    } else {
        test_fail("162", "expected false for invalid Base58 chars");
    }
}
