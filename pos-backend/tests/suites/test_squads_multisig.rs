use crate::{test_fail, test_pass};

pub fn run_suite() {
    println!("\n📦 Squads Multisig Tests (164-213)");
    test_164_anchor_discriminator_value();
    test_165_anchor_discriminator_length();
    test_166_instruction_data_length();
    test_167_instruction_data_borsh_encoding();
    test_168_instruction_data_proposal_index();
    test_169_instruction_data_execution_type();
    test_170_instruction_data_draft_false();
    test_171_instruction_data_draft_true();
    test_172_create_squads_proposal_json();
    test_173_update_squads_proposal_status();
    test_174_proposal_index_auto_increment();
    test_175_proposal_json_structure();
    test_176_proposal_json_program_id();
    test_177_proposal_json_anchor_discriminator();
    test_178_proposal_json_multisig();
    test_179_proposal_json_vault();
    test_180_proposal_json_proposer();
    test_181_proposal_json_recipient();
    test_182_proposal_json_amount();
    test_183_proposal_json_memo();
    test_184_hex_encode_basic();
    test_185_hex_encode_empty();
    test_186_hex_encode_all_zeros();
    test_187_base64_encode_basic();
    test_188_base64_encode_empty();
    test_189_base64_encode_padding();
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

fn test_164_anchor_discriminator_value() {
    let expected = [132, 116, 68, 174, 216, 160, 198, 22];
    if pos_core_logic::ANCHOR_DISCRIMINATOR == expected {
        test_pass("164: ANCHOR_DISCRIMINATOR value correct");
    } else {
        test_fail("164", "value mismatch");
    }
}

fn test_165_anchor_discriminator_length() {
    if pos_core_logic::ANCHOR_DISCRIMINATOR.len() == 8 {
        test_pass("165: ANCHOR_DISCRIMINATOR is 8 bytes");
    } else {
        test_fail("165", "wrong length");
    }
}

fn test_166_instruction_data_length() {
    let data = pos_core_logic::build_squads_v4_instruction_data(0, 0, false);
    if data.len() == 18 {
        test_pass("166: instruction data is 18 bytes");
    } else {
        test_fail("166", &format!("length: {}", data.len()));
    }
}

fn test_167_instruction_data_borsh_encoding() {
    let data = pos_core_logic::build_squads_v4_instruction_data(0, 0, false);
    assert_eq!(&data[..8], &pos_core_logic::ANCHOR_DISCRIMINATOR);
    assert_eq!(&data[8..16], &[0, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(data[16], 0);
    assert_eq!(data[17], 0);
    test_pass("167: Borsh encoding correct");
}

fn test_168_instruction_data_proposal_index() {
    let data = pos_core_logic::build_squads_v4_instruction_data(42, 0, false);
    let index = u64::from_le_bytes(data[8..16].try_into().unwrap());
    if index == 42 {
        test_pass("168: proposal_index encoded correctly");
    } else {
        test_fail("168", &format!("index: {}", index));
    }
}

fn test_169_instruction_data_execution_type() {
    let data = pos_core_logic::build_squads_v4_instruction_data(0, 5, false);
    if data[16] == 5 {
        test_pass("169: execution_type encoded correctly");
    } else {
        test_fail("169", &format!("type: {}", data[16]));
    }
}

fn test_170_instruction_data_draft_false() {
    let data = pos_core_logic::build_squads_v4_instruction_data(0, 0, false);
    if data[17] == 0 {
        test_pass("170: draft=false encoded as 0");
    } else {
        test_fail("170", &format!("draft: {}", data[17]));
    }
}

fn test_171_instruction_data_draft_true() {
    let data = pos_core_logic::build_squads_v4_instruction_data(0, 0, true);
    if data[17] == 1 {
        test_pass("171: draft=true encoded as 1");
    } else {
        test_fail("171", &format!("draft: {}", data[17]));
    }
}

fn test_172_create_squads_proposal_json() {
    let result = pos_core_logic::build_squads_v4_proposal(
        "multisig111", "vault111", "proposer111", "recipient111", 10.0, 5, "Refund",
    );
    if result.get("program_id").is_some() && result.get("proposal_index").is_some() {
        test_pass("172: create_squads_proposal returns valid JSON");
    } else {
        test_fail("172", &format!("result: {}", result));
    }
}

fn test_173_update_squads_proposal_status() {
    let conn = setup_test_db();
    create_test_invoice(&conn, "INV-173");
    let idx = pos_backend::db::squads::create_proposal(&conn, "INV-173", "recipient1", 2.41).unwrap();
    let updated = pos_backend::db::squads::update_proposal_status(&conn, idx, "approved").unwrap();
    if updated {
        test_pass("173: update_squads_proposal_status returns true");
    } else {
        test_fail("173", "should return true");
    }
}

fn test_174_proposal_index_auto_increment() {
    let conn = setup_test_db();
    create_test_invoice(&conn, "INV-1");
    create_test_invoice(&conn, "INV-2");
    create_test_invoice(&conn, "INV-3");
    let idx1 = pos_backend::db::squads::create_proposal(&conn, "INV-1", "recipient1", 10.0).unwrap();
    let idx2 = pos_backend::db::squads::create_proposal(&conn, "INV-2", "recipient2", 20.0).unwrap();
    let idx3 = pos_backend::db::squads::create_proposal(&conn, "INV-3", "recipient3", 30.0).unwrap();
    if idx1 < idx2 && idx2 < idx3 {
        test_pass("174: proposal_index auto-increments");
    } else {
        test_fail("174", &format!("{} < {} < {}", idx1, idx2, idx3));
    }
}

fn test_175_proposal_json_structure() {
    let result = pos_core_logic::build_squads_v4_proposal(
        "multisig111", "vault111", "proposer111", "recipient111", 10.0, 5, "Refund",
    );
    let required = ["program_id", "multisig_pubkey", "vault_pubkey", "proposer_pubkey",
                    "recipient_pubkey", "amount_usdc", "proposal_index", "memo",
                    "anchor_discriminator", "instruction_data_hex", "instruction_data_base64"];
    let mut missing = vec![];
    for key in &required {
        if result.get(*key).is_none() {
            missing.push(*key);
        }
    }
    if missing.is_empty() {
        test_pass("175: JSON has all required fields");
    } else {
        test_fail("175", &format!("missing: {:?}", missing));
    }
}

fn test_176_proposal_json_program_id() {
    let result = pos_core_logic::build_squads_v4_proposal(
        "multisig111", "vault111", "proposer111", "recipient111", 10.0, 5, "Refund",
    );
    if result["program_id"] == "SQDS4ep65T869rmQrGGsybZb26a6Uq3vig54W62pkhm" {
        test_pass("176: program_id correct");
    } else {
        test_fail("176", &format!("program_id: {}", result["program_id"]));
    }
}

fn test_177_proposal_json_anchor_discriminator() {
    let result = pos_core_logic::build_squads_v4_proposal(
        "multisig111", "vault111", "proposer111", "recipient111", 10.0, 5, "Refund",
    );
    if result["anchor_discriminator"] == "847444aed8a0c616" {
        test_pass("177: anchor discriminator hex correct");
    } else {
        test_fail("177", &format!("discriminator: {}", result["anchor_discriminator"]));
    }
}

fn test_178_proposal_json_multisig() {
    let result = pos_core_logic::build_squads_v4_proposal(
        "multisig111", "vault111", "proposer111", "recipient111", 10.0, 5, "Refund",
    );
    if result["multisig_pubkey"] == "multisig111" {
        test_pass("178: multisig_pubkey correct");
    } else {
        test_fail("178", &format!("multisig: {}", result["multisig_pubkey"]));
    }
}

fn test_179_proposal_json_vault() {
    let result = pos_core_logic::build_squads_v4_proposal(
        "multisig111", "vault111", "proposer111", "recipient111", 10.0, 5, "Refund",
    );
    if result["vault_pubkey"] == "vault111" {
        test_pass("179: vault_pubkey correct");
    } else {
        test_fail("179", &format!("vault: {}", result["vault_pubkey"]));
    }
}

fn test_180_proposal_json_proposer() {
    let result = pos_core_logic::build_squads_v4_proposal(
        "multisig111", "vault111", "proposer111", "recipient111", 10.0, 5, "Refund",
    );
    if result["proposer_pubkey"] == "proposer111" {
        test_pass("180: proposer_pubkey correct");
    } else {
        test_fail("180", &format!("proposer: {}", result["proposer_pubkey"]));
    }
}

fn test_181_proposal_json_recipient() {
    let result = pos_core_logic::build_squads_v4_proposal(
        "multisig111", "vault111", "proposer111", "recipient111", 10.0, 5, "Refund",
    );
    if result["recipient_pubkey"] == "recipient111" {
        test_pass("181: recipient_pubkey correct");
    } else {
        test_fail("181", &format!("recipient: {}", result["recipient_pubkey"]));
    }
}

fn test_182_proposal_json_amount() {
    let result = pos_core_logic::build_squads_v4_proposal(
        "multisig111", "vault111", "proposer111", "recipient111", 10.5, 5, "Refund",
    );
    if result["amount_usdc"] == 10.5 {
        test_pass("182: amount_usdc correct");
    } else {
        test_fail("182", &format!("amount: {}", result["amount_usdc"]));
    }
}

fn test_183_proposal_json_memo() {
    let result = pos_core_logic::build_squads_v4_proposal(
        "multisig111", "vault111", "proposer111", "recipient111", 10.0, 5, "Test Memo",
    );
    if result["memo"] == "Test Memo" {
        test_pass("183: memo correct");
    } else {
        test_fail("183", &format!("memo: {}", result["memo"]));
    }
}

fn test_184_hex_encode_basic() {
    let encoded = pos_core_logic::hex_encode(&[0x84, 0x74, 0x44, 0xae]);
    if encoded == "847444ae" {
        test_pass("184: hex_encode basic");
    } else {
        test_fail("184", &format!("encoded: {}", encoded));
    }
}

fn test_185_hex_encode_empty() {
    let encoded = pos_core_logic::hex_encode(&[]);
    if encoded.is_empty() {
        test_pass("185: hex_encode empty");
    } else {
        test_fail("185", &format!("encoded: {}", encoded));
    }
}

fn test_186_hex_encode_all_zeros() {
    let encoded = pos_core_logic::hex_encode(&[0x00, 0x00, 0x00]);
    if encoded == "000000" {
        test_pass("186: hex_encode all zeros");
    } else {
        test_fail("186", &format!("encoded: {}", encoded));
    }
}

fn test_187_base64_encode_basic() {
    let encoded = pos_core_logic::base64_encode(b"Hello, World!");
    if encoded == "SGVsbG8sIFdvcmxkIQ==" {
        test_pass("187: base64_encode basic");
    } else {
        test_fail("187", &format!("encoded: {}", encoded));
    }
}

fn test_188_base64_encode_empty() {
    let encoded = pos_core_logic::base64_encode(b"");
    if encoded.is_empty() {
        test_pass("188: base64_encode empty");
    } else {
        test_fail("188", &format!("encoded: {}", encoded));
    }
}

fn test_189_base64_encode_padding() {
    let encoded = pos_core_logic::base64_encode(b"M");
    if encoded == "TQ==" {
        test_pass("189: base64_encode padding");
    } else {
        test_fail("189", &format!("encoded: {}", encoded));
    }
}

fn test_190_proposal_status_created() {
    let conn = setup_test_db();
    create_test_invoice(&conn, "INV-190");
    let idx = pos_backend::db::squads::create_proposal(&conn, "INV-190", "recipient1", 10.0).unwrap();
    let status: String = conn.query_row("SELECT status FROM squads_proposals WHERE proposal_index = ?1", [idx], |row| row.get(0)).unwrap();
    if status == "created" {
        test_pass("190: proposal status starts as 'created'");
    } else {
        test_fail("190", &format!("status: {}", status));
    }
}

fn test_191_proposal_status_approved() {
    let conn = setup_test_db();
    create_test_invoice(&conn, "INV-191");
    let idx = pos_backend::db::squads::create_proposal(&conn, "INV-191", "recipient1", 10.0).unwrap();
    pos_backend::db::squads::update_proposal_status(&conn, idx, "approved").unwrap();
    let status: String = conn.query_row("SELECT status FROM squads_proposals WHERE proposal_index = ?1", [idx], |row| row.get(0)).unwrap();
    if status == "approved" {
        test_pass("191: proposal status updated to 'approved'");
    } else {
        test_fail("191", &format!("status: {}", status));
    }
}

fn test_192_proposal_status_rejected() {
    let conn = setup_test_db();
    create_test_invoice(&conn, "INV-192");
    let idx = pos_backend::db::squads::create_proposal(&conn, "INV-192", "recipient1", 10.0).unwrap();
    pos_backend::db::squads::update_proposal_status(&conn, idx, "rejected").unwrap();
    let status: String = conn.query_row("SELECT status FROM squads_proposals WHERE proposal_index = ?1", [idx], |row| row.get(0)).unwrap();
    if status == "rejected" {
        test_pass("192: proposal status updated to 'rejected'");
    } else {
        test_fail("192", &format!("status: {}", status));
    }
}

fn test_193_proposal_invalid_status() {
    let conn = setup_test_db();
    create_test_invoice(&conn, "INV-193");
    let idx = pos_backend::db::squads::create_proposal(&conn, "INV-193", "recipient1", 10.0).unwrap();
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
        "multisig111", "vault111", "proposer111", "recipient111", 10.0, 5, "Refund",
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
        "multisig111", "vault111", "proposer111", "recipient111", 10.0, 5, "Refund",
    );
    let b64 = result["instruction_data_base64"].as_str().unwrap();
    if !b64.is_empty() && b64.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=') {
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
    let idx = pos_backend::db::squads::create_proposal(&conn, "INV-202", "recipient1", 10.0).unwrap();
    let invoice_id: String = conn.query_row("SELECT invoice_id FROM squads_proposals WHERE proposal_index = ?1", [idx], |row| row.get(0)).unwrap();
    if invoice_id == "INV-202" {
        test_pass("202: proposal references correct invoice");
    } else {
        test_fail("202", &format!("invoice_id: {}", invoice_id));
    }
}

fn test_203_proposal_amount_precision() {
    let conn = setup_test_db();
    create_test_invoice(&conn, "INV-203");
    let idx = pos_backend::db::squads::create_proposal(&conn, "INV-203", "recipient1", 10.123456).unwrap();
    let amount: f64 = conn.query_row("SELECT amount_usdc FROM squads_proposals WHERE proposal_index = ?1", [idx], |row| row.get(0)).unwrap();
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
        "multisig111", "vault111", "proposer111", "recipient111", 10.0, 5, "Refund",
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
    let idx = pos_backend::db::squads::create_proposal(&conn, "INV-200", "recipient1", 2.41).unwrap();
    if idx > 0 {
        test_pass("210: proposal created with invoice reference");
    } else {
        test_fail("210", &format!("idx: {}", idx));
    }
}

fn test_211_proposal_update_cascade() {
    let conn = setup_test_db();
    create_test_invoice(&conn, "INV-211");
    let idx = pos_backend::db::squads::create_proposal(&conn, "INV-211", "recipient1", 10.0).unwrap();
    pos_backend::db::squads::update_proposal_status(&conn, idx, "approved").unwrap();
    pos_backend::db::squads::update_proposal_status(&conn, idx, "rejected").unwrap();
    let status: String = conn.query_row("SELECT status FROM squads_proposals WHERE proposal_index = ?1", [idx], |row| row.get(0)).unwrap();
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
        "multisig111", "vault111", "proposer111", "recipient111", 10.0, 5, "Refund",
    );
    let json_str = serde_json::to_string(&result).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    if parsed.get("program_id").is_some() {
        test_pass("213: JSON serialization round-trip works");
    } else {
        test_fail("213", "round-trip failed");
    }
}
