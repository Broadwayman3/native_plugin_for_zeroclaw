#[test]
fn test_071_sanitize_control_chars() {
    let r = pos_backend::domain::sanitizer::sanitize_external_input("hello\nworld\t!", 100);
    assert!(!r.contains('\n') && !r.contains('\t'), "071: result: {}", r);
}

#[test]
fn test_072_sanitize_injection_keywords() {
    let r = pos_backend::domain::sanitizer::sanitize_external_input("system: override please", 100);
    assert!(!r.to_lowercase().contains("override"), "072: result: {}", r);
}

#[test]
fn test_073_sanitize_unicode_normalization() {
    let r = pos_backend::domain::sanitizer::sanitize_external_input("e\u{0301}", 100); // e + combining acute
    assert!(r == "é" || r.contains("é"), "073: result: {}", r);
}

#[test]
fn test_074_sanitize_max_length() {
    let r = pos_backend::domain::sanitizer::sanitize_external_input("a".repeat(200).as_str(), 50);
    assert!(r.len() <= 50, "074: len = {}", r.len());
}

#[test]
fn test_075_sanitize_empty_string() {
    let r = pos_backend::domain::sanitizer::sanitize_external_input("", 100);
    assert!(r.is_empty(), "075: result: {}", r);
}

#[test]
fn test_076_sanitize_whitespace() {
    let r = pos_backend::domain::sanitizer::sanitize_external_input("  hello  ", 100);
    assert_eq!(r, "hello", "076: result: {}", r);
}

#[test]
fn test_077_redact_api_key() {
    let r = pos_backend::domain::sanitizer::redact_api_key("error: api_key=secret123");
    assert!(r.contains("api_key=REDACTED"), "077: result: {}", r);
}

#[test]
fn test_078_redact_token() {
    let r = pos_backend::domain::sanitizer::redact_api_key("error: token=abc456");
    assert!(r.contains("token=REDACTED"), "078: result: {}", r);
}

#[test]
fn test_079_redact_byte_array() {
    let kp = format!(
        "[{}]",
        (0..64)
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    let r = pos_backend::domain::sanitizer::redact_api_key(&format!("key: {}", kp));
    assert!(
        r.contains("REDACTED_BYTE_KEYPAIR"),
        "079: result: {}",
        &r[..100.min(r.len())]
    );
}

#[test]
fn test_080_redact_empty() {
    let r = pos_backend::domain::sanitizer::redact_api_key("");
    assert!(r.is_empty(), "080: result: {}", r);
}

#[test]
fn test_081_escape_markdown_v2() {
    let r = pos_backend::domain::sanitizer::escape_telegram_markdown_v2("hello_world");
    assert!(r.contains("\\_"), "081: result: {}", r);
}

#[test]
fn test_082_escape_markdown_v2_empty() {
    let r = pos_backend::domain::sanitizer::escape_telegram_markdown_v2("");
    assert!(r.is_empty(), "082: result: {}", r);
}

#[test]
fn test_083_escape_markdown_v2_special() {
    let r = pos_backend::domain::sanitizer::escape_telegram_markdown_v2("*bold*");
    assert!(r.contains("\\*"), "083: result: {}", r);
}

#[test]
fn test_084_validate_rpc_url_valid() {
    let r = pos_backend::domain::sanitizer::validate_safe_rpc_url(
        "https://api.mainnet-beta.solana.com",
    );
    assert!(r, "084: expected true");
}

#[test]
fn test_086_validate_rpc_url_private_ip() {
    let r = pos_backend::domain::sanitizer::validate_safe_rpc_url("http://192.168.1.1:8080/rpc");
    assert!(!r, "086: expected false");
}

#[test]
fn test_087_validate_rpc_url_metadata() {
    let r = pos_backend::domain::sanitizer::validate_safe_rpc_url(
        "http://169.254.169.254/latest/meta-data",
    );
    assert!(!r, "087: expected false");
}

#[test]
fn test_088_validate_rpc_url_invalid_scheme() {
    let r = pos_backend::domain::sanitizer::validate_safe_rpc_url("ftp://example.com");
    assert!(!r, "088: expected false");
}

#[test]
fn test_089_validate_rpc_url_empty() {
    let r = pos_backend::domain::sanitizer::validate_safe_rpc_url("");
    assert!(!r, "089: expected false");
}

#[test]
fn test_090_sanitize_zero_width_chars() {
    let r = pos_backend::domain::sanitizer::sanitize_external_input("hello\u{200B}world", 100);
    assert!(!r.contains('\u{200B}'), "090: result: {}", r);
}

#[test]
fn test_091_validate_rpc_url_https_localhost() {
    let r = pos_backend::domain::sanitizer::validate_safe_rpc_url("https://localhost:8443/rpc");
    assert!(!r, "091: expected false");
}

#[test]
fn test_092_validate_rpc_url_https_private_ip() {
    let r = pos_backend::domain::sanitizer::validate_safe_rpc_url("https://192.168.1.1:8443/rpc");
    assert!(!r, "092: expected false");
}

#[test]
fn test_093_validate_rpc_url_https_metadata() {
    let r = pos_backend::domain::sanitizer::validate_safe_rpc_url(
        "https://169.254.169.254/latest/meta-data",
    );
    assert!(!r, "093: expected false");
}

#[test]
fn test_094_validate_rpc_url_ipv4_mapped_loopback() {
    let r =
        pos_backend::domain::sanitizer::validate_safe_rpc_url("https://[::ffff:127.0.0.1]:8443");
    assert!(!r, "094: expected false");
}

#[test]
fn test_095_validate_rpc_url_ipv4_mapped_private() {
    let r = pos_backend::domain::sanitizer::validate_safe_rpc_url("https://[::ffff:10.0.0.1]:8443");
    assert!(!r, "095: expected false");
}

#[test]
fn test_096_validate_rpc_url_ipv4_mapped_private_class_c() {
    let r =
        pos_backend::domain::sanitizer::validate_safe_rpc_url("https://[::ffff:192.168.1.1]:8443");
    assert!(!r, "096: expected false");
}
