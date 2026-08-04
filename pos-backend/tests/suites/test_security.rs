#[test]
fn test_143_is_btn_click_correct_direction() {
    use pos_backend::domain::keyboards::is_btn_click;
    assert!(
        is_btn_click("✍️ Enter custom amount", "btn_custom"),
        "143a: exact text should match"
    );
    assert!(
        !is_btn_click("custom", "btn_custom"),
        "143b: partial text should NOT match"
    );
    assert!(
        is_btn_click("☕ Quick receipt ({amount} {currency})", "btn_quick_uah"),
        "143c: quick receipt template should match"
    );
}

#[test]
fn test_144_ipv6_bypass_blocked() {
    use pos_backend::domain::sanitizer::validate_safe_rpc_url;
    let cases = vec![
        ("http://[::1]:8080", "IPv6 loopback"),
        ("http://[::]:8080", "IPv6 unspecified"),
        ("http://[fe80::1]:8080", "IPv6 link-local"),
        ("http://[fc00::1]:8080", "IPv6 unique-local"),
        ("http://[2001:db8::1]:8080", "IPv6 documentation"),
    ];
    for (url, desc) in cases {
        assert!(!validate_safe_rpc_url(url), "144: {} not blocked", desc);
    }
}

#[test]
fn test_145_dns_timeout_fail_closed() {
    use pos_backend::domain::sanitizer::validate_safe_rpc_url;
    assert!(
        !validate_safe_rpc_url("http://nonexistent.invalid:8080"),
        "145: nonexistent domain not blocked"
    );
}

#[test]
fn test_146_ssrf_localhost_blocked() {
    use pos_backend::domain::sanitizer::validate_safe_rpc_url;
    let cases = vec![
        ("http://localhost:8080", "localhost hostname"),
        ("http://127.0.0.1:8080", "IPv4 loopback"),
        ("http://0.0.0.0:8080", "unspecified address"),
        ("http://[::1]:8080", "IPv6 loopback"),
    ];
    for (url, desc) in cases {
        assert!(!validate_safe_rpc_url(url), "146: {} not blocked", desc);
    }
}
