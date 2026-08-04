use crate::{test_fail, test_pass};

pub fn run_suite() {
    println!("\n📦 Squads Multisig Tests V2 (190-213)");
    test_190_proposal_status_created();
    test_191_proposal_status_approved();
    test_192_proposal_status_rejected();
    test_193_proposal_invalid_status();
    test_194_instruction_data_zero_index();
    test_195_instruction_data_max_index();
    test_196_instruction_data_execution_type_range();
    test_197_squads_program_id_constant();
    test_198_proposal_json_instruction_data_hex();
    test_199_proposal_json_instruction_data_base64();
    test_200_instruction_data_byte_layout();
    test_201_proposal_index_unique();
    test_202_proposal_invoice_reference();
    test_203_proposal_amount_precision();
    test_204_hex_encode_bytes();
    test_205_base64_encode_bytes();
    test_206_instruction_data_le_byte_order();
    test_207_proposal_json_error_handling();
    test_208_instruction_data_execution_type_zero();
    test_209_instruction_data_execution_type_one();
    test_210_proposal_create_with_invoice();
    test_211_proposal_update_cascade();
    test_212_instruction_data_consistency();
    test_213_proposal_json_serialization();
}

fn setup_test_db() -> rusqlite::Connection {
    let conn = pos_backend::db::get_db_connection(":memory:").unwrap();
    pos_backend::db::schema::init_db(&conn, false).unwrap();
    conn
}

fn create_test_invoice(conn: &rusqlite::Connection, id: &str) {
    let _ = pos_backend::db::invoices::create_invoice(
        conn,
        &pos_backend::db::invoices::CreateInvoiceRequest {
            id: id.to_string(),
            reference_pubkey: format!("ref_{}", id),
            fiat_currency: Some("UAH".to_string()),
            fiat_amount: Some(100.0),
            usdc_amount: 2.41,
        },
    );
}

fn test_190_proposal_status_created() {
    let conn = setup_test_db();
    create_test_invoice(&conn, "INV-190");
    let idx =
        pos_backend::db::squads::create_proposal(&conn, "INV-190", "recipient1", 10.0).unwrap();
    let status: String = conn
        .query_row(
            "SELECT status FROM squads_proposals WHERE proposal_index = ?1",
            [idx],
            |row| row.get(0),
        )
        .unwrap();
    if status == "created" {
        test_pass("190: proposal status starts as 'created'");
    } else {
        test_fail("190", &format!("status: {}", status));
    }
}

fn test_191_proposal_status_approved() {
    let conn = setup_test_db();
    create_test_invoice(&conn, "INV-191");
    let idx =
        pos_backend::db::squads::create_proposal(&conn, "INV-191", "recipient1", 10.0).unwrap();
    pos_backend::db::squads::update_proposal_status(&conn, idx, "approved").unwrap();
    let status: String = conn
        .query_row(
            "SELECT status FROM squads_proposals WHERE proposal_index = ?1",
            [idx],
            |row| row.get(0),
        )
        .unwrap();
    if status == "approved" {
        test_pass("191: proposal status updated to 'approved'");
    } else {
        test_fail("191", &format!("status: {}", status));
    }
}

fn test_192_proposal_status_rejected() {
    let conn = setup_test_db();
    create_test_invoice(&conn, "INV-192");
    let idx =
        pos_backend::db::squads::create_proposal(&conn, "INV-192", "recipient1", 10.0).unwrap();
    pos_backend::db::squads::update_proposal_status(&conn, idx, "rejected").unwrap();
    let status: String = conn
        .query_row(
            "SELECT status FROM squads_proposals WHERE proposal_index = ?1",
            [idx],
            |row| row.get(0),
        )
        .unwrap();
    if status == "rejected" {
        test_pass("192: proposal status updated to 'rejected'");
    } else {
        test_fail("192", &format!("status: {}", status));
    }
}

fn test_193_proposal_invalid_status() {
    let conn = setup_test_db();
    create_test_invoice(&conn, "INV-193");
    let idx =
        pos_backend::db::squads::create_proposal(&conn, "INV-193", "recipient1", 10.0).unwrap();
    let result = pos_backend::db::squads::update_proposal_status(&conn, idx, "invalid_status");
    if result.is_ok() {
        test_pass("193: invalid status accepted (DB doesn't validate)");
    } else {
        test_fail("193", "should accept any status string");
    }
}

