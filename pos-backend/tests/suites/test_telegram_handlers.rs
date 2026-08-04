use crate::{test_fail, test_pass};

pub fn run_suite() {
    println!("\n📦 Telegram Handler Tests (244-281)");
    test_244_sanitize_control_chars();
    test_245_sanitize_injection_keywords();
    test_246_sanitize_unicode_normalization();
    test_247_sanitize_max_length();
    test_248_sanitize_empty_string();
    test_249_sanitize_whitespace();
    test_250_sanitize_zero_width_chars();
    test_251_sanitize_nfc_to_nfkc();
    test_252_sanitize_prompt_injection_system();
    test_253_sanitize_prompt_injection_override();
    test_254_sanitize_prompt_injection_approve_refund();
    test_255_redact_api_key();
    test_256_redact_token();
    test_257_redact_byte_array();
    test_258_redact_empty();
    test_259_escape_markdown_v2();
    test_260_escape_markdown_v2_empty();
    test_261_escape_markdown_v2_special();
    test_262_validate_rpc_url_valid();
    test_263_validate_rpc_url_localhost();
    test_264_validate_rpc_url_private_ip();
    test_265_validate_rpc_url_metadata();
    test_266_validate_rpc_url_invalid_scheme();
    test_267_validate_rpc_url_empty();
    test_268_validate_rpc_url_loopback();
    test_269_validate_rpc_url_unspecified();
    test_270_validate_rpc_url_link_local();
    test_271_validate_rpc_url_broadcast();
    test_272_validate_rpc_url_reserved();
    test_273_validate_rpc_url_reserved_v4();
    test_274_validate_rpc_url_reserved_v6();
    test_275_validate_rpc_url_multicast_v6();
    test_276_validate_rpc_url_private_v4();
    test_277_validate_rpc_urlDocumentation_v6();
    test_278_validate_rpc_url_unique_local_v6();
    test_279_validate_rpc_url_tld_local();
    test_280_validate_rpc_url_tld_internal();
    test_281_is_payment_amount_valid();
}

fn test_244_sanitize_control_chars() {
    let r = pos_backend::domain::sanitizer::sanitize_external_input("hello\nworld\t!", 100);
    if !r.contains('\n') && !r.contains('\t') {
        test_pass("244: control chars removed");
    } else {
        test_fail("244", &format!("result: {}", r));
    }
}

fn test_245_sanitize_injection_keywords() {
    let r = pos_backend::domain::sanitizer::sanitize_external_input("system: override please", 100);
    if !r.to_lowercase().contains("override") {
        test_pass("245: injection keywords removed");
    } else {
        test_fail("245", &format!("result: {}", r));
    }
}

fn test_246_sanitize_unicode_normalization() {
    let r = pos_backend::domain::sanitizer::sanitize_external_input("e\u{0301}", 100);
    if r.contains("é") || r == "é" {
        test_pass("246: NFKC normalization works");
    } else {
        test_fail("246", &format!("result: {}", r));
    }
}

fn test_247_sanitize_max_length() {
    let r = pos_backend::domain::sanitizer::sanitize_external_input(&"a".repeat(200), 50);
    if r.len() <= 50 {
        test_pass("247: max length enforced");
    } else {
        test_fail("247", &format!("len: {}", r.len()));
    }
}

fn test_248_sanitize_empty_string() {
    let r = pos_backend::domain::sanitizer::sanitize_external_input("", 100);
    if r.is_empty() {
        test_pass("248: empty string returns empty");
    } else {
        test_fail("248", &format!("result: {}", r));
    }
}

fn test_249_sanitize_whitespace() {
    let r = pos_backend::domain::sanitizer::sanitize_external_input("  hello  ", 100);
    if r == "hello" {
        test_pass("249: whitespace trimmed");
    } else {
        test_fail("249", &format!("result: {}", r));
    }
}

fn test_250_sanitize_zero_width_chars() {
    let r = pos_backend::domain::sanitizer::sanitize_external_input("hello\u{200B}world", 100);
    if !r.contains('\u{200B}') {
        test_pass("250: zero-width space removed");
    } else {
        test_fail("250", &format!("result: {}", r));
    }
}

fn test_251_sanitize_nfc_to_nfkc() {
    // Full-width A (Ａ) should normalize to A
    let r = pos_backend::domain::sanitizer::sanitize_external_input("Ａ", 100);
    if r == "A" {
        test_pass("251: NFKC normalizes full-width chars");
    } else {
        test_fail("251", &format!("result: {}", r));
    }
}

