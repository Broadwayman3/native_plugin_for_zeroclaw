use crate::{test_fail, test_pass};

pub fn run_suite() {
    println!("\n📦 Security Regression Tests (141-148)");
    test_141_ssrf_blocks_ipv6_reserved();
    test_142_ssrf_dns_timeout();
    test_143_is_btn_click_correct_direction();
    test_144_ipv6_bypass_blocked();
    test_145_dns_timeout_fail_closed();
    test_146_ssrf_localhost_blocked();
}

fn test_141_ssrf_blocks_ipv6_reserved() {
    use pos_backend::domain::sanitizer::validate_safe_rpc_url;
    let mut all_passed = true;

    if validate_safe_rpc_url("http://[::1]:8080/rpc") {
        test_fail("141a", "IPv6 loopback ::1 not blocked");
        all_passed = false;
    }
    if validate_safe_rpc_url("http://[fe80::1]:8080/rpc") {
        test_fail("141b", "IPv6 link-local fe80::1 not blocked");
        all_passed = false;
    }
    if validate_safe_rpc_url("http://[fc00::1]:8080/rpc") {
        test_fail("141c", "IPv6 unique-local fc00::1 not blocked");
        all_passed = false;
    }
    if validate_safe_rpc_url("http://[2001:db8::1]:8080/rpc") {
        test_fail("141d", "IPv6 documentation 2001:db8::1 not blocked");
        all_passed = false;
    }

    if all_passed {
        test_pass("141: IPv6 SSRF blocks all reserved ranges");
    }
}

fn test_142_ssrf_dns_timeout() {
    use pos_backend::domain::sanitizer::validate_safe_rpc_url;
    // Non-existent domain should be blocked (fail-closed)
    if validate_safe_rpc_url("http://this-domain-definitely-does-not-exist-xyzzy.com:8080/rpc") {
        test_fail("142", "Non-existent domain not blocked");
    } else {
        test_pass("142: DNS timeout fail-closed works");
    }
}

fn test_143_is_btn_click_correct_direction() {
    use pos_backend::domain::keyboards::is_btn_click;
    let mut all_passed = true;

    // User types button text → should match
    if !is_btn_click("✍️ Enter custom amount", "btn_custom") {
        test_fail("143a", "Exact button text should match");
        all_passed = false;
    }

    // User types partial → should NOT match
    if is_btn_click("custom", "btn_custom") {
        test_fail("143b", "Partial text should NOT match");
        all_passed = false;
    }

    // Another test case
    if !is_btn_click("☕ Quick receipt (200 UAH)", "btn_quick_uah") {
        test_fail("143c", "Quick UAH button text should match");
        all_passed = false;
    }

    if all_passed {
        test_pass("143: is_btn_click direction correct");
    }
}

fn test_144_ipv6_bypass_blocked() {
    use pos_backend::domain::sanitizer::validate_safe_rpc_url;
    let mut all_passed = true;

    // All IPv6 reserved ranges should be blocked
    let test_cases = vec![
        ("http://[::1]:8080", "IPv6 loopback"),
        ("http://[::]:8080", "IPv6 unspecified"),
        ("http://[fe80::1]:8080", "IPv6 link-local"),
        ("http://[fc00::1]:8080", "IPv6 unique-local"),
        ("http://[2001:db8::1]:8080", "IPv6 documentation"),
    ];

    for (url, desc) in test_cases {
        if validate_safe_rpc_url(url) {
            test_fail("144", &format!("{} not blocked", desc));
            all_passed = false;
        }
    }

    if all_passed {
        test_pass("144: all IPv6 bypass attempts blocked");
    }
}

fn test_145_dns_timeout_fail_closed() {
    use pos_backend::domain::sanitizer::validate_safe_rpc_url;
    // Non-existent domain should return false (fail-closed)
    if validate_safe_rpc_url("http://nonexistent.invalid:8080") {
        test_fail("145", "nonexistent domain not blocked");
    } else {
        test_pass("145: DNS timeout fail-closed works");
    }
}

fn test_146_ssrf_localhost_blocked() {
    use pos_backend::domain::sanitizer::validate_safe_rpc_url;
    let mut all_passed = true;

    // All localhost/loopback variants should be blocked
    let test_cases = vec![
        ("http://localhost:8080", "localhost hostname"),
        ("http://127.0.0.1:8080", "IPv4 loopback"),
        ("http://0.0.0.0:8080", "unspecified address"),
        ("http://[::1]:8080", "IPv6 loopback"),
    ];

    for (url, desc) in test_cases {
        if validate_safe_rpc_url(url) {
            test_fail("146", &format!("{} not blocked", desc));
            all_passed = false;
        }
    }

    if all_passed {
        test_pass("146: all localhost/loopback blocked");
    }
}
