use crate::{test_fail, test_pass};
use std::env;

pub fn run_suite() {
    println!("\n📦 Config Tests (343-346)");
    test_343_config_from_env_defaults();
    test_344_config_from_env_custom();
    test_345_config_from_env_bad_port();
    test_346_config_from_env_bad_manager_id();
}

fn test_343_config_from_env_defaults() {
    let keys = [
        "MANAGER_TELEGRAM_ID",
        "MERCHANT_WALLET_PUBKEY",
        "SOLANA_RPC_URL",
        "SOLANA_FALLBACK_RPC_URL",
        "USDC_MINT_ADDRESS",
        "NONCE_ACCOUNT_PUBKEY",
        "HOST",
        "PORT",
        "DB_PATH",
    ];
    let saved: Vec<(String, Result<String, env::VarError>)> =
        keys.iter().map(|k| (k.to_string(), env::var(k))).collect();
    for k in &keys {
        env::remove_var(k);
    }
    let result = pos_backend::config::AppConfig::from_env();
    for (k, v) in &saved {
        match v {
            Ok(val) => env::set_var(k, val),
            Err(_) => env::remove_var(k),
        }
    }
    match result {
        Ok(cfg) => {
            if cfg.port == 8080 && cfg.host == "0.0.0.0" {
                test_pass("343: from_env() succeeds with defaults");
            } else {
                test_fail("343", &format!("port={} host={}", cfg.port, cfg.host));
            }
        }
        Err(e) => test_fail("343", &format!("Err: {}", e)),
    }
}

fn test_344_config_from_env_custom() {
    let keys = [
        "MANAGER_TELEGRAM_ID",
        "MERCHANT_WALLET_PUBKEY",
        "SOLANA_RPC_URL",
        "SOLANA_FALLBACK_RPC_URL",
        "USDC_MINT_ADDRESS",
        "NONCE_ACCOUNT_PUBKEY",
        "HOST",
        "PORT",
        "DB_PATH",
    ];
    let saved: Vec<(String, Result<String, env::VarError>)> =
        keys.iter().map(|k| (k.to_string(), env::var(k))).collect();
    for k in &keys {
        env::remove_var(k);
    }
    env::set_var("MANAGER_TELEGRAM_ID", "12345");
    let result = pos_backend::config::AppConfig::from_env();
    for (k, v) in &saved {
        match v {
            Ok(val) => env::set_var(k, val),
            Err(_) => env::remove_var(k),
        }
    }
    match result {
        Ok(cfg) => {
            if cfg.manager_telegram_id == 12345 {
                test_pass("344: MANAGER_TELEGRAM_ID=12345 parsed correctly");
            } else {
                test_fail(
                    "344",
                    &format!("manager_telegram_id = {}", cfg.manager_telegram_id),
                );
            }
        }
        Err(e) => test_fail("344", &format!("Err: {}", e)),
    }
}

fn test_345_config_from_env_bad_port() {
    let keys = [
        "MANAGER_TELEGRAM_ID",
        "MERCHANT_WALLET_PUBKEY",
        "SOLANA_RPC_URL",
        "SOLANA_FALLBACK_RPC_URL",
        "USDC_MINT_ADDRESS",
        "NONCE_ACCOUNT_PUBKEY",
        "HOST",
        "PORT",
        "DB_PATH",
    ];
    let saved: Vec<(String, Result<String, env::VarError>)> =
        keys.iter().map(|k| (k.to_string(), env::var(k))).collect();
    for k in &keys {
        env::remove_var(k);
    }
    env::set_var("PORT", "notanumber");
    let result = pos_backend::config::AppConfig::from_env();
    for (k, v) in &saved {
        match v {
            Ok(val) => env::set_var(k, val),
            Err(_) => env::remove_var(k),
        }
    }
    match result {
        Err(_) => test_pass("345: PORT=notanumber returns Err"),
        Ok(_) => test_fail("345", "expected Err, got Ok"),
    }
}

fn test_346_config_from_env_bad_manager_id() {
    let keys = [
        "MANAGER_TELEGRAM_ID",
        "MERCHANT_WALLET_PUBKEY",
        "SOLANA_RPC_URL",
        "SOLANA_FALLBACK_RPC_URL",
        "USDC_MINT_ADDRESS",
        "NONCE_ACCOUNT_PUBKEY",
        "HOST",
        "PORT",
        "DB_PATH",
    ];
    let saved: Vec<(String, Result<String, env::VarError>)> =
        keys.iter().map(|k| (k.to_string(), env::var(k))).collect();
    for k in &keys {
        env::remove_var(k);
    }
    env::set_var("MANAGER_TELEGRAM_ID", "notanumber");
    let result = pos_backend::config::AppConfig::from_env();
    for (k, v) in &saved {
        match v {
            Ok(val) => env::set_var(k, val),
            Err(_) => env::remove_var(k),
        }
    }
    match result {
        Err(_) => test_pass("346: MANAGER_TELEGRAM_ID=notanumber returns Err"),
        Ok(_) => test_fail("346", "expected Err, got Ok"),
    }
}
