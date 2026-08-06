use std::env;

/// Application configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub manager_telegram_id: i64,
    pub telegram_bot_token: String,
    pub merchant_wallet_pubkey: String,
    pub solana_rpc_url: String,
    pub fallback_rpc_url: String,
    pub usdc_mint_address: String,
    pub nonce_account_pubkey: String,
    pub host: String,
    pub port: u16,
    pub db_path: String,
    /// Max requests per second per IP (default: 10).
    pub rate_limit_rps: u32,
    /// Telegram Bot API secret token for webhook auth.
    pub telegram_bot_secret_token: Option<String>,
    /// Telegram Webhook public URL (if set, webhook mode is enabled instead of long polling).
    pub telegram_webhook_url: Option<String>,
    /// API keys for external client auth (comma-separated in env).
    pub api_keys: Vec<String>,
    /// Quick receipt button amount (default: 200.0).
    pub quick_receipt_amount: f64,
    /// Quick receipt button currency (default: "UAH").
    pub quick_receipt_currency: String,
    /// Allow local HTTP RPC endpoints for dev testing (default: false).
    pub allow_local_rpc: bool,
    /// Stale update TTL threshold in seconds (default: 300, 0 = disabled).
    pub stale_update_ttl_secs: u64,
}

impl AppConfig {
    /// Load configuration from environment variables.
    pub fn from_env() -> Result<Self, anyhow::Error> {
        let port: u16 = env_or_default("PORT", env_or_default("POS_PORT", "8080").as_str())
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid PORT: {}", e))?;
        if port == 0 {
            anyhow::bail!("PORT must be between 1 and 65535");
        }

        let manager_telegram_id: i64 = env_or_default("MANAGER_TELEGRAM_ID", "0").parse()?;

        let merchant_wallet_pubkey = env_or_default(
            "MERCHANT_WALLET_PUBKEY",
            "8xAZmQ1111111111111111111111111111111111111",
        );
        if merchant_wallet_pubkey.len() < 32 || merchant_wallet_pubkey.len() > 44 {
            anyhow::bail!(
                "MERCHANT_WALLET_PUBKEY must be 32-44 characters (Base58), got {}",
                merchant_wallet_pubkey.len()
            );
        }

        let rate_limit_rps: u32 = env_or_default("RATE_LIMIT_RPS", "10").parse().unwrap_or(10);

        let mut telegram_bot_secret_token = env::var("TELEGRAM_BOT_SECRET_TOKEN").ok();
        let telegram_webhook_url = env::var("TELEGRAM_WEBHOOK_URL").ok();

        // Enforce secret token if Webhook mode is enabled to prevent endpoint spoofing
        if let Some(ref url) = telegram_webhook_url {
            if !url.trim().is_empty()
                && telegram_bot_secret_token
                    .as_deref()
                    .unwrap_or("")
                    .trim()
                    .is_empty()
            {
                let auto_secret =
                    format!("sec_{}", uuid::Uuid::new_v4().to_string().replace('-', ""));
                tracing::info!(
                    "Auto-generated secure TELEGRAM_BOT_SECRET_TOKEN for Webhook authentication"
                );
                telegram_bot_secret_token = Some(auto_secret);
            }
        }

        let api_keys: Vec<String> = env_or_default("API_KEYS", "")
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let quick_receipt_amount: f64 = env_or_default("QUICK_RECEIPT_AMOUNT", "200.0")
            .parse()
            .unwrap_or(200.0);
        let quick_receipt_currency = env_or_default("QUICK_RECEIPT_CURRENCY", "UAH");

        let allow_local_rpc = matches!(
            env_or_default("ALLOW_LOCAL_RPC", "false")
                .to_lowercase()
                .as_str(),
            "true" | "1" | "yes"
        );

        let stale_update_ttl_secs: u64 = env_or_default("STALE_UPDATE_TTL_SECS", "300")
            .parse()
            .unwrap_or(300);

        let telegram_bot_token = env_or_default("TELEGRAM_BOT_TOKEN", "");

        Ok(Self {
            manager_telegram_id,
            telegram_bot_token,
            merchant_wallet_pubkey,
            solana_rpc_url: env_or_default(
                "SOLANA_RPC_URL",
                "https://devnet.helius-rpc.com/?api-key=test",
            ),
            fallback_rpc_url: env_or_default(
                "SOLANA_FALLBACK_RPC_URL",
                "https://api.devnet.solana.com",
            ),
            usdc_mint_address: env_or_default(
                "USDC_MINT_ADDRESS",
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            ),
            nonce_account_pubkey: env_or_default("NONCE_ACCOUNT_PUBKEY", ""),
            host: env_or_default("HOST", env_or_default("POS_HOST", "0.0.0.0").as_str()),
            port,
            db_path: env_or_default("DB_PATH", "data/pos_store.db"),
            rate_limit_rps,
            telegram_bot_secret_token,
            telegram_webhook_url,
            api_keys,
            quick_receipt_amount,
            quick_receipt_currency,
            allow_local_rpc,
            stale_update_ttl_secs,
        })
    }
}

fn env_or_default(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}
