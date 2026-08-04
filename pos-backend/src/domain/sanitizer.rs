use once_cell::sync::Lazy;
use regex::Regex;
use std::net::ToSocketAddrs;

// Lazy-compiled regex statics (compiled once, reused on every call)
static RE_CONTROL: Lazy<Regex> = Lazy::new(|| Regex::new(r"[\r\n\t\x00-\x1f\x7f-\x9f]").unwrap());
static RE_INVISIBLE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[\u{200B}-\u{200D}\u{FEFF}\u{202E}\u{00AD}\u{200E}\u{200F}\u{2060}]").unwrap()
});
static RE_INJECTION: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(system\s*:|override|ignore\s+previous|approve_refund|developer\s+mode)")
        .unwrap()
});
static RE_ESCAPE_MD: Lazy<Regex> = Lazy::new(|| {
    let escape_chars = r"_*[]()~`>#+-=|{}.!";
    Regex::new(&format!(r"([{}])", regex::escape(escape_chars))).unwrap()
});
static RE_API_KEY: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)(api[_-]?key|token|secret)=[^&\s]+").unwrap());
static RE_BYTE_ARRAY: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\[\s*\d{1,3}\s*(?:,\s*\d{1,3}\s*){31,}\]").unwrap());

/// Cleans external untrusted strings from control characters, system tags, and prompt injection patterns.
pub fn sanitize_external_input(user_string: &str, max_length: usize) -> String {
    if user_string.is_empty() {
        return String::new();
    }

    // 0. Unicode NFKC Normalization
    use unicode_normalization::UnicodeNormalization;
    let mut cleaned: String = user_string.nfkc().collect();

    // 1. Remove control characters, system tags, and line breaks
    cleaned = RE_CONTROL.replace_all(&cleaned, " ").to_string();

    // 2. Strip invisible zero-width and directional Unicode characters
    cleaned = RE_INVISIBLE.replace_all(&cleaned, "").to_string();

    // 3. Case-insensitive removal of prompt injection keywords
    cleaned = RE_INJECTION.replace_all(&cleaned, "").to_string();

    // 4. Strip leading/trailing whitespace and limit length
    cleaned.trim().chars().take(max_length).collect()
}

/// Redacts API keys, tokens, secrets, and Solana byte array keypairs from error messages.
pub fn redact_api_key(error_msg: &str) -> String {
    if error_msg.is_empty() {
        return String::new();
    }

    let masked = RE_API_KEY.replace_all(error_msg, "$1=REDACTED");
    RE_BYTE_ARRAY
        .replace_all(&masked, "[REDACTED_BYTE_KEYPAIR]")
        .to_string()
}

/// Escapes special characters for Telegram MarkdownV2 format.
pub fn escape_telegram_markdown_v2(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }

    RE_ESCAPE_MD.replace_all(text, r"\$1").to_string()
}

/// Evaluates Solana RPC URL to prevent SSRF attacks.
pub fn validate_safe_rpc_url(url_str: &str) -> bool {
    if url_str.is_empty() {
        return false;
    }

    // Must be HTTPS
    if !url_str.starts_with("https://") {
        return false;
    }

    let parsed = match url::Url::parse(url_str) {
        Ok(u) => u,
        Err(_) => return false,
    };

    let hostname = match parsed.host_str() {
        Some(h) => h,
        None => return false,
    };

    // Check if hostname is an IP address
    if let Ok(ip) = hostname.parse::<std::net::IpAddr>() {
        match ip {
            std::net::IpAddr::V4(v4) => {
                if v4.is_private() || is_reserved_v4(&v4) || v4.is_link_local() || v4.is_broadcast()
                {
                    return false;
                }
            }
            std::net::IpAddr::V6(v6) => {
                if v6.is_loopback() || v6.is_unspecified() || v6.is_multicast() {
                    return false;
                }
                // IPv4-mapped IPv6: ::ffff:127.0.0.1, ::ffff:10.x.x.x, etc.
                if let Some(v4) = v6.to_ipv4_mapped() {
                    if v4.is_private()
                        || v4.is_loopback()
                        || is_reserved_v4(&v4)
                        || v4.is_link_local()
                    {
                        return false;
                    }
                }
                let octets = v6.octets();
                // fe80::/10 (link-local)
                if octets[0] == 0xfe && (octets[1] & 0xc0) == 0x80 {
                    return false;
                }
                // fc00::/7 (unique local)
                if (octets[0] & 0xfe) == 0xfc {
                    return false;
                }
                // 2001:db8::/32 (documentation)
                if octets[0..4] == [0x20, 0x01, 0x0d, 0xb8] {
                    return false;
                }
            }
        }
    } else {
        // Not an IP - check for suspicious hostnames
        if hostname.contains("127.0.0.1")
            || hostname.contains("169.254.169.254")
            || hostname.ends_with(".local")
            || hostname.ends_with(".internal")
        {
            return false;
        }

        // DNS resolution check with timeout (2 seconds max)
        // NOTE: This blocks for up to 2 seconds. Called synchronously from
        // validate_safe_rpc_url. Do NOT call from async context.
        let addr_str = format!("{}:443", hostname);
        let result = std::thread::scope(|s| {
            let handle = s.spawn(|| {
                addr_str
                    .to_socket_addrs()
                    .map(|addrs| addrs.collect::<Vec<_>>())
            });
            std::thread::sleep(std::time::Duration::from_secs(2));
            handle.join().unwrap_or(Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "DNS resolution timeout",
            )))
        });
        match result {
            Ok(addrs) => {
                for addr in addrs {
                    let ip = addr.ip();
                    if ip.is_loopback() || ip.is_unspecified() {
                        return false;
                    }
                    if let std::net::IpAddr::V4(v4) = ip {
                        if v4.is_private() || is_reserved_v4(&v4) {
                            return false;
                        }
                    }
                }
            }
            Err(_) => return false,
        }
    }

    true
}

fn is_reserved_v4(v4: &std::net::Ipv4Addr) -> bool {
    // 0.0.0.0/8, 100.64.0.0/10, 169.254.0.0/16, 192.0.0.0/24, 192.0.2.0/24,
    // 198.18.0.0/15, 198.51.100.0/24, 203.0.113.0/24, 240.0.0.0/4, 255.255.255.255/32
    let octets = v4.octets();
    matches!(
        octets[0],
        0 | 100..=100 | 169 | 192 | 198 | 203 | 240..=255
    ) && (octets[0] != 100 || (octets[1] & 0xC0) == 64)
        && (octets[0] != 169 || octets[1] == 254)
        && (octets[0] != 192 || octets[1] != 0 || octets[2] != 0)
        && (octets[0] != 192 || octets[1] != 0 || octets[2] != 2)
        && (octets[0] != 198 || (octets[1] & 0xFE) != 18)
        && (octets[0] != 198 || octets[1] != 51 || octets[2] != 100)
        && (octets[0] != 203 || octets[1] != 0 || octets[2] != 113)
}

/// Checks if a paid amount is within slippage tolerance of the expected amount.
/// Delegates to pos-core-logic to avoid duplication.
pub fn is_payment_amount_valid(
    paid_usdc: f64,
    expected_usdc: f64,
    slippage_tolerance_pct: f64,
) -> bool {
    pos_core_logic::is_payment_amount_valid(paid_usdc, expected_usdc, slippage_tolerance_pct)
}
