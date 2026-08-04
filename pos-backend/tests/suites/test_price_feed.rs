#[test]
fn test_061_static_rates_usd() {
    let rates = pos_backend::domain::price_feed::get_static_fiat_rates();
    assert_eq!(rates.get("USD"), Some(&1.0), "061: USD rate not found");
}

#[test]
fn test_062_static_rates_uah() {
    let rates = pos_backend::domain::price_feed::get_static_fiat_rates();
    assert!(
        rates.get("UAH").is_some() && *rates.get("UAH").unwrap() > 30.0,
        "062: UAH rate not found or too low"
    );
}

#[test]
fn test_063_static_rates_brl() {
    let rates = pos_backend::domain::price_feed::get_static_fiat_rates();
    assert!(
        rates.get("BRL").is_some() && *rates.get("BRL").unwrap() > 4.0,
        "063: BRL rate not found or too low"
    );
}

#[test]
fn test_064_fallback_to_static() {
    let result = pos_backend::domain::price_feed::get_multitier_fiat_rate(
        "UAH", None, None, None, None, true,
    );
    match result {
        Ok(r) => {
            let tier = r.get("tier").and_then(|v| v.as_str());
            assert_eq!(
                tier,
                Some("quaternary_static_fallback"),
                "064: tier: {:?}",
                r.get("tier")
            );
        }
        Err(e) => panic!("064: {}", e),
    }
}

#[test]
fn test_065_unmapped_currency_fails() {
    let result = pos_backend::domain::price_feed::get_multitier_fiat_rate(
        "XYZ", None, None, None, None, true,
    );
    assert!(result.is_err(), "065: expected error for unmapped currency");
}

#[test]
fn test_066_static_rates_count() {
    let rates = pos_backend::domain::price_feed::get_static_fiat_rates();
    assert!(rates.len() >= 20, "066: count = {}", rates.len());
}

#[test]
fn test_067_rate_tier_primary() {
    let data = serde_json::json!({"rate": 41.5, "timestamp": 1700000000});
    let result = pos_backend::domain::price_feed::get_multitier_fiat_rate(
        "UAH",
        Some(&data),
        None,
        None,
        Some(1700000050),
        true,
    );
    match result {
        Ok(r) => {
            let tier = r.get("tier").and_then(|v| v.as_str());
            assert_eq!(
                tier,
                Some("primary_switchboard"),
                "067: tier: {:?}",
                r.get("tier")
            );
        }
        Err(e) => panic!("067: {}", e),
    }
}

#[test]
fn test_068_rate_tier_secondary() {
    let data = serde_json::json!({"rate": 41.5, "timestamp": 1700000000});
    let result = pos_backend::domain::price_feed::get_multitier_fiat_rate(
        "UAH",
        None,
        Some(&data),
        None,
        Some(1700000050),
        true,
    );
    match result {
        Ok(r) => {
            let tier = r.get("tier").and_then(|v| v.as_str());
            assert_eq!(
                tier,
                Some("secondary_pyth_hermes"),
                "068: tier: {:?}",
                r.get("tier")
            );
        }
        Err(e) => panic!("068: {}", e),
    }
}

#[test]
fn test_069_rate_tier_cache() {
    let data = serde_json::json!({"rate": 41.5, "timestamp": 1700000000});
    let result = pos_backend::domain::price_feed::get_multitier_fiat_rate(
        "UAH",
        None,
        None,
        Some(&data),
        Some(1700000050),
        true,
    );
    match result {
        Ok(r) => {
            let tier = r.get("tier").and_then(|v| v.as_str());
            assert_eq!(
                tier,
                Some("tertiary_cache"),
                "069: tier: {:?}",
                r.get("tier")
            );
        }
        Err(e) => panic!("069: {}", e),
    }
}

#[test]
fn test_070_rate_tier_static() {
    let result = pos_backend::domain::price_feed::get_multitier_fiat_rate(
        "EUR", None, None, None, None, true,
    );
    match result {
        Ok(r) => {
            let tier = r.get("tier").and_then(|v| v.as_str());
            assert_eq!(
                tier,
                Some("quaternary_static_fallback"),
                "070: tier: {:?}",
                r.get("tier")
            );
        }
        Err(e) => panic!("070: {}", e),
    }
}

#[test]
fn test_071_stale_primary_fallback_to_secondary() {
    let now = 1700000000i64;
    let stale_ts = now - 600; // 10 minutes old (> 300s threshold)
    let primary = serde_json::json!({"rate": 41.5, "timestamp": stale_ts});
    let secondary = serde_json::json!({"rate": 42.0, "timestamp": now});

    let result = pos_backend::domain::price_feed::get_multitier_fiat_rate(
        "UAH",
        Some(&primary),
        Some(&secondary),
        None,
        Some(now),
        false,
    );
    match result {
        Ok(r) => {
            let tier = r.get("tier").and_then(|v| v.as_str());
            assert_eq!(
                tier,
                Some("secondary_pyth_hermes"),
                "071: tier: {:?}",
                r.get("tier")
            );
        }
        Err(e) => panic!("071: {}", e),
    }
}

