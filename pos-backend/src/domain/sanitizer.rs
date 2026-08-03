use regex::Regex;
use std::net::ToSocketAddrs;

/// Cleans external untrusted strings from control characters, system tags, and prompt injection patterns.
pub fn sanitize_external_input(user_string: &str, max_length: usize) -> String {
    if user_string.is_empty() {
        return String::new();
    }

    // 0. Unicode NFKC Normalization
    use unicode_normalization::UnicodeNormalization;
    let mut cleaned: String = user_string.nfkc().collect();

    // 1. Remove control characters, system tags, and line breaks
    let re_control = Regex::new(r"[\r\n\t\x00-\x1f\x7f-\x9f]").unwrap();
    cleaned = re_control.replace_all(&cleaned, " ").to_string();

    // 2. Strip invisible zero-width and directional Unicode characters
    let re_invisible =
        Regex::new(r"[\u{200B}-\u{200D}\u{FEFF}\u{202E}\u{00AD}\u{200E}\u{200F}\u{2060}]").unwrap();
    cleaned = re_invisible.replace_all(&cleaned, "").to_string();

    // 3. Case-insensitive removal of prompt injection keywords
    let re_injection =
        Regex::new(r"(?i)(system\s*:|override|ignore\s+previous|approve_refund|developer\s+mode)")
            .unwrap();
    cleaned = re_injection.replace_all(&cleaned, "").to_string();

    // 4. Strip leading/trailing whitespace and limit length
    cleaned.trim().chars().take(max_length).collect()
}

/// Redacts API keys, tokens, secrets, and Solana byte array keypairs from error messages.
pub fn redact_api_key(error_msg: &str) -> String {
    if error_msg.is_empty() {
        return String::new();
    }

    let re_api_key = Regex::new(r"(?i)(api[_-]?key|token|secret)=[^&\s]+").unwrap();
    let re_byte_array = Regex::new(r"\[\s*\d{1,3}\s*(?:,\s*\d{1,3}\s*){31,}\]").unwrap();

    let masked = re_api_key.replace_all(error_msg, "$1=REDACTED");
    re_byte_array
        .replace_all(&masked, "[REDACTED_BYTE_KEYPAIR]")
        .to_string()
}

/// Escapes special characters for Telegram MarkdownV2 format.
pub fn escape_telegram_markdown_v2(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }

    let escape_chars = r"_*[]()~`>#+-=|{}.!";
    let re = Regex::new(&format!(r"([{}])", regex::escape(escape_chars))).unwrap();

    re.replace_all(text, r"\$1").to_string()
}

/// Evaluates Solana RPC URL to prevent SSRF attacks.
pub fn validate_safe_rpc_url(url: &str) -> bool {
    if url.is_empty() {
        return false;
    }

    // Check scheme
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return false;
    }

    // Parse URL to extract hostname
    let parsed = match url::Url::parse(url) {
        Ok(u) => u,
        Err(_) => return false,
    };

    let hostname = match parsed.host_str() {
        Some(h) => h
            .to_lowercase()
            .trim_matches(|c| c == '[' || c == ']')
            .to_string(),
        None => return false,
    };

    // Reject localhost and known bad hostnames
    if hostname.is_empty() || hostname == "localhost" || hostname == "0.0.0.0" || hostname == "::1"
    {
        return false;
    }

    // Try parsing as IP address
    if let Ok(ip) = hostname.parse::<std::net::IpAddr>() {
        if ip.is_loopback() || ip.is_unspecified() {
            return false;
        }
        // Check for private/link-local ranges
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
                    match ip {
                        std::net::IpAddr::V4(v4) => {
                            if v4.is_private() || is_reserved_v4(&v4) || v4.is_link_local() {
                                return false;
                            }
                        }
                        std::net::IpAddr::V6(v6) => {
                            if v6.is_loopback() || v6.is_unspecified() {
                                return false;
                            }
                            let octets = v6.octets();
                            if octets[0] == 0xfe && (octets[1] & 0xc0) == 0x80 {
                                return false; // fe80::/10
                            }
                            if (octets[0] & 0xfe) == 0xfc {
                                return false; // fc00::/7
                            }
                        }
                    }
                }
            }
            Err(_) => return false, // Fail-closed: DNS failure = deny
        }
    }

    true
}

/// Checks if a payment amount is within slippage tolerance.
pub fn is_payment_amount_valid(
    paid_usdc: f64,
    expected_usdc: f64,
    slippage_tolerance_pct: f64,
) -> bool {
    paid_usdc >= (expected_usdc * (1.0 - (slippage_tolerance_pct / 100.0)))
}

/// Checks if an IPv4 address is in a reserved range.
/// Covers: 100.64.0.0/10, 192.0.0.0/24, 192.0.2.0/24, 198.18.0.0/15,
/// 198.51.100.0/24, 203.0.113.0/24, 240.0.0.0/4
fn is_reserved_v4(addr: &std::net::Ipv4Addr) -> bool {
    let octets = addr.octets();
    match octets {
        // 100.64.0.0/10 (Shared Address Space)
        [100, b, ..] if (64..=127).contains(&b) => true,
        // 192.0.0.0/24 (IETF Protocol Assignments)
        [192, 0, 0, _] => true,
        // 192.0.2.0/24 (TEST-NET-1)
        [192, 0, 2, _] => true,
        // 198.18.0.0/15 (Benchmarking)
        [198, b, ..] if b == 18 || b == 19 => true,
        // 198.51.100.0/24 (TEST-NET-2)
        [198, 51, 100, _] => true,
        // 203.0.113.0/24 (TEST-NET-3)
        [203, 0, 113, _] => true,
        // 240.0.0.0/4 (Reserved for future use)
        [b, ..] if b >= 240 => true,
        _ => false,
    }
}
