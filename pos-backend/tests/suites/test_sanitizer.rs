use crate::{test_fail, test_pass};

pub fn run_suite() {
    println!("\n📦 Sanitizer Tests (071-090)");
    test_071_sanitize_control_chars();
    test_072_sanitize_injection_keywords();
    test_073_sanitize_unicode_normalization();
    test_074_sanitize_max_length();
    test_075_sanitize_empty_string();
    test_076_sanitize_whitespace();
    test_077_redact_api_key();
    test_078_redact_token();
    test_079_redact_byte_array();
    test_080_redact_empty();
    test_081_escape_markdown_v2();
    test_082_escape_markdown_v2_empty();
    test_083_escape_markdown_v2_special();
    test_084_validate_rpc_url_valid();
    test_085_validate_rpc_url_localhost();
    test_086_validate_rpc_url_private_ip();
    test_087_validate_rpc_url_metadata();
    test_088_validate_rpc_url_invalid_scheme();
    test_089_validate_rpc_url_empty();
    test_090_sanitize_zero_width_chars();
}

fn test_071_sanitize_control_chars() {
    let r = pos_backend::domain::sanitizer::sanitize_external_input("hello\nworld\t!", 100);
    if !r.contains('\n') && !r.contains('\t') {
        test_pass("071: control chars removed");
    } else {
        test_fail("071", &format!("result: {}", r));
    }
}

fn test_072_sanitize_injection_keywords() {
    let r = pos_backend::domain::sanitizer::sanitize_external_input("system: override please", 100);
    if !r.to_lowercase().contains("override") {
        test_pass("072: injection keywords removed");
    } else {
        test_fail("072", &format!("result: {}", r));
    }
}

fn test_073_sanitize_unicode_normalization() {
    let r = pos_backend::domain::sanitizer::sanitize_external_input("e\u{0301}", 100); // e + combining acute
    if r == "é" || r.contains("é") {
        test_pass("073: NFKC normalization works");
    } else {
        test_fail("073", &format!("result: {}", r));
    }
}

fn test_074_sanitize_max_length() {
    let r = pos_backend::domain::sanitizer::sanitize_external_input("a".repeat(200).as_str(), 50);
    if r.len() <= 50 {
        test_pass("074: max length enforced");
    } else {
        test_fail("074", &format!("len = {}", r.len()));
    }
}

fn test_075_sanitize_empty_string() {
    let r = pos_backend::domain::sanitizer::sanitize_external_input("", 100);
    if r.is_empty() {
        test_pass("075: empty string returns empty");
    } else {
        test_fail("075", &format!("result: {}", r));
    }
}

fn test_076_sanitize_whitespace() {
    let r = pos_backend::domain::sanitizer::sanitize_external_input("  hello  ", 100);
    if r == "hello" {
        test_pass("076: whitespace trimmed");
    } else {
        test_fail("076", &format!("result: {}", r));
    }
}

fn test_077_redact_api_key() {
    let r = pos_backend::domain::sanitizer::redact_api_key("error: api_key=secret123");
    if r.contains("api_key=REDACTED") {
        test_pass("077: API key redacted");
    } else {
        test_fail("077", &format!("result: {}", r));
    }
}

fn test_078_redact_token() {
    let r = pos_backend::domain::sanitizer::redact_api_key("error: token=abc456");
    if r.contains("token=REDACTED") {
        test_pass("078: token redacted");
    } else {
        test_fail("078", &format!("result: {}", r));
    }
}

fn test_079_redact_byte_array() {
    let kp = format!(
        "[{}]",
        (0..64)
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    let r = pos_backend::domain::sanitizer::redact_api_key(&format!("key: {}", kp));
    if r.contains("REDACTED_BYTE_KEYPAIR") {
        test_pass("079: byte array keypair redacted");
    } else {
        test_fail("079", &format!("result: {}", &r[..100.min(r.len())]));
    }
}

fn test_080_redact_empty() {
    let r = pos_backend::domain::sanitizer::redact_api_key("");
    if r.is_empty() {
        test_pass("080: empty input returns empty");
    } else {
        test_fail("080", &format!("result: {}", r));
    }
}

fn test_081_escape_markdown_v2() {
    let r = pos_backend::domain::sanitizer::escape_telegram_markdown_v2("hello_world");
    if r.contains("\\_") {
        test_pass("081: underscore escaped");
    } else {
        test_fail("081", &format!("result: {}", r));
    }
}

fn test_082_escape_markdown_v2_empty() {
    let r = pos_backend::domain::sanitizer::escape_telegram_markdown_v2("");
    if r.is_empty() {
        test_pass("082: empty string returns empty");
    } else {
        test_fail("082", &format!("result: {}", r));
    }
}

fn test_083_escape_markdown_v2_special() {
    let r = pos_backend::domain::sanitizer::escape_telegram_markdown_v2("*bold*");
    if r.contains("\\*") {
        test_pass("083: asterisks escaped");
    } else {
        test_fail("083", &format!("result: {}", r));
    }
}

fn test_084_validate_rpc_url_valid() {
    let r = pos_backend::domain::sanitizer::validate_safe_rpc_url(
        "https://api.mainnet-beta.solana.com",
    );
    if r {
        test_pass("084: valid RPC URL accepted");
    } else {
        test_fail("084", "expected true");
    }
}

fn test_085_validate_rpc_url_localhost() {
    let r = pos_backend::domain::sanitizer::validate_safe_rpc_url("http://localhost:8080/rpc");
    if !r {
        test_pass("085: localhost rejected");
    } else {
        test_fail("085", "expected false");
    }
}

fn test_086_validate_rpc_url_private_ip() {
    let r = pos_backend::domain::sanitizer::validate_safe_rpc_url("http://192.168.1.1:8080/rpc");
    if !r {
        test_pass("086: private IP rejected");
    } else {
        test_fail("086", "expected false");
    }
}

fn test_087_validate_rpc_url_metadata() {
    let r = pos_backend::domain::sanitizer::validate_safe_rpc_url(
        "http://169.254.169.254/latest/meta-data",
    );
    if !r {
        test_pass("087: cloud metadata rejected");
    } else {
        test_fail("087", "expected false");
    }
}

fn test_088_validate_rpc_url_invalid_scheme() {
    let r = pos_backend::domain::sanitizer::validate_safe_rpc_url("ftp://example.com");
    if !r {
        test_pass("088: FTP scheme rejected");
    } else {
        test_fail("088", "expected false");
    }
}

fn test_089_validate_rpc_url_empty() {
    let r = pos_backend::domain::sanitizer::validate_safe_rpc_url("");
    if !r {
        test_pass("089: empty URL rejected");
    } else {
        test_fail("089", "expected false");
    }
}

fn test_090_sanitize_zero_width_chars() {
    let r = pos_backend::domain::sanitizer::sanitize_external_input("hello\u{200B}world", 100);
    if !r.contains('\u{200B}') {
        test_pass("090: zero-width space removed");
    } else {
        test_fail("090", &format!("result: {}", r));
    }
}
