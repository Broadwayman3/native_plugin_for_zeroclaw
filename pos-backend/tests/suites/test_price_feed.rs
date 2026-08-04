use crate::{test_fail, test_pass};

pub fn run_suite() {
    println!("\n📦 Price Feed Tests (061-080)");
    test_061_static_rates_usd();
    test_062_static_rates_uah();
    test_063_static_rates_brl();
    test_064_fallback_to_static();
    test_065_unmapped_currency_fails();
    test_066_static_rates_count();
    test_067_rate_tier_primary();
    test_068_rate_tier_secondary();
    test_069_rate_tier_cache();
    test_070_rate_tier_static();
    test_071_stale_primary_fallback_to_secondary();
    test_072_millisecond_timestamp_normalized();
    test_073_negative_rate_rejected();
    test_074_zero_rate_rejected();
    test_075_all_tiers_stale_fallback_static();
    test_076_static_fallback_disabled();
    test_077_cache_tier_wider_window();
    test_078_primary_rate_returned_correctly();
    test_079_secondary_rate_returned_correctly();
    test_080_cache_rate_returned_correctly();
}

fn test_061_static_rates_usd() {
    let rates = pos_backend::domain::price_feed::get_static_fiat_rates();
    if rates.get("USD") == Some(&1.0) {
        test_pass("061: USD rate = 1.0");
    } else {
        test_fail("061", "USD rate not found");
    }
}

fn test_062_static_rates_uah() {
    let rates = pos_backend::domain::price_feed::get_static_fiat_rates();
    if rates.get("UAH").is_some() && *rates.get("UAH").unwrap() > 30.0 {
        test_pass("062: UAH rate > 30");
    } else {
        test_fail("062", "UAH rate not found or too low");
    }
}

fn test_063_static_rates_brl() {
    let rates = pos_backend::domain::price_feed::get_static_fiat_rates();
    if rates.get("BRL").is_some() && *rates.get("BRL").unwrap() > 4.0 {
        test_pass("063: BRL rate > 4");
    } else {
        test_fail("063", "BRL rate not found or too low");
    }
}

fn test_064_fallback_to_static() {
    let result = pos_backend::domain::price_feed::get_multitier_fiat_rate(
        "UAH", None, None, None, None, true,
    );
    match result {
        Ok(r) if r.get("tier").and_then(|v| v.as_str()) == Some("quaternary_static_fallback") => {
            test_pass("064: fallback to static rate");
        }
        Ok(r) => test_fail("064", &format!("tier: {:?}", r.get("tier"))),
        Err(e) => test_fail("064", e),
    }
}

fn test_065_unmapped_currency_fails() {
    let result = pos_backend::domain::price_feed::get_multitier_fiat_rate(
        "XYZ", None, None, None, None, true,
    );
    if result.is_err() {
        test_pass("065: unmapped currency fails");
    } else {
        test_fail("065", "expected error for unmapped currency");
    }
}

fn test_066_static_rates_count() {
    let rates = pos_backend::domain::price_feed::get_static_fiat_rates();
    if rates.len() >= 20 {
        test_pass("066: >= 20 static rates");
    } else {
        test_fail("066", &format!("count = {}", rates.len()));
    }
}

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
        Ok(r) if r.get("tier").and_then(|v| v.as_str()) == Some("primary_switchboard") => {
            test_pass("067: primary tier selected");
        }
        Ok(r) => test_fail("067", &format!("tier: {:?}", r.get("tier"))),
        Err(e) => test_fail("067", e),
    }
}

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
        Ok(r) if r.get("tier").and_then(|v| v.as_str()) == Some("secondary_pyth_hermes") => {
            test_pass("068: secondary tier selected");
        }
        Ok(r) => test_fail("068", &format!("tier: {:?}", r.get("tier"))),
        Err(e) => test_fail("068", e),
    }
}

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
        Ok(r) if r.get("tier").and_then(|v| v.as_str()) == Some("tertiary_cache") => {
            test_pass("069: tertiary cache tier selected");
        }
        Ok(r) => test_fail("069", &format!("tier: {:?}", r.get("tier"))),
        Err(e) => test_fail("069", e),
    }
}

fn test_070_rate_tier_static() {
    let result = pos_backend::domain::price_feed::get_multitier_fiat_rate(
        "EUR", None, None, None, None, true,
    );
    match result {
        Ok(r) if r.get("tier").and_then(|v| v.as_str()) == Some("quaternary_static_fallback") => {
            test_pass("070: quaternary static tier selected");
        }
        Ok(r) => test_fail("070", &format!("tier: {:?}", r.get("tier"))),
        Err(e) => test_fail("070", e),
    }
}

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
        Ok(r) if r.get("tier").and_then(|v| v.as_str()) == Some("secondary_pyth_hermes") => {
            test_pass("071: stale primary falls back to secondary");
        }
        Ok(r) => test_fail("071", &format!("tier: {:?}", r.get("tier"))),
        Err(e) => test_fail("071", e),
    }
}

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
        Ok(r) if r.get("rate").and_then(|v| v.as_f64()) == Some(41.5) => {
            test_pass("072: millisecond timestamp normalized to seconds");
        }
        Ok(r) => test_fail("072", &format!("rate: {:?}", r.get("rate"))),
        Err(e) => test_fail("072", e),
    }
}

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
    match result {
        Ok(r) => test_fail("073", &format!("should reject negative rate, got: {:?}", r)),
        Err(_) => test_pass("073: negative rate rejected"),
    }
}

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
    match result {
        Ok(r) => test_fail("074", &format!("should reject zero rate, got: {:?}", r)),
        Err(_) => test_pass("074: zero rate rejected"),
    }
}

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
        Ok(r) if r.get("tier").and_then(|v| v.as_str()) == Some("quaternary_static_fallback") => {
            test_pass("075: all stale tiers fall back to static");
        }
        Ok(r) => test_fail("075", &format!("tier: {:?}", r.get("tier"))),
        Err(e) => test_fail("075", e),
    }
}

fn test_076_static_fallback_disabled() {
    let result = pos_backend::domain::price_feed::get_multitier_fiat_rate(
        "EUR", None, None, None, None, false,
    );
    if result.is_err() {
        test_pass("076: static fallback disabled returns error");
    } else {
        test_fail("076", "expected error when static fallback disabled");
    }
}

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
        Ok(r) if r.get("tier").and_then(|v| v.as_str()) == Some("tertiary_cache") => {
            test_pass("077: cache tier accepts 600s-old data");
        }
        Ok(r) => test_fail("077", &format!("tier: {:?}", r.get("tier"))),
        Err(e) => test_fail("077", e),
    }
}

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
        Ok(r) if r.get("rate").and_then(|v| v.as_f64()) == Some(41.5) => {
            test_pass("078: primary rate value returned correctly");
        }
        Ok(r) => test_fail("078", &format!("rate: {:?}", r.get("rate"))),
        Err(e) => test_fail("078", e),
    }
}

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
        Ok(r) if r.get("rate").and_then(|v| v.as_f64()) == Some(42.0) => {
            test_pass("079: secondary rate value returned correctly");
        }
        Ok(r) => test_fail("079", &format!("rate: {:?}", r.get("rate"))),
        Err(e) => test_fail("079", e),
    }
}

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
        Ok(r) if r.get("rate").and_then(|v| v.as_f64()) == Some(43.0) => {
            test_pass("080: cache rate value returned correctly");
        }
        Ok(r) => test_fail("080", &format!("rate: {:?}", r.get("rate"))),
        Err(e) => test_fail("080", e),
    }
}
