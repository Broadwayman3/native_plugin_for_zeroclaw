use crate::common;
use once_cell::sync::Lazy;
use std::env;
use std::sync::Mutex;

static ENV_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

const CONFIG_KEYS: &[&str] = &[
    "MANAGER_TELEGRAM_ID",
    "MERCHANT_WALLET_PUBKEY",
    "SOLANA_RPC_URL",
    "SOLANA_FALLBACK_RPC_URL",
    "USDC_MINT_ADDRESS",
    "NONCE_ACCOUNT_PUBKEY",
    "HOST",
    "PORT",
    "DB_PATH",
    "RATE_LIMIT_RPS",
    "TELEGRAM_BOT_SECRET_TOKEN",
    "API_KEYS",
];

#[test]
fn test_343_config_from_env_defaults() {
    let _guard = ENV_MUTEX.lock().unwrap();
    let saved = common::save_and_clear_env(CONFIG_KEYS);
    let result = pos_backend::config::AppConfig::from_env();
    common::restore_env(&saved);

    let cfg = result.expect("343: from_env() should succeed with defaults");
    assert_eq!(cfg.port, 8080, "343: default port should be 8080");
    assert_eq!(cfg.host, "0.0.0.0", "343: default host should be 0.0.0.0");
}

#[test]
fn test_344_config_from_env_custom() {
    let _guard = ENV_MUTEX.lock().unwrap();
    let saved = common::save_and_clear_env(CONFIG_KEYS);
    env::set_var("MANAGER_TELEGRAM_ID", "12345");
    let result = pos_backend::config::AppConfig::from_env();
    common::restore_env(&saved);

    let cfg = result.expect("344: from_env() should succeed");
    assert_eq!(
        cfg.manager_telegram_id, 12345,
        "344: MANAGER_TELEGRAM_ID should be 12345"
    );
}

#[test]
fn test_345_config_from_env_bad_port() {
    let _guard = ENV_MUTEX.lock().unwrap();
    let saved = common::save_and_clear_env(CONFIG_KEYS);
    env::set_var("PORT", "notanumber");
    let result = pos_backend::config::AppConfig::from_env();
    common::restore_env(&saved);

    assert!(result.is_err(), "345: PORT=notanumber should return Err");
}

#[test]
fn test_346_config_from_env_bad_manager_id() {
    let _guard = ENV_MUTEX.lock().unwrap();
    let saved = common::save_and_clear_env(CONFIG_KEYS);
    env::set_var("MANAGER_TELEGRAM_ID", "notanumber");
    let result = pos_backend::config::AppConfig::from_env();
    common::restore_env(&saved);

    assert!(
        result.is_err(),
        "346: MANAGER_TELEGRAM_ID=notanumber should return Err"
    );
}

#[test]
fn test_373_config_defaults_when_vars_missing() {
    let _guard = ENV_MUTEX.lock().unwrap();
    let subset_keys = &[
        "MANAGER_TELEGRAM_ID",
        "MERCHANT_WALLET_PUBKEY",
        "SOLANA_RPC_URL",
        "SOLANA_FALLBACK_RPC_URL",
        "USDC_MINT_ADDRESS",
        "DB_PATH",
    ];
    let saved = common::save_and_clear_env(subset_keys);
    env::set_var("MANAGER_TELEGRAM_ID", "42");
    let result = pos_backend::config::AppConfig::from_env();
    common::restore_env(&saved);

    let config = result.expect("373: from_env() should succeed");
    assert_eq!(config.db_path, "data/pos_store.db", "373: default db_path");
    assert!(
        config.solana_rpc_url.contains("helius"),
        "373: default RPC should contain helius"
    );
    assert!(
        config.fallback_rpc_url.contains("devnet"),
        "373: fallback RPC should contain devnet"
    );
}

#[test]
fn test_374_config_port_fallback() {
    let _guard = ENV_MUTEX.lock().unwrap();
    let keys = ["PORT", "POS_PORT"];
    let saved = common::save_and_clear_env(&keys);
    env::set_var("MANAGER_TELEGRAM_ID", "42");
    env::set_var("POS_PORT", "9090");
    let result = pos_backend::config::AppConfig::from_env();
    common::restore_env(&saved);

    let config = result.expect("374: from_env() should succeed");
    assert_eq!(
        config.port, 9090,
        "374: POS_PORT should be used as fallback"
    );
}
