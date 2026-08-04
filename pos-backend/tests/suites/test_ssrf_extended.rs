use crate::{test_fail, test_pass};

pub fn run_suite() {
    println!("\n📦 SSRF Extended Tests (268-281)");
    test_268_validate_rpc_url_loopback();
    test_269_validate_rpc_url_unspecified();
    test_270_validate_rpc_url_link_local();
    test_271_validate_rpc_url_broadcast();
    test_272_validate_rpc_url_reserved();
    test_273_validate_rpc_url_reserved_v4();
    test_274_validate_rpc_url_reserved_v6();
    test_275_validate_rpc_url_multicast_v6();
    test_276_validate_rpc_url_private_v4();
    test_277_validate_rpc_url_documentation_v6();
    test_278_validate_rpc_url_unique_local_v6();
    test_279_validate_rpc_url_tld_local();
    test_280_validate_rpc_url_tld_internal();
    test_281_is_payment_amount_valid();
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

fn test_277_validate_rpc_url_documentation_v6() {
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
