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
fn test_164_anchor_discriminator_value() {
    let expected = [132, 116, 68, 174, 216, 160, 198, 22];
    assert_eq!(
        pos_core_logic::ANCHOR_DISCRIMINATOR,
        expected,
        "164: ANCHOR_DISCRIMINATOR value correct"
    );
}

#[test]
fn test_165_anchor_discriminator_length() {
    assert_eq!(
        pos_core_logic::ANCHOR_DISCRIMINATOR.len(),
        8,
        "165: ANCHOR_DISCRIMINATOR is 8 bytes"
    );
}

#[test]
fn test_166_instruction_data_length() {
    let data = pos_core_logic::build_squads_v4_instruction_data(0, 0, false);
    assert_eq!(
        data.len(),
        18,
        "166: instruction data is 18 bytes, length: {}",
        data.len()
    );
}

#[test]
fn test_167_instruction_data_borsh_encoding() {
    let data = pos_core_logic::build_squads_v4_instruction_data(0, 0, false);
    assert_eq!(&data[..8], &pos_core_logic::ANCHOR_DISCRIMINATOR);
    assert_eq!(&data[8..16], &[0, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(data[16], 0);
    assert_eq!(data[17], 0);
}

#[test]
fn test_168_instruction_data_proposal_index() {
    let data = pos_core_logic::build_squads_v4_instruction_data(42, 0, false);
    let index = u64::from_le_bytes(data[8..16].try_into().unwrap());
    assert_eq!(index, 42, "168: proposal_index encoded correctly");
}

#[test]
fn test_169_instruction_data_execution_type() {
    let data = pos_core_logic::build_squads_v4_instruction_data(0, 5, false);
    assert_eq!(data[16], 5, "169: execution_type encoded correctly");
}

#[test]
fn test_170_instruction_data_draft_false() {
    let data = pos_core_logic::build_squads_v4_instruction_data(0, 0, false);
    assert_eq!(data[17], 0, "170: draft=false encoded as 0");
}

#[test]
fn test_171_instruction_data_draft_true() {
    let data = pos_core_logic::build_squads_v4_instruction_data(0, 0, true);
    assert_eq!(data[17], 1, "171: draft=true encoded as 1");
}

#[test]
fn test_172_create_squads_proposal_json() {
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
        result.get("program_id").is_some() && result.get("proposal_index").is_some(),
        "172: create_squads_proposal returns valid JSON, result: {}",
        result
    );
}

#[test]
fn test_173_update_squads_proposal_status() {
    let conn = setup_test_db();
    create_test_invoice(&conn, "INV-173");
    let idx =
        pos_backend::db::squads::create_proposal(&conn, "INV-173", "recipient1", 2.41).unwrap();
    let updated = pos_backend::db::squads::update_proposal_status(&conn, idx, "approved").unwrap();
    assert!(
        updated,
        "173: update_squads_proposal_status should return true"
    );
}

#[test]
fn test_174_proposal_index_auto_increment() {
    let conn = setup_test_db();
    create_test_invoice(&conn, "INV-1");
    create_test_invoice(&conn, "INV-2");
    create_test_invoice(&conn, "INV-3");
    let idx1 =
        pos_backend::db::squads::create_proposal(&conn, "INV-1", "recipient1", 10.0).unwrap();
    let idx2 =
        pos_backend::db::squads::create_proposal(&conn, "INV-2", "recipient2", 20.0).unwrap();
    let idx3 =
        pos_backend::db::squads::create_proposal(&conn, "INV-3", "recipient3", 30.0).unwrap();
    assert!(
        idx1 < idx2 && idx2 < idx3,
        "174: proposal_index auto-increments, {} < {} < {}",
        idx1,
        idx2,
        idx3
    );
}

#[test]
fn test_175_proposal_json_structure() {
    let result = pos_core_logic::build_squads_v4_proposal(
        "multisig111",
        "vault111",
        "proposer111",
        "recipient111",
        10.0,
        5,
        "Refund",
    );
    let required = [
        "program_id",
        "multisig_pubkey",
        "vault_pubkey",
        "proposer_pubkey",
        "recipient_pubkey",
        "amount_usdc",
        "proposal_index",
        "memo",
        "anchor_discriminator",
        "instruction_data_hex",
        "instruction_data_base64",
    ];
    let mut missing = vec![];
    for key in &required {
        if result.get(*key).is_none() {
            missing.push(*key);
        }
    }
    assert!(
        missing.is_empty(),
        "175: JSON has all required fields, missing: {:?}",
        missing
    );
}

#[test]
fn test_176_proposal_json_program_id() {
    let result = pos_core_logic::build_squads_v4_proposal(
        "multisig111",
        "vault111",
        "proposer111",
        "recipient111",
        10.0,
        5,
        "Refund",
    );
    assert_eq!(
        result["program_id"], "SQDS4ep65T869rmQrGGsybZb26a6Uq3vig54W62pkhm",
        "176: program_id correct"
    );
}

#[test]
fn test_177_proposal_json_anchor_discriminator() {
    let result = pos_core_logic::build_squads_v4_proposal(
        "multisig111",
        "vault111",
        "proposer111",
        "recipient111",
        10.0,
        5,
        "Refund",
    );
    assert_eq!(
        result["anchor_discriminator"], "847444aed8a0c616",
        "177: anchor discriminator hex correct"
    );
}

#[test]
fn test_178_proposal_json_multisig() {
    let result = pos_core_logic::build_squads_v4_proposal(
        "multisig111",
        "vault111",
        "proposer111",
        "recipient111",
        10.0,
        5,
        "Refund",
    );
    assert_eq!(
        result["multisig_pubkey"], "multisig111",
        "178: multisig_pubkey correct"
    );
}

#[test]
fn test_179_proposal_json_vault() {
    let result = pos_core_logic::build_squads_v4_proposal(
        "multisig111",
        "vault111",
        "proposer111",
        "recipient111",
        10.0,
        5,
        "Refund",
    );
    assert_eq!(
        result["vault_pubkey"], "vault111",
        "179: vault_pubkey correct"
    );
}

#[test]
fn test_180_proposal_json_proposer() {
    let result = pos_core_logic::build_squads_v4_proposal(
        "multisig111",
        "vault111",
        "proposer111",
        "recipient111",
        10.0,
        5,
        "Refund",
    );
    assert_eq!(
        result["proposer_pubkey"], "proposer111",
        "180: proposer_pubkey correct"
    );
}

#[test]
fn test_181_proposal_json_recipient() {
    let result = pos_core_logic::build_squads_v4_proposal(
        "multisig111",
        "vault111",
        "proposer111",
        "recipient111",
        10.0,
        5,
        "Refund",
    );
    assert_eq!(
        result["recipient_pubkey"], "recipient111",
        "181: recipient_pubkey correct"
    );
}

#[test]
fn test_182_proposal_json_amount() {
    let result = pos_core_logic::build_squads_v4_proposal(
        "multisig111",
        "vault111",
        "proposer111",
        "recipient111",
        10.5,
        5,
        "Refund",
    );
    assert_eq!(result["amount_usdc"], 10.5, "182: amount_usdc correct");
}

#[test]
fn test_183_proposal_json_memo() {
    let result = pos_core_logic::build_squads_v4_proposal(
        "multisig111",
        "vault111",
        "proposer111",
        "recipient111",
        10.0,
        5,
        "Test Memo",
    );
    assert_eq!(result["memo"], "Test Memo", "183: memo correct");
}

#[test]
fn test_184_hex_encode_basic() {
    let encoded = pos_core_logic::hex_encode(&[0x84, 0x74, 0x44, 0xae]);
    assert_eq!(encoded, "847444ae", "184: hex_encode basic");
}

#[test]
fn test_185_hex_encode_empty() {
    let encoded = pos_core_logic::hex_encode(&[]);
    assert!(
        encoded.is_empty(),
        "185: hex_encode empty, encoded: {}",
        encoded
    );
}

#[test]
fn test_186_hex_encode_all_zeros() {
    let encoded = pos_core_logic::hex_encode(&[0x00, 0x00, 0x00]);
    assert_eq!(encoded, "000000", "186: hex_encode all zeros");
}

#[test]
fn test_187_base64_encode_basic() {
    let encoded = pos_core_logic::base64_encode(b"Hello, World!");
    assert_eq!(encoded, "SGVsbG8sIFdvcmxkIQ==", "187: base64_encode basic");
}

#[test]
fn test_188_base64_encode_empty() {
    let encoded = pos_core_logic::base64_encode(b"");
    assert!(
        encoded.is_empty(),
        "188: base64_encode empty, encoded: {}",
        encoded
    );
}

#[test]
fn test_189_base64_encode_padding() {
    let encoded = pos_core_logic::base64_encode(b"M");
    assert_eq!(encoded, "TQ==", "189: base64_encode padding");
}