fn test_194_instruction_data_zero_index() {
    let data = pos_core_logic::build_squads_v4_instruction_data(0, 0, false);
    let index = u64::from_le_bytes(data[8..16].try_into().unwrap());
    if index == 0 {
        test_pass("194: zero index works");
    } else {
        test_fail("194", &format!("index: {}", index));
    }
}

fn test_195_instruction_data_max_index() {
    let data = pos_core_logic::build_squads_v4_instruction_data(u64::MAX, 0, false);
    let index = u64::from_le_bytes(data[8..16].try_into().unwrap());
    if index == u64::MAX {
        test_pass("195: max u64 index works");
    } else {
        test_fail("195", &format!("index: {}", index));
    }
}

fn test_196_instruction_data_execution_type_range() {
    for exec_type in 0..=255u8 {
        let data = pos_core_logic::build_squads_v4_instruction_data(0, exec_type, false);
        assert_eq!(data[16], exec_type);
    }
    test_pass("196: execution_type 0-255 all work");
}

fn test_197_squads_program_id_constant() {
    if pos_core_logic::SQUADS_V4_PROGRAM_ID == "SQDS4ep65T869rmQrGGsybZb26a6Uq3vig54W62pkhm" {
        test_pass("197: SQUADS_V4_PROGRAM_ID correct");
    } else {
        test_fail("197", "wrong program_id");
    }
}

fn test_198_proposal_json_instruction_data_hex() {
    let result = pos_core_logic::build_squads_v4_proposal(
        "multisig111",
        "vault111",
        "proposer111",
        "recipient111",
        10.0,
        5,
        "Refund",
    );
    let hex = result["instruction_data_hex"].as_str().unwrap();
    if hex.len() == 36 && hex.starts_with("847444ae") {
        test_pass("198: instruction_data_hex correct");
    } else {
        test_fail("198", &format!("hex: {}", hex));
    }
}

fn test_199_proposal_json_instruction_data_base64() {
    let result = pos_core_logic::build_squads_v4_proposal(
        "multisig111",
        "vault111",
        "proposer111",
        "recipient111",
        10.0,
        5,
        "Refund",
    );
    let b64 = result["instruction_data_base64"].as_str().unwrap();
    if !b64.is_empty()
        && b64
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
    {
        test_pass("199: instruction_data_base64 valid");
    } else {
        test_fail("199", &format!("base64: {}", b64));
    }
}

fn test_200_instruction_data_byte_layout() {
    let data = pos_core_logic::build_squads_v4_instruction_data(42, 1, true);
    assert_eq!(data.len(), 18);
    assert_eq!(&data[..8], &pos_core_logic::ANCHOR_DISCRIMINATOR);
    assert_eq!(u64::from_le_bytes(data[8..16].try_into().unwrap()), 42);
    assert_eq!(data[16], 1);
    assert_eq!(data[17], 1);
    test_pass("200: full byte layout correct");
}

fn test_201_proposal_index_unique() {
    let conn = setup_test_db();
    create_test_invoice(&conn, "INV-201a");
    create_test_invoice(&conn, "INV-201b");
    let idx1 = pos_backend::db::squads::create_proposal(&conn, "INV-201a", "r1", 10.0).unwrap();
    let idx2 = pos_backend::db::squads::create_proposal(&conn, "INV-201b", "r2", 20.0).unwrap();
    if idx1 != idx2 {
        test_pass("201: proposal indices are unique");
    } else {
        test_fail("201", "indices not unique");
    }
}

fn test_202_proposal_invoice_reference() {
    let conn = setup_test_db();
    create_test_invoice(&conn, "INV-202");
    let idx =
        pos_backend::db::squads::create_proposal(&conn, "INV-202", "recipient1", 10.0).unwrap();
    let invoice_id: String = conn
        .query_row(
            "SELECT invoice_id FROM squads_proposals WHERE proposal_index = ?1",
            [idx],
            |row| row.get(0),
        )
        .unwrap();
    if invoice_id == "INV-202" {
        test_pass("202: proposal references correct invoice");
    } else {
        test_fail("202", &format!("invoice_id: {}", invoice_id));
    }
}

