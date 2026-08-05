#[test]
fn test_270_validate_rpc_url_link_local() {
    assert!(
        !pos_backend::domain::sanitizer::validate_safe_rpc_url("http://169.254.0.1:8080/rpc"),
        "270: link-local not blocked"
    );
}

#[test]
fn test_271_validate_rpc_url_broadcast() {
    assert!(
        !pos_backend::domain::sanitizer::validate_safe_rpc_url("http://255.255.255.255:8080/rpc"),
        "271: broadcast not blocked"
    );
}

#[test]
fn test_272_validate_rpc_url_reserved() {
    assert!(
        !pos_backend::domain::sanitizer::validate_safe_rpc_url("http://100.64.0.1:8080/rpc"),
        "272: reserved not blocked"
    );
}

#[test]
fn test_273_validate_rpc_url_reserved_v4() {
    assert!(
        !pos_backend::domain::sanitizer::validate_safe_rpc_url("http://192.0.0.1:8080/rpc"),
        "273: reserved v4 not blocked"
    );
}

#[test]
fn test_275_validate_rpc_url_multicast_v6() {
    assert!(
        !pos_backend::domain::sanitizer::validate_safe_rpc_url("http://[ff02::1]:8080/rpc"),
        "275: IPv6 multicast not blocked"
    );
}

#[test]
fn test_276_validate_rpc_url_private_v4() {
    assert!(
        !pos_backend::domain::sanitizer::validate_safe_rpc_url("http://10.0.0.1:8080/rpc"),
        "276: private v4 not blocked"
    );
}

#[test]
fn test_279_validate_rpc_url_tld_local() {
    assert!(
        !pos_backend::domain::sanitizer::validate_safe_rpc_url("http://myserver.local:8080/rpc"),
        "279: .local TLD not blocked"
    );
}

#[test]
fn test_280_validate_rpc_url_tld_internal() {
    assert!(
        !pos_backend::domain::sanitizer::validate_safe_rpc_url("http://myserver.internal:8080/rpc"),
        "280: .internal TLD not blocked"
    );
}

#[test]
fn test_281_is_payment_amount_valid() {
    assert!(
        pos_backend::domain::sanitizer::is_payment_amount_valid(9.9, 10.0, 1.0),
        "281: 9.9 within 1% of 10.0 should be valid"
    );
    assert!(
        !pos_backend::domain::sanitizer::is_payment_amount_valid(9.8, 10.0, 1.0),
        "281: 9.8 outside 1% of 10.0 should be invalid"
    );
}

#[test]
fn test_328_validate_rpc_url_local_toggle() {
    assert!(
        !pos_backend::domain::sanitizer::validate_safe_rpc_url("http://127.0.0.1:8899"),
        "328: default validate_safe_rpc_url blocks http://127.0.0.1"
    );
    assert!(
        pos_backend::domain::sanitizer::validate_safe_rpc_url_with_config(
            "http://127.0.0.1:8899",
            true
        ),
        "328: validate_safe_rpc_url_with_config allows http://127.0.0.1 when allow_local_rpc=true"
    );
    assert!(
        pos_backend::domain::sanitizer::validate_safe_rpc_url_with_config(
            "http://localhost:8899",
            true
        ),
        "328: validate_safe_rpc_url_with_config allows http://localhost when allow_local_rpc=true"
    );
}
