/// Brazil EMV QRCPS PIX QR Code Generator
/// Calculates EMV QRCPS CRC16 (CCITT-FALSE, polynomial 0x1021, init 0xFFFF).
pub fn calculate_pix_crc16(payload_without_crc: &str) -> String {
    let data_to_hash = format!("{}6304", payload_without_crc);
    let bytes = data_to_hash.as_bytes();

    let mut crc: u16 = 0xFFFF;
    for &byte in bytes {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            if crc & 0x8000 != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
            crc &= 0xFFFF;
        }
    }

    format!("{:04X}", crc)
}

/// Generates Brazil EMV QRCPS PIX payload with valid CRC16 checksum.
pub fn generate_pix_emv_payload(pix_key: &str, amount_brl: f64, merchant_name: &str) -> String {
    generate_pix_emv_payload_with_txid(pix_key, amount_brl, merchant_name, "101")
}

/// Generates Brazil EMV QRCPS PIX payload with dynamic invoice TxID reference and recalculates Tag 62 length.
pub fn generate_pix_emv_payload_with_txid(
    pix_key: &str,
    amount_brl: f64,
    merchant_name: &str,
    invoice_id: &str,
) -> String {
    let amount_str = format!("{:.2}", amount_brl);
    let merchant_name = if merchant_name.is_empty() {
        "ZeroClaw POS"
    } else {
        merchant_name
    };

    // Truncate merchant name to 99 chars (UTF-8 safe)
    let merchant_truncated: String = merchant_name.chars().take(99).collect();
    // Truncate pix key to 99 chars (UTF-8 safe)
    let pix_key_truncated: String = pix_key.chars().take(99).collect();

    let merchant_len = merchant_truncated.len();
    let pix_key_len = pix_key_truncated.len();

    // Construct Field 05 (TxID reference) inside Tag 62
    let raw_txid = if invoice_id.is_empty() {
        "101".to_string()
    } else {
        format!(
            "INV{}",
            invoice_id
                .trim_start_matches("INV-")
                .trim_start_matches('#')
        )
    };
    let txid_clean: String = raw_txid
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .take(25)
        .collect();

    let field05 = format!("05{:02}{}", txid_clean.len(), txid_clean);
    let tag62 = format!("62{:02}{}", field05.len(), field05);

    let payload_base = format!(
        "00020126580014br.gov.bcb.pix\
         01{:02}{}\
         520400005303986\
         54{:02}{}\
         5802BR\
         59{:02}{}\
         6009SAO PAULO\
         {}",
        pix_key_len,
        pix_key_truncated,
        amount_str.len(),
        amount_str,
        merchant_len,
        merchant_truncated,
        tag62
    );

    let crc_hex = calculate_pix_crc16(&payload_base);
    format!("{}6304{}", payload_base, crc_hex)
}