fn test_203_proposal_amount_precision() {
    let conn = setup_test_db();
    create_test_invoice(&conn, "INV-203");
    let idx = pos_backend::db::squads::create_proposal(&conn, "INV-203", "recipient1", 10.123456)
        .unwrap();
    let amount: f64 = conn
        .query_row(
            "SELECT amount_usdc FROM squads_proposals WHERE proposal_index = ?1",
            [idx],
            |row| row.get(0),
        )
        .unwrap();
    if (amount - 10.123456).abs() < f64::EPSILON {
        test_pass("203: amount precision preserved");
    } else {
        test_fail("203", &format!("amount: {}", amount));
    }
}

fn test_204_hex_encode_bytes() {
    let encoded = pos_core_logic::hex_encode(&[0xFF, 0x00, 0xAB]);
    if encoded == "ff00ab" {
        test_pass("204: hex_encode byte values");
    } else {
        test_fail("204", &format!("encoded: {}", encoded));
    }
}

fn test_205_base64_encode_bytes() {
    let encoded = pos_core_logic::base64_encode(&[0x00, 0xFF, 0xAB]);
    assert!(!encoded.is_empty());
    test_pass("205: base64_encode bytes");
}

fn test_206_instruction_data_le_byte_order() {
    let data = pos_core_logic::build_squads_v4_instruction_data(0x0102030405060708, 0, false);
    let index = u64::from_le_bytes(data[8..16].try_into().unwrap());
    if index == 0x0102030405060708 {
        test_pass("206: LE byte order correct");
    } else {
        test_fail("206", &format!("index: {:x}", index));
    }
}

fn test_207_proposal_json_error_handling() {
    let result = pos_core_logic::build_squads_v4_proposal(
        "multisig111",
        "vault111",
        "proposer111",
        "recipient111",
        10.0,
        5,
        "Refund",
    );
    if result.get("program_id").is_some() {
        test_pass("207: valid inputs return valid JSON");
    } else {
        test_fail("207", "valid inputs should succeed");
    }
}

fn test_208_instruction_data_execution_type_zero() {
    let data = pos_core_logic::build_squads_v4_instruction_data(0, 0, false);
    if data[16] == 0 {
        test_pass("208: execution_type 0 works");
    } else {
        test_fail("208", &format!("type: {}", data[16]));
    }
}

fn test_209_instruction_data_execution_type_one() {
    let data = pos_core_logic::build_squads_v4_instruction_data(0, 1, false);
    if data[16] == 1 {
        test_pass("209: execution_type 1 works");
    } else {
        test_fail("209", &format!("type: {}", data[16]));
    }
}

fn test_210_proposal_create_with_invoice() {
    let conn = setup_test_db();
    create_test_invoice(&conn, "INV-200");
    let idx =
        pos_backend::db::squads::create_proposal(&conn, "INV-200", "recipient1", 2.41).unwrap();
    if idx > 0 {
        test_pass("210: proposal created with invoice reference");
    } else {
        test_fail("210", &format!("idx: {}", idx));
    }
}

fn test_211_proposal_update_cascade() {
    let conn = setup_test_db();
    create_test_invoice(&conn, "INV-211");
    let idx =
        pos_backend::db::squads::create_proposal(&conn, "INV-211", "recipient1", 10.0).unwrap();
    pos_backend::db::squads::update_proposal_status(&conn, idx, "approved").unwrap();
    pos_backend::db::squads::update_proposal_status(&conn, idx, "rejected").unwrap();
    let status: String = conn
        .query_row(
            "SELECT status FROM squads_proposals WHERE proposal_index = ?1",
            [idx],
            |row| row.get(0),
        )
        .unwrap();
    if status == "rejected" {
        test_pass("211: multiple status updates work");
    } else {
        test_fail("211", &format!("status: {}", status));
    }
}

fn test_212_instruction_data_consistency() {
    let data1 = pos_core_logic::build_squads_v4_instruction_data(42, 1, true);
    let data2 = pos_core_logic::build_squads_v4_instruction_data(42, 1, true);
    if data1 == data2 {
        test_pass("212: instruction data is deterministic");
    } else {
        test_fail("212", "outputs differ");
    }
}

fn test_213_proposal_json_serialization() {
    let result = pos_core_logic::build_squads_v4_proposal(
        "multisig111",
        "vault111",
        "proposer111",
        "recipient111",
        10.0,
        5,
        "Refund",
    );
    let json_str = serde_json::to_string(&result).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    if parsed.get("program_id").is_some() {
        test_pass("213: JSON serialization round-trip works");
    } else {
        test_fail("213", "round-trip failed");
    }
}
