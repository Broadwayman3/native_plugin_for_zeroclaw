/// Multi-tier price feed fallback circuit breaker.
/// 4 tiers: Switchboard primary, Pyth secondary, cache tertiary, static offline quaternary.
use std::collections::HashMap;

/// Default static fiat rates for offline fallback.
pub fn get_static_fiat_rates() -> HashMap<String, f64> {
    let mut rates = HashMap::new();
    rates.insert("USD".to_string(), 1.00);
    rates.insert("EUR".to_string(), 0.92);
    rates.insert("BRL".to_string(), 5.45);
    rates.insert("UAH".to_string(), 41.50);
    rates.insert("GBP".to_string(), 0.78);
    rates.insert("CAD".to_string(), 1.37);
    rates.insert("JPY".to_string(), 152.50);
    rates.insert("MXN".to_string(), 19.80);
    rates.insert("PLN".to_string(), 3.98);
    rates.insert("CHF".to_string(), 0.88);
    rates.insert("AUD".to_string(), 1.52);
    rates.insert("SEK".to_string(), 10.45);
    rates.insert("NOK".to_string(), 10.85);
    rates.insert("DKK".to_string(), 6.88);
    rates.insert("NZD".to_string(), 1.65);
    rates.insert("SGD".to_string(), 1.34);
    rates.insert("HKD".to_string(), 7.81);
    rates.insert("INR".to_string(), 83.70);
    rates.insert("TRY".to_string(), 33.10);
    rates.insert("ZAR".to_string(), 18.20);
    rates.insert("AED".to_string(), 3.67);
    rates.insert("CZK".to_string(), 23.20);
    rates.insert("HUF".to_string(), 365.00);
    rates.insert("THB".to_string(), 35.80);
    rates.insert("PHP".to_string(), 58.40);
    rates.insert("IDR".to_string(), 16250.0);
    rates.insert("ILS".to_string(), 3.72);
    rates.insert("CLP".to_string(), 940.00);
    rates.insert("COP".to_string(), 4050.00);
    rates.insert("ARS".to_string(), 930.00);
    rates
}

/// Parses Pyth Hermes V2 REST response JSON into normalized rate and timestamp.
/// Response structure: parsed[0].price.price (i64) and parsed[0].price.expo (i32).
/// Includes overflow/underflow protection, is_finite() check, and exponent bounds validation.
pub fn parse_pyth_hermes_v2_price(pyth_json: &serde_json::Value) -> Option<(f64, i64)> {
    let parsed_array = pyth_json.get("parsed")?.as_array()?;
    let first_entry = parsed_array.first()?;
    let price_obj = first_entry.get("price")?;

    let price_raw = price_obj.get("price")?.as_str()?.parse::<f64>().ok()?;
    let expo = price_obj.get("expo")?.as_i64()?;
    let publish_time = price_obj.get("publish_time")?.as_i64()?;

    // Bound exponent check (-30..=30) to prevent overflow/underflow
    if !(-30..=30).contains(&expo) {
        return None;
    }

    let rate = price_raw * 10f64.powi(expo as i32);
    if rate > 0.0 && rate.is_finite() {
        Some((rate, publish_time))
    } else {
        None
    }
}

/// Multi-tier price feed fallback circuit breaker.
pub fn get_multitier_fiat_rate(
    fiat_currency: &str,
    primary_data: Option<&serde_json::Value>,
    secondary_data: Option<&serde_json::Value>,
    cached_data: Option<&serde_json::Value>,
    current_ts: Option<i64>,
    allow_static_fallback: bool,
) -> Result<serde_json::Value, &'static str> {
    let ts = current_ts.unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    });

    let curr = fiat_currency.to_uppercase();

    // Tier 1: Primary Switchboard
    if let Some(data) = primary_data {
        if let Some(rate) = data.get("rate").and_then(|v| v.as_f64()) {
            let data_ts = data.get("timestamp").and_then(|v| v.as_i64()).unwrap_or(0);
            let data_ts = if data_ts > 1_000_000_000_000 {
                data_ts / 1000
            } else {
                data_ts
            };
            let diff = ts - data_ts;
            if rate > 0.0 && (-15..=300).contains(&diff) {
                return Ok(serde_json::json!({
                    "rate": rate,
                    "tier": "primary_switchboard",
                    "status": "OK"
                }));
            }
        }
    }

    // Tier 2: Secondary Pyth / REST Fiat API
    if let Some(data) = secondary_data {
        if let Some(rate) = data.get("rate").and_then(|v| v.as_f64()) {
            let data_ts = data.get("timestamp").and_then(|v| v.as_i64()).unwrap_or(0);
            let data_ts = if data_ts > 1_000_000_000_000 {
                data_ts / 1000
            } else {
                data_ts
            };
            let diff = ts - data_ts;
            if rate > 0.0 && (-15..=300).contains(&diff) {
                return Ok(serde_json::json!({
                    "rate": rate,
                    "tier": "secondary_pyth_hermes",
                    "status": "OK"
                }));
            }
        }
    }

    // Tier 3: Tertiary Cached Fallback
    if let Some(data) = cached_data {
        if let Some(rate) = data.get("rate").and_then(|v| v.as_f64()) {
            let data_ts = data.get("timestamp").and_then(|v| v.as_i64()).unwrap_or(0);
            let data_ts = if data_ts > 1_000_000_000_000 {
                data_ts / 1000
            } else {
                data_ts
            };
            let diff = ts - data_ts;
            if rate > 0.0 && (-15..=900).contains(&diff) {
                return Ok(serde_json::json!({
                    "rate": rate,
                    "tier": "tertiary_cache",
                    "status": "WARNING_USING_CACHE"
                }));
            }
        }
    }

    // Tier 4: Quaternary Static Offline Fallback
    if allow_static_fallback {
        let rates = get_static_fiat_rates();
        if let Some(&rate) = rates.get(&curr) {
            return Ok(serde_json::json!({
                "rate": rate,
                "tier": "quaternary_static_fallback",
                "status": "OFFLINE_STATIC_FALLBACK"
            }));
        }
    }

    // Fail-Closed Guard
    Err("FAIL_CLOSED: Unsupported or unmapped fiat currency")
}

/// Returns true if the price feed result indicates static offline fallback was used.
pub fn is_static_fallback(rate_data: &serde_json::Value) -> bool {
    rate_data
        .get("tier")
        .and_then(|v| v.as_str())
        .map(|t| t == "quaternary_static_fallback")
        .unwrap_or(false)
}
