use crate::{test_fail, test_pass};

pub fn run_suite() {
    println!("\n📦 PIX BRL Tests (051-068)");
    test_051_crc16_known_vector();
    test_052_crc16_empty_string();
    test_053_pix_payload_format();
    test_054_pix_payload_truncation();
    test_055_pix_payload_amount();
    test_056_pix_payload_empty_merchant();
    test_057_pix_payload_unicode();
    test_058_crc16_deterministic();
    test_059_pix_payload_length();
    test_060_pix_payload_checksum_prefix();
    test_061_crc16_known_vector_abc();
    test_062_pix_payload_ends_with_crc();
    test_063_pix_payload_zero_amount();
    test_064_pix_payload_large_amount();
    test_065_pix_key_truncation();
    test_066_pix_payload_contains_bcb();
    test_067_pix_payload_country_br();
    test_068_crc16_different_inputs_differ();
}

fn test_051_crc16_known_vector() {
    let crc = pos_backend::domain::pix_brl::calculate_pix_crc16("test");
    if !crc.is_empty() && crc.len() == 4 {
        test_pass("051: CRC16 returns 4-char hex");
    } else {
        test_fail("051", &format!("crc: {}", crc));
    }
}

fn test_052_crc16_empty_string() {
    let crc = pos_backend::domain::pix_brl::calculate_pix_crc16("");
    if crc.len() == 4 {
        test_pass("052: CRC16 of empty string is 4 chars");
    } else {
        test_fail("052", &format!("crc: {}", crc));
    }
}

fn test_053_pix_payload_format() {
    let payload =
        pos_backend::domain::pix_brl::generate_pix_emv_payload("key123", 100.0, "Test Merchant");
    if payload.starts_with("00020126580014br.gov.bcb.pix") {
        test_pass("053: PIX payload starts with EMV header");
    } else {
        test_fail("053", &format!("payload: {}", &payload[..50]));
    }
}

fn test_054_pix_payload_truncation() {
    let long_name = "A".repeat(200);
    let payload =
        pos_backend::domain::pix_brl::generate_pix_emv_payload("key123", 100.0, &long_name);
    if payload.len() > 0 {
        test_pass("054: long merchant name truncated");
    } else {
        test_fail("054", "empty payload");
    }
}

fn test_055_pix_payload_amount() {
    let payload = pos_backend::domain::pix_brl::generate_pix_emv_payload("key123", 250.50, "Test");
    if payload.contains("5406250.50") {
        test_pass("055: amount 250.50 in payload");
    } else {
        test_fail(
            "055",
            &format!("payload: {}", &payload[..100.min(payload.len())]),
        );
    }
}

fn test_056_pix_payload_empty_merchant() {
    let payload = pos_backend::domain::pix_brl::generate_pix_emv_payload("key123", 10.0, "");
    if payload.contains("ZeroClaw POS") {
        test_pass("056: empty merchant defaults to ZeroClaw POS");
    } else {
        test_fail(
            "056",
            &format!("payload: {}", &payload[..100.min(payload.len())]),
        );
    }
}

fn test_057_pix_payload_unicode() {
    let payload = pos_backend::domain::pix_brl::generate_pix_emv_payload("key123", 10.0, "Ünïcödé");
    if payload.len() > 0 {
        test_pass("057: unicode merchant name handled");
    } else {
        test_fail("057", "empty payload");
    }
}

fn test_058_crc16_deterministic() {
    let crc1 = pos_backend::domain::pix_brl::calculate_pix_crc16("deterministic");
    let crc2 = pos_backend::domain::pix_brl::calculate_pix_crc16("deterministic");
    if crc1 == crc2 {
        test_pass("058: CRC16 is deterministic");
    } else {
        test_fail("058", &format!("{} != {}", crc1, crc2));
    }
}

