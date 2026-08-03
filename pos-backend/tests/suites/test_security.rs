use crate::{test_fail, test_pass};

pub fn run_suite() {
    println!("\n📦 Security Regression Tests (141-143)");
    test_141_ssrf_blocks_ipv6_reserved();
    test_142_ssrf_dns_timeout();
    test_143_is_btn_click_correct_direction();
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
