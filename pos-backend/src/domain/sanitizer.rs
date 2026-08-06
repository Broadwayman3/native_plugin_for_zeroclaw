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
static RPC_HOST_CACHE: Lazy<
    std::sync::Mutex<std::collections::HashMap<String, std::time::Instant>>,
> = Lazy::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

/// Transliterates Cyrillic/Greek homoglyphs that visually mimic ASCII letters to defeat jailbreaks.
pub fn transliterate_homoglyphs(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            'а' => 'a',
            'е' => 'e',
            'о' | 'ο' => 'o',
            'р' | 'ρ' => 'p',
            'с' => 'c',
            'х' | 'χ' => 'x',
            'у' => 'y',
            'і' | 'ι' => 'i',
            'А' => 'A',
            'Е' => 'E',
            'О' | 'Ο' => 'O',
            'Р' | 'Ρ' => 'P',
            'С' => 'C',
            'Х' | 'Χ' => 'X',
            'У' => 'Y',
            other => other,
        })
        .collect()
}

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

    // 3. Case-insensitive removal of prompt injection keywords (with homoglyph character index mapping)
    let homoglyph_safe = transliterate_homoglyphs(&cleaned);
    if RE_INJECTION.is_match(&homoglyph_safe) {
        let input_chars: Vec<char> = cleaned.chars().collect();
        let safe_chars: Vec<char> = homoglyph_safe.chars().collect();
        if input_chars.len() == safe_chars.len() {
            let mut keep: Vec<bool> = vec![true; input_chars.len()];
            for mat in RE_INJECTION.find_iter(&homoglyph_safe) {
                let start_idx = homoglyph_safe[..mat.start()].chars().count();
                let match_len = mat.as_str().chars().count();
                for i in start_idx..(start_idx + match_len) {
                    if i < keep.len() {
                        keep[i] = false;
                    }
                }
            }
            cleaned = input_chars
                .iter()
                .zip(keep.iter())
                .filter_map(|(&c, &k)| if k { Some(c) } else { None })
                .collect();
        } else {
            cleaned = RE_INJECTION.replace_all(&cleaned, "").to_string();
        }
    } else {
        cleaned = RE_INJECTION.replace_all(&cleaned, "").to_string();
    }

    // 4. Strip leading/trailing whitespace and limit length
    cleaned.trim().chars().take(max_length).collect()
}

/// Sanitizes command/ID arguments with a strict maximum limit of 100 characters.
pub fn sanitize_command_input(user_string: &str) -> String {
    sanitize_external_input(user_string, 100)
}

/// Sanitizes item titles/names with a maximum limit of 200 characters.
pub fn sanitize_item_name(user_string: &str) -> String {
    sanitize_external_input(user_string, 200)
}

/// Sanitizes payment descriptions or notes with a maximum limit of 1000 characters.
pub fn sanitize_description(user_string: &str) -> String {
    sanitize_external_input(user_string, 1000)
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
    validate_safe_rpc_url_with_config(url_str, false)
}

/// Evaluates Solana RPC URL to prevent SSRF attacks with optional local RPC dev toggle.
pub fn validate_safe_rpc_url_with_config(url_str: &str, allow_local_rpc: bool) -> bool {
    if url_str.is_empty() {
        return false;
    }

    if allow_local_rpc
        && (url_str.starts_with("http://127.0.0.1") || url_str.starts_with("http://localhost"))
    {
        return true;
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

    // Check if hostname is an IP address (strip brackets for IPv6 like [::1])
    let hostname_clean = hostname.trim_start_matches('[').trim_end_matches(']');
    if let Ok(ip) = hostname_clean.parse::<std::net::IpAddr>() {
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

        // Check in-memory 60s cache first
        if let Ok(cache) = RPC_HOST_CACHE.lock() {
            if let Some(timestamp) = cache.get(hostname) {
                if timestamp.elapsed() < std::time::Duration::from_secs(60) {
                    return true;
                }
            }
        }

        // DNS resolution check with timeout (500ms max)
        // Uses a background thread + mpsc recv_timeout for non-blocking resolution.
        // Returns immediately when DNS resolves, only waits up to 500ms if slow.
        use std::sync::mpsc;
        let (tx, rx) = mpsc::channel();
        let addr_str = format!("{}:443", hostname);
        std::thread::spawn(move || {
            let result = addr_str.to_socket_addrs().map(|mut addrs| addrs.next());
            let _ = tx.send(result);
        });
        match rx.recv_timeout(std::time::Duration::from_millis(500)) {
            Ok(Ok(Some(sock_addr))) => {
                let ip = sock_addr.ip();
                if ip.is_loopback() || ip.is_unspecified() {
                    return false;
                }
                if let std::net::IpAddr::V4(v4) = ip {
                    if v4.is_private() || is_reserved_v4(&v4) {
                        return false;
                    }
                }
                if let Ok(mut cache) = RPC_HOST_CACHE.lock() {
                    cache.insert(hostname.to_string(), std::time::Instant::now());
                }
            }
            _ => {
                tracing::warn!(
                    hostname = %hostname,
                    "SSRF Guard Warning: DNS resolution timed out (>500ms) or failed for RPC URL"
                );
                return false;
            }
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

/// Normalizes Telegram bot commands in group chats by stripping '@bot_username' suffixes.
/// E.g. "/start@my_pos_bot" -> "/start", "/refund@my_pos_bot INV-1 5" -> "/refund INV-1 5".
pub fn strip_bot_mention(text: &str) -> String {
    let trimmed = text.trim();
    if !trimmed.starts_with('/') {
        return trimmed.to_string();
    }

    let mut parts = trimmed.split_whitespace();
    if let Some(cmd) = parts.next() {
        let clean_cmd = if let Some(at_idx) = cmd.find('@') {
            &cmd[..at_idx]
        } else {
            cmd
        };

        let rest: Vec<&str> = parts.collect();
        if rest.is_empty() {
            clean_cmd.to_string()
        } else {
            format!("{} {}", clean_cmd, rest.join(" "))
        }
    } else {
        trimmed.to_string()
    }
}