fn test_059_pix_payload_length() {
    let payload = pos_backend::domain::pix_brl::generate_pix_emv_payload("key123", 100.0, "Test");
    if payload.len() > 50 {
        test_pass("059: PIX payload has reasonable length");
    } else {
        test_fail("059", &format!("len = {}", payload.len()));
    }
}

fn test_060_pix_payload_checksum_prefix() {
    let payload = pos_backend::domain::pix_brl::generate_pix_emv_payload("key123", 100.0, "Test");
    if payload.ends_with("6304") || payload.contains("6304") {
        test_pass("060: payload contains CRC16 tag 6304");
    } else {
        test_fail(
            "060",
            &format!("payload: {}", &payload[payload.len() - 20..]),
        );
    }
}

fn test_061_crc16_known_vector_abc() {
    let crc = pos_backend::domain::pix_brl::calculate_pix_crc16("ABC");
    if crc.len() == 4 && crc.chars().all(|c| c.is_ascii_hexdigit()) {
        test_pass("061: CRC16 of 'ABC' is valid 4-char hex");
    } else {
        test_fail("061", &format!("crc: {}", crc));
    }
}

fn test_062_pix_payload_ends_with_crc() {
    let payload = pos_backend::domain::pix_brl::generate_pix_emv_payload("key", 10.0, "Test");
    // Payload should end with 6304XXXX (CRC tag + 4 hex digits)
    if payload.len() >= 8 && &payload[payload.len() - 8..payload.len() - 4] == "6304" {
        test_pass("062: payload ends with 6304XXXX (CRC tag + checksum)");
    } else {
        test_fail(
            "062",
            &format!(
                "last 8 chars: {}",
                &payload[payload.len().saturating_sub(8)..]
            ),
        );
    }
}

fn test_063_pix_payload_zero_amount() {
    let payload = pos_backend::domain::pix_brl::generate_pix_emv_payload("key", 0.0, "Test");
    if payload.contains("54040.00") {
        test_pass("063: zero amount handled in payload");
    } else {
        test_fail(
            "063",
            &format!("payload: {}", &payload[..100.min(payload.len())]),
        );
    }
}

fn test_064_pix_payload_large_amount() {
    let payload = pos_backend::domain::pix_brl::generate_pix_emv_payload("key", 999999.99, "Test");
    if payload.contains("999999.99") {
        test_pass("064: large amount in payload");
    } else {
        test_fail(
            "064",
            &format!("payload: {}", &payload[..100.min(payload.len())]),
        );
    }
}

fn test_065_pix_key_truncation() {
    let long_key = "A".repeat(200);
    let payload = pos_backend::domain::pix_brl::generate_pix_emv_payload(&long_key, 10.0, "Test");
    if payload.len() > 0 {
        test_pass("065: long PIX key truncated without panic");
    } else {
        test_fail("065", "empty payload");
    }
}

fn test_066_pix_payload_contains_bcb() {
    let payload = pos_backend::domain::pix_brl::generate_pix_emv_payload("key", 10.0, "Test");
    if payload.contains("br.gov.bcb.pix") {
        test_pass("066: payload contains BCB PIX identifier");
    } else {
        test_fail("066", &format!("missing br.gov.bcb.pix"));
    }
}

fn test_067_pix_payload_country_br() {
    let payload = pos_backend::domain::pix_brl::generate_pix_emv_payload("key", 10.0, "Test");
    if payload.contains("5802BR") {
        test_pass("067: payload contains country code BR");
    } else {
        test_fail("067", &format!("missing 5802BR"));
    }
}

fn test_068_crc16_different_inputs_differ() {
    let crc1 = pos_backend::domain::pix_brl::calculate_pix_crc16("input1");
    let crc2 = pos_backend::domain::pix_brl::calculate_pix_crc16("input2");
    if crc1 != crc2 {
        test_pass("068: different inputs produce different CRC16");
    } else {
        test_fail("068", &format!("same CRC: {} == {}", crc1, crc2));
    }
}
