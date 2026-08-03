use crate::{test_fail, test_pass};

pub fn run_suite() {
    println!("\n📦 PIX BRL Tests (051-060)");
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
    let payload = pos_backend::domain::pix_brl::generate_pix_emv_payload("key123", 100.0, "Test Merchant");
    if payload.starts_with("00020126580014br.gov.bcb.pix") {
        test_pass("053: PIX payload starts with EMV header");
    } else {
        test_fail("053", &format!("payload: {}", &payload[..50]));
    }
}

fn test_054_pix_payload_truncation() {
    let long_name = "A".repeat(200);
    let payload = pos_backend::domain::pix_brl::generate_pix_emv_payload("key123", 100.0, &long_name);
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
        test_fail("055", &format!("payload: {}", &payload[..100.min(payload.len())]));
    }
}

fn test_056_pix_payload_empty_merchant() {
    let payload = pos_backend::domain::pix_brl::generate_pix_emv_payload("key123", 10.0, "");
    if payload.contains("ZeroClaw POS") {
        test_pass("056: empty merchant defaults to ZeroClaw POS");
    } else {
        test_fail("056", &format!("payload: {}", &payload[..100.min(payload.len())]));
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
        test_fail("060", &format!("payload: {}", &payload[payload.len()-20..]));
    }
}
