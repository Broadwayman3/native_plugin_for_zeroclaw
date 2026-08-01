#!/usr/bin/env python3
"""
ZeroClaw Solana POS Agent - Brazil EMV QRCPS & PIX QR Generator Core Module
"""

def calculate_pix_crc16(payload_without_crc: str) -> str:
    """
    Calculates EMV QRCPS CRC16 (CCITT-FALSE, polynomial 0x1021, init 0xFFFF).
    Appends '6304' before computing checksum as per EMV Co / BR Code specification.
    Returns 4-character uppercase hexadecimal string (e.g. '1D2C').
    """
    data_to_hash = (payload_without_crc + "6304").encode('utf-8')
    crc = 0xFFFF
    for byte in data_to_hash:
        crc ^= (byte << 8)
        for _ in range(8):
            if crc & 0x8000:
                crc = ((crc << 1) ^ 0x1021) & 0xFFFF
            else:
                crc = (crc << 1) & 0xFFFF
    return f"{crc:04X}"

def generate_pix_emv_payload(pix_key: str, amount_brl: float, merchant_name: str = "ZeroClaw POS") -> str:
    """
    Generates Brazil-first EMV QRCPS PIX payload with valid CRC16 CCITT-FALSE checksum.
    Compatible with Brazilian banking apps (br.gov.bcb.pix).
    Guarantees Tag 59 byte length <= 99 bytes to prevent EMV parsing failures.
    """
    amount_str = f"{amount_brl:.2f}"
    merchant_name = merchant_name or "ZeroClaw POS"
    pix_key = pix_key or ""
    merchant_bytes = merchant_name.encode('utf-8')

    if len(merchant_bytes) > 99:
        merchant_bytes = merchant_bytes[:99]
        merchant_name = merchant_bytes.decode('utf-8', errors='ignore')
        merchant_bytes = merchant_name.encode('utf-8')
        
    merchant_len = len(merchant_bytes)
    pix_key_bytes = pix_key.encode('utf-8')
    pix_key_len = len(pix_key_bytes)
    payload_base = (
        "00020126580014br.gov.bcb.pix"
        f"01{pix_key_len:02d}{pix_key}"
        "520400005303986"
        f"54{len(amount_str):02d}{amount_str}"
        "5802BR"
        f"59{merchant_len:02d}{merchant_name}"
        "6009SAO PAULO"
        "62070503***"
    )
    crc_hex = calculate_pix_crc16(payload_base)
    return f"{payload_base}6304{crc_hex}"