#[test]
fn test_072_millisecond_timestamp_normalized() {
    let now = 1700000000i64;
    let ms_ts = now * 1000; // Millisecond timestamp
    let primary = serde_json::json!({"rate": 41.5, "timestamp": ms_ts});

    let result = pos_backend::domain::price_feed::get_multitier_fiat_rate(
        "UAH",
        Some(&primary),
        None,
        None,
        Some(now),
        false,
    );
    match result {
        Ok(r) => {
            let rate = r.get("rate").and_then(|v| v.as_f64());
            assert_eq!(rate, Some(41.5), "072: rate: {:?}", r.get("rate"));
        }
        Err(e) => panic!("072: {}", e),
    }
}

#[test]
fn test_073_negative_rate_rejected() {
    let now = 1700000000i64;
    let primary = serde_json::json!({"rate": -41.5, "timestamp": now});
    let result = pos_backend::domain::price_feed::get_multitier_fiat_rate(
        "UAH",
        Some(&primary),
        None,
        None,
        Some(now),
        false,
    );
    // Negative rate should be rejected, falling through to next tier
    assert!(
        result.is_err(),
        "073: should reject negative rate, got: {:?}",
        result.ok()
    );
}

#[test]
fn test_074_zero_rate_rejected() {
    let now = 1700000000i64;
    let primary = serde_json::json!({"rate": 0.0, "timestamp": now});
    let result = pos_backend::domain::price_feed::get_multitier_fiat_rate(
        "UAH",
        Some(&primary),
        None,
        None,
        Some(now),
        false,
    );
    assert!(
        result.is_err(),
        "074: should reject zero rate, got: {:?}",
        result.ok()
    );
}

#[test]
fn test_075_all_tiers_stale_fallback_static() {
    let now = 1700000000i64;
    let stale_ts = now - 1000; // >900s old — exceeds cache window too
    let primary = serde_json::json!({"rate": 41.5, "timestamp": stale_ts});
    let secondary = serde_json::json!({"rate": 42.0, "timestamp": stale_ts});
    let cache = serde_json::json!({"rate": 43.0, "timestamp": stale_ts});

    let result = pos_backend::domain::price_feed::get_multitier_fiat_rate(
        "UAH",
        Some(&primary),
        Some(&secondary),
        Some(&cache),
        Some(now),
        true,
    );
    match result {
        Ok(r) => {
            let tier = r.get("tier").and_then(|v| v.as_str());
            assert_eq!(
                tier,
                Some("quaternary_static_fallback"),
                "075: tier: {:?}",
                r.get("tier")
            );
        }
        Err(e) => panic!("075: {}", e),
    }
}

#[test]
fn test_076_static_fallback_disabled() {
    let result = pos_backend::domain::price_feed::get_multitier_fiat_rate(
        "EUR", None, None, None, None, false,
    );
    assert!(
        result.is_err(),
        "076: expected error when static fallback disabled"
    );
}

#[test]
fn test_077_cache_tier_wider_window() {
    let now = 1700000000i64;
    // Cache data is 600s old (within 900s window for cache, but >300s for primary/secondary)
    let cache_ts = now - 600;
    let cache = serde_json::json!({"rate": 43.0, "timestamp": cache_ts});

    let result = pos_backend::domain::price_feed::get_multitier_fiat_rate(
        "UAH",
        None,
        None,
        Some(&cache),
        Some(now),
        false,
    );
    match result {
        Ok(r) => {
            let tier = r.get("tier").and_then(|v| v.as_str());
            assert_eq!(
                tier,
                Some("tertiary_cache"),
                "077: tier: {:?}",
                r.get("tier")
            );
        }
        Err(e) => panic!("077: {}", e),
    }
}

#[test]
fn test_078_primary_rate_returned_correctly() {
    let now = 1700000000i64;
    let primary = serde_json::json!({"rate": 41.5, "timestamp": now});
    let result = pos_backend::domain::price_feed::get_multitier_fiat_rate(
        "UAH",
        Some(&primary),
        None,
        None,
        Some(now),
        false,
    );
    match result {
        Ok(r) => {
            let rate = r.get("rate").and_then(|v| v.as_f64());
            assert_eq!(rate, Some(41.5), "078: rate: {:?}", r.get("rate"));
        }
        Err(e) => panic!("078: {}", e),
    }
}

#[test]
fn test_079_secondary_rate_returned_correctly() {
    let now = 1700000000i64;
    let secondary = serde_json::json!({"rate": 42.0, "timestamp": now});
    let result = pos_backend::domain::price_feed::get_multitier_fiat_rate(
        "UAH",
        None,
        Some(&secondary),
        None,
        Some(now),
        false,
    );
    match result {
        Ok(r) => {
            let rate = r.get("rate").and_then(|v| v.as_f64());
            assert_eq!(rate, Some(42.0), "079: rate: {:?}", r.get("rate"));
        }
        Err(e) => panic!("079: {}", e),
    }
}

#[test]
fn test_080_cache_rate_returned_correctly() {
    let now = 1700000000i64;
    let cache = serde_json::json!({"rate": 43.0, "timestamp": now});
    let result = pos_backend::domain::price_feed::get_multitier_fiat_rate(
        "UAH",
        None,
        None,
        Some(&cache),
        Some(now),
        false,
    );
    match result {
        Ok(r) => {
            let rate = r.get("rate").and_then(|v| v.as_f64());
            assert_eq!(rate, Some(43.0), "080: rate: {:?}", r.get("rate"));
        }
        Err(e) => panic!("080: {}", e),
    }
}
