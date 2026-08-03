use crate::{test_fail, test_pass};

pub fn run_suite() {
    println!("\n📦 API Tests (131-140)");
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
}

fn test_131_formatters_pubkey_short() {
    let r = pos_backend::domain::formatters::format_pubkey_short("8xAZmQ1111111111111111111111111111111111111");
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
    let r = pos_backend::domain::formatters::is_valid_base58("8xAZmQ1111111111111111111111111111111111111");
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
    let result = pos_backend::domain::order_parser::parse_pos_order_input("2x Cappuccino 200 UAH", "Order", None);
    if result.get("has_price").and_then(|v| v.as_bool()) == Some(true) {
        test_pass("135: order with currency parsed");
    } else {
        test_fail("135", &format!("result: {:?}", result));
    }
}

fn test_136_order_parser_bare_number() {
    let result = pos_backend::domain::order_parser::parse_pos_order_input("150", "Order", None);
    if result.get("amount").and_then(|v| v.as_f64()) == Some(150.0) {
        test_pass("136: bare number parsed as UAH");
    } else {
        test_fail("136", &format!("result: {:?}", result));
    }
}

fn test_137_order_parser_no_price() {
    let result = pos_backend::domain::order_parser::parse_pos_order_input("Coffee please", "Order", None);
    if result.get("has_price").and_then(|v| v.as_bool()) == Some(false) {
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
        "merchant_key", 10.0, "ref_key", None, "My Label", "My Message",
    );
    if url.contains("label=My%20Label") && url.contains("message=My%20Message") {
        test_pass("140: custom label/message in URL");
    } else {
        test_fail("140", &format!("url: {}", url));
    }
}
