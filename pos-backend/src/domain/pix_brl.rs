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
pub fn generate_pix_emv_payload(
    pix_key: &str,
    amount_brl: f64,
    merchant_name: &str,
) -> String {
    let amount_str = format!("{:.2}", amount_brl);
    let merchant_name = if merchant_name.is_empty() {
        "ZeroClaw POS"
    } else {
        merchant_name
    };

    // Truncate merchant name to 99 bytes
    let merchant_bytes = merchant_name.as_bytes();
    let merchant_truncated = if merchant_bytes.len() > 99 {
        std::str::from_utf8(&merchant_bytes[..99])
            .unwrap_or("ZeroClaw POS")
    } else {
        merchant_name
    };

    // Truncate pix key to 99 bytes
    let pix_key_bytes = pix_key.as_bytes();
    let pix_key_truncated = if pix_key_bytes.len() > 99 {
        std::str::from_utf8(&pix_key_bytes[..99]).unwrap_or("")
    } else {
        pix_key
    };

    let merchant_len = merchant_truncated.len();
    let pix_key_len = pix_key_truncated.len();

    let payload_base = format!(
        "00020126580014br.gov.bcb.pix\
         01{:02}{}\
         520400005303986\
         54{:02}{}\
         5802BR\
         59{:02}{}\
         6009SAO PAULO\
         62070503***",
        pix_key_len, pix_key_truncated,
        amount_str.len(), amount_str,
        merchant_len, merchant_truncated
    );

    let crc_hex = calculate_pix_crc16(&payload_base);
    format!("{}6304{}", payload_base, crc_hex)
}
