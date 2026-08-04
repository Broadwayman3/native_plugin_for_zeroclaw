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

#[test]
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
    assert_eq!(
        status, "created",
        "190: proposal status starts as 'created'"
    );
}

#[test]
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
    assert_eq!(
        status, "approved",
        "191: proposal status updated to 'approved'"
    );
}

#[test]
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
    assert_eq!(
        status, "rejected",
        "192: proposal status updated to 'rejected'"
    );
}

#[test]
fn test_193_proposal_invalid_status() {
    let conn = setup_test_db();
    create_test_invoice(&conn, "INV-193");
    let idx =
        pos_backend::db::squads::create_proposal(&conn, "INV-193", "recipient1", 10.0).unwrap();
    let result = pos_backend::db::squads::update_proposal_status(&conn, idx, "invalid_status");
    assert!(
        result.is_ok(),
        "193: invalid status should be accepted (DB doesn't validate)"
    );
}

#[test]
fn test_194_instruction_data_zero_index() {
    let data = pos_core_logic::build_squads_v4_instruction_data(0, 0, false);
    let index = u64::from_le_bytes(data[8..16].try_into().unwrap());
    assert_eq!(index, 0, "194: zero index works");
}

#[test]
fn test_195_instruction_data_max_index() {
    let data = pos_core_logic::build_squads_v4_instruction_data(u64::MAX, 0, false);
    let index = u64::from_le_bytes(data[8..16].try_into().unwrap());
    assert_eq!(index, u64::MAX, "195: max u64 index works");
}

#[test]
fn test_196_instruction_data_execution_type_range() {
    for exec_type in 0..=255u8 {
        let data = pos_core_logic::build_squads_v4_instruction_data(0, exec_type, false);
        assert_eq!(data[16], exec_type);
    }
}

#[test]
fn test_197_squads_program_id_constant() {
    assert_eq!(
        pos_core_logic::SQUADS_V4_PROGRAM_ID,
        "SQDS4ep65T869rmQrGGsybZb26a6Uq3vig54W62pkhm",
        "197: SQUADS_V4_PROGRAM_ID correct"
    );
}

#[test]
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
    assert!(
        hex.len() == 36 && hex.starts_with("847444ae"),
        "198: instruction_data_hex correct, hex: {}",
        hex
    );
}

#[test]
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
    assert!(
        !b64.is_empty()
            && b64
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='),
        "199: instruction_data_base64 valid, base64: {}",
        b64
    );
}

#[test]
fn test_200_instruction_data_byte_layout() {
    let data = pos_core_logic::build_squads_v4_instruction_data(42, 1, true);
    assert_eq!(data.len(), 18);
    assert_eq!(&data[..8], &pos_core_logic::ANCHOR_DISCRIMINATOR);
    assert_eq!(u64::from_le_bytes(data[8..16].try_into().unwrap()), 42);
    assert_eq!(data[16], 1);
    assert_eq!(data[17], 1);
}

#[test]
fn test_201_proposal_index_unique() {
    let conn = setup_test_db();
    create_test_invoice(&conn, "INV-201a");
    create_test_invoice(&conn, "INV-201b");
    let idx1 = pos_backend::db::squads::create_proposal(&conn, "INV-201a", "r1", 10.0).unwrap();
    let idx2 = pos_backend::db::squads::create_proposal(&conn, "INV-201b", "r2", 20.0).unwrap();
    assert_ne!(idx1, idx2, "201: proposal indices are unique");
}

#[test]
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
    assert_eq!(
        invoice_id, "INV-202",
        "202: proposal references correct invoice"
    );
}

#[test]
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
    assert!(
        (amount - 10.123456).abs() < f64::EPSILON,
        "203: amount precision preserved, amount: {}",
        amount
    );
}

#[test]
fn test_204_hex_encode_bytes() {
    let encoded = pos_core_logic::hex_encode(&[0xFF, 0x00, 0xAB]);
    assert_eq!(encoded, "ff00ab", "204: hex_encode byte values");
}

#[test]
fn test_205_base64_encode_bytes() {
    let encoded = pos_core_logic::base64_encode(&[0x00, 0xFF, 0xAB]);
    assert!(!encoded.is_empty());
}

#[test]
fn test_206_instruction_data_le_byte_order() {
    let data = pos_core_logic::build_squads_v4_instruction_data(0x0102030405060708, 0, false);
    let index = u64::from_le_bytes(data[8..16].try_into().unwrap());
    assert_eq!(index, 0x0102030405060708, "206: LE byte order correct");
}

#[test]
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
    assert!(
        result.get("program_id").is_some(),
        "207: valid inputs should succeed"
    );
}

#[test]
fn test_208_instruction_data_execution_type_zero() {
    let data = pos_core_logic::build_squads_v4_instruction_data(0, 0, false);
    assert_eq!(data[16], 0, "208: execution_type 0 works");
}

#[test]
fn test_209_instruction_data_execution_type_one() {
    let data = pos_core_logic::build_squads_v4_instruction_data(0, 1, false);
    assert_eq!(data[16], 1, "209: execution_type 1 works");
}

#[test]
fn test_210_proposal_create_with_invoice() {
    let conn = setup_test_db();
    create_test_invoice(&conn, "INV-200");
    let idx =
        pos_backend::db::squads::create_proposal(&conn, "INV-200", "recipient1", 2.41).unwrap();
    assert!(
        idx > 0,
        "210: proposal created with invoice reference, idx: {}",
        idx
    );
}

#[test]
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
    assert_eq!(status, "rejected", "211: multiple status updates work");
}

#[test]
fn test_212_instruction_data_consistency() {
    let data1 = pos_core_logic::build_squads_v4_instruction_data(42, 1, true);
    let data2 = pos_core_logic::build_squads_v4_instruction_data(42, 1, true);
    assert_eq!(data1, data2, "212: instruction data is deterministic");
}

#[test]
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
    assert!(
        parsed.get("program_id").is_some(),
        "213: JSON serialization round-trip failed"
    );
}