fn test_252_sanitize_prompt_injection_system() {
    let r = pos_backend::domain::sanitizer::sanitize_external_input("system: admin mode", 100);
    if !r.contains("system") {
        test_pass("252: 'system:' keyword removed");
    } else {
        test_fail("252", &format!("result: {}", r));
    }
}

fn test_253_sanitize_prompt_injection_override() {
    let r = pos_backend::domain::sanitizer::sanitize_external_input("override settings", 100);
    if !r.contains("override") {
        test_pass("253: 'override' keyword removed");
    } else {
        test_fail("253", &format!("result: {}", r));
    }
}

fn test_254_sanitize_prompt_injection_approve_refund() {
    let r =
        pos_backend::domain::sanitizer::sanitize_external_input("approve_refund_immediately", 100);
    if !r.contains("approve_refund") {
        test_pass("254: 'approve_refund' keyword removed");
    } else {
        test_fail("254", &format!("result: {}", r));
    }
}

fn test_255_redact_api_key() {
    let r = pos_backend::domain::sanitizer::redact_api_key("error: api_key=secret123");
    if r.contains("api_key=REDACTED") {
        test_pass("255: API key redacted");
    } else {
        test_fail("255", &format!("result: {}", r));
    }
}

fn test_256_redact_token() {
    let r = pos_backend::domain::sanitizer::redact_api_key("error: token=abc456");
    if r.contains("token=REDACTED") {
        test_pass("256: token redacted");
    } else {
        test_fail("256", &format!("result: {}", r));
    }
}

