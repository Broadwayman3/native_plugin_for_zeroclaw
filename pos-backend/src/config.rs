use std::env;

/// Application configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub manager_telegram_id: i64,
    pub merchant_wallet_pubkey: String,
    pub solana_rpc_url: String,
    pub fallback_rpc_url: String,
    pub usdc_mint_address: String,
    pub nonce_account_pubkey: String,
    pub host: String,
    pub port: u16,
    pub db_path: String,
}

impl AppConfig {
    /// Load configuration from environment variables.
    pub fn from_env() -> Result<Self, anyhow::Error> {
        Ok(Self {
            manager_telegram_id: env_or_default("MANAGER_TELEGRAM_ID", "0").parse()?,
            merchant_wallet_pubkey: env_or_default(
                "MERCHANT_WALLET_PUBKEY",
                "8xAZmQ1111111111111111111111111111111111111",
            ),
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
            port: env_or_default("PORT", env_or_default("POS_PORT", "8080").as_str()).parse()?,
            db_path: env_or_default("DB_PATH", "data/pos_store.db"),
        })
    }
}

fn env_or_default(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}