fn test_257_redact_byte_array() {
    let kp = format!(
        "[{}]",
        (0..64)
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    let r = pos_backend::domain::sanitizer::redact_api_key(&format!("key: {}", kp));
    if r.contains("REDACTED_BYTE_KEYPAIR") {
        test_pass("257: byte array redacted");
    } else {
        test_fail("257", &format!("result: {}", &r[..100.min(r.len())]));
    }
}

fn test_258_redact_empty() {
    let r = pos_backend::domain::sanitizer::redact_api_key("");
    if r.is_empty() {
        test_pass("258: empty input returns empty");
    } else {
        test_fail("258", &format!("result: {}", r));
    }
}

fn test_259_escape_markdown_v2() {
    let r = pos_backend::domain::sanitizer::escape_telegram_markdown_v2("hello_world");
    if r.contains("\\_") {
        test_pass("259: underscore escaped");
    } else {
        test_fail("259", &format!("result: {}", r));
    }
}

fn test_260_escape_markdown_v2_empty() {
    let r = pos_backend::domain::sanitizer::escape_telegram_markdown_v2("");
    if r.is_empty() {
        test_pass("260: empty string returns empty");
    } else {
        test_fail("260", &format!("result: {}", r));
    }
}

fn test_261_escape_markdown_v2_special() {
    let r = pos_backend::domain::sanitizer::escape_telegram_markdown_v2("*bold*");
    if r.contains("\\*") {
        test_pass("261: asterisks escaped");
    } else {
        test_fail("261", &format!("result: {}", r));
    }
}

fn test_262_validate_rpc_url_valid() {
    let r = pos_backend::domain::sanitizer::validate_safe_rpc_url(
        "https://api.mainnet-beta.solana.com",
    );
    if r {
        test_pass("262: valid RPC URL accepted");
    } else {
        test_fail("262", "expected true");
    }
}

fn test_263_validate_rpc_url_localhost() {
    let r = pos_backend::domain::sanitizer::validate_safe_rpc_url("http://localhost:8080/rpc");
    if !r {
        test_pass("263: localhost rejected");
    } else {
        test_fail("263", "expected false");
    }
}

fn test_264_validate_rpc_url_private_ip() {
    let r = pos_backend::domain::sanitizer::validate_safe_rpc_url("http://192.168.1.1:8080/rpc");
    if !r {
        test_pass("264: private IP rejected");
    } else {
        test_fail("264", "expected false");
    }
}

fn test_265_validate_rpc_url_metadata() {
    let r = pos_backend::domain::sanitizer::validate_safe_rpc_url(
        "http://169.254.169.254/latest/meta-data",
    );
    if !r {
        test_pass("265: cloud metadata rejected");
    } else {
        test_fail("265", "expected false");
    }
}

fn test_266_validate_rpc_url_invalid_scheme() {
    let r = pos_backend::domain::sanitizer::validate_safe_rpc_url("ftp://example.com");
    if !r {
        test_pass("266: FTP scheme rejected");
    } else {
        test_fail("266", "expected false");
    }
}

fn test_267_validate_rpc_url_empty() {
    let r = pos_backend::domain::sanitizer::validate_safe_rpc_url("");
    if !r {
        test_pass("267: empty URL rejected");
    } else {
        test_fail("267", "expected false");
    }
}

fn test_268_validate_rpc_url_loopback() {
    let r = pos_backend::domain::sanitizer::validate_safe_rpc_url("http://127.0.0.1:8080/rpc");
    if !r {
        test_pass("268: loopback rejected");
    } else {
        test_fail("268", "expected false");
    }
}

fn test_269_validate_rpc_url_unspecified() {
    let r = pos_backend::domain::sanitizer::validate_safe_rpc_url("http://0.0.0.0:8080/rpc");
    if !r {
        test_pass("269: unspecified address rejected");
    } else {
        test_fail("269", "expected false");
    }
}

fn test_270_validate_rpc_url_link_local() {
    let r = pos_backend::domain::sanitizer::validate_safe_rpc_url("http://169.254.0.1:8080/rpc");
    if !r {
        test_pass("270: link-local rejected");
    } else {
        test_fail("270", "expected false");
    }
}

fn test_271_validate_rpc_url_broadcast() {
    let r =
        pos_backend::domain::sanitizer::validate_safe_rpc_url("http://255.255.255.255:8080/rpc");
    if !r {
        test_pass("271: broadcast rejected");
    } else {
        test_fail("271", "expected false");
    }
}

fn test_272_validate_rpc_url_reserved() {
    let r = pos_backend::domain::sanitizer::validate_safe_rpc_url("http://100.64.0.1:8080/rpc");
    if !r {
        test_pass("272: reserved (100.64.0.0/10) rejected");
    } else {
        test_fail("272", "expected false");
    }
}

fn test_273_validate_rpc_url_reserved_v4() {
    let r = pos_backend::domain::sanitizer::validate_safe_rpc_url("http://192.0.0.1:8080/rpc");
    if !r {
        test_pass("273: reserved (192.0.0.0/24) rejected");
    } else {
        test_fail("273", "expected false");
    }
}

fn test_274_validate_rpc_url_reserved_v6() {
    let r = pos_backend::domain::sanitizer::validate_safe_rpc_url("http://[::1]:8080/rpc");
    if !r {
        test_pass("274: IPv6 loopback rejected");
    } else {
        test_fail("274", "expected false");
    }
}

fn test_275_validate_rpc_url_multicast_v6() {
    let r = pos_backend::domain::sanitizer::validate_safe_rpc_url("http://[ff02::1]:8080/rpc");
    if !r {
        test_pass("275: IPv6 multicast rejected");
    } else {
        test_fail("275", "expected false");
    }
}

fn test_276_validate_rpc_url_private_v4() {
    let r = pos_backend::domain::sanitizer::validate_safe_rpc_url("http://10.0.0.1:8080/rpc");
    if !r {
        test_pass("276: private (10.0.0.0/8) rejected");
    } else {
        test_fail("276", "expected false");
    }
}

fn test_277_validate_rpc_urlDocumentation_v6() {
    let r = pos_backend::domain::sanitizer::validate_safe_rpc_url("http://[2001:db8::1]:8080/rpc");
    if !r {
        test_pass("277: documentation (2001:db8::/32) rejected");
    } else {
        test_fail("277", "expected false");
    }
}

fn test_278_validate_rpc_url_unique_local_v6() {
    let r = pos_backend::domain::sanitizer::validate_safe_rpc_url("http://[fc00::1]:8080/rpc");
    if !r {
        test_pass("278: unique-local (fc00::/7) rejected");
    } else {
        test_fail("278", "expected false");
    }
}

fn test_279_validate_rpc_url_tld_local() {
    let r = pos_backend::domain::sanitizer::validate_safe_rpc_url("http://myserver.local:8080/rpc");
    if !r {
        test_pass("279: .local TLD rejected");
    } else {
        test_fail("279", "expected false");
    }
}

fn test_280_validate_rpc_url_tld_internal() {
    let r =
        pos_backend::domain::sanitizer::validate_safe_rpc_url("http://myserver.internal:8080/rpc");
    if !r {
        test_pass("280: .internal TLD rejected");
    } else {
        test_fail("280", "expected false");
    }
}

fn test_281_is_payment_amount_valid() {
    let valid = pos_backend::domain::sanitizer::is_payment_amount_valid(9.9, 10.0, 1.0);
    let invalid = pos_backend::domain::sanitizer::is_payment_amount_valid(9.8, 10.0, 1.0);
    if valid && !invalid {
        test_pass("281: slippage tolerance works");
    } else {
        test_fail("281", &format!("valid={}, invalid={}", valid, invalid));
    }
}
