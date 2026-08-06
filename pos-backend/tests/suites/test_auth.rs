use crate::common::TempDbGuard;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use pos_backend::config::AppConfig;

fn test_config(guard: &TempDbGuard) -> AppConfig {
    AppConfig {
        manager_telegram_id: 12345,
        telegram_bot_token: String::new(),
        merchant_wallet_pubkey: "8xAZnR2pMQR3Qv5xK8c7mQ11rF4eG7hJ9kL2nP4s".into(),
        solana_rpc_url: "https://api.mainnet.solana.com".into(),
        fallback_rpc_url: "https://api.mainnet.solana.com".into(),
        usdc_mint_address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".into(),
        nonce_account_pubkey: "8xAZnR2pMQR3Qv5xK8c7mQ11rF4eG7hJ9kL2nP4s".into(),
        host: "127.0.0.1".into(),
        port: 8080,
        db_path: guard.path().into(),
        rate_limit_rps: 100,
        telegram_bot_secret_token: Some("test-secret-token".to_string()),
        api_keys: vec!["test-api-key-1".to_string(), "test-api-key-2".to_string()],
        quick_receipt_amount: 200.0,
        quick_receipt_currency: "UAH".into(),
        allow_local_rpc: false,
    }
}

async fn setup_app(guard: &TempDbGuard) -> axum::Router {
    let config = test_config(guard);
    let conn = pos_backend::db::get_db_connection(&config.db_path).unwrap();
    pos_backend::db::schema::init_db(&conn, false).unwrap();
    drop(conn);
    pos_backend::api::build_router(&config).await
}

async fn app_request(app: &axum::Router, req: Request<Body>) -> (StatusCode, String) {
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let body = String::from_utf8_lossy(&body).to_string();
    (status, body)
}

#[test]
fn test_373_auth_no_header_rejected() {
    let guard = TempDbGuard::new("auth_373");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let req = Request::builder()
            .uri("/api/v1/pos/create-order")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"chat_id":1,"text":"150 UAH"}"#))
            .unwrap();
        let (status, _) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "373: expected 401");
    });
}

#[test]
fn test_374_auth_telegram_token_accepted() {
    let guard = TempDbGuard::new("auth_374");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let req = Request::builder()
            .uri("/api/v1/pos/create-order")
            .method("POST")
            .header("content-type", "application/json")
            .header("X-Telegram-Bot-Api-Secret-Token", "test-secret-token")
            .body(Body::from(r#"{"chat_id":1,"text":"coffee"}"#))
            .unwrap();
        let (status, _) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::OK, "374: expected 200");
    });
}

#[test]
fn test_375_auth_api_key_accepted() {
    let guard = TempDbGuard::new("auth_375");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let req = Request::builder()
            .uri("/api/v1/pos/create-order")
            .method("POST")
            .header("content-type", "application/json")
            .header("X-Api-Key", "test-api-key-1")
            .body(Body::from(r#"{"chat_id":1,"text":"coffee"}"#))
            .unwrap();
        let (status, _) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::OK, "375: expected 200");
    });
}

#[test]
fn test_376_auth_invalid_token_rejected() {
    let guard = TempDbGuard::new("auth_376");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let req = Request::builder()
            .uri("/api/v1/pos/create-order")
            .method("POST")
            .header("content-type", "application/json")
            .header("X-Telegram-Bot-Api-Secret-Token", "wrong-token")
            .body(Body::from(r#"{"chat_id":1,"text":"coffee"}"#))
            .unwrap();
        let (status, _) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "376: expected 401");
    });
}

#[test]
fn test_377_auth_invalid_api_key_rejected() {
    let guard = TempDbGuard::new("auth_377");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let req = Request::builder()
            .uri("/api/v1/pos/create-order")
            .method("POST")
            .header("content-type", "application/json")
            .header("X-Api-Key", "wrong-key")
            .body(Body::from(r#"{"chat_id":1,"text":"coffee"}"#))
            .unwrap();
        let (status, _) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "377: expected 401");
    });
}

#[test]
fn test_378_auth_read_route_no_auth_needed() {
    let guard = TempDbGuard::new("auth_378");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let req = Request::builder()
            .uri("/api/v1/invoices")
            .body(Body::empty())
            .unwrap();
        let (status, _) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::OK, "378: expected 200");
    });
}

#[test]
fn test_379_auth_health_check_no_auth() {
    let guard = TempDbGuard::new("auth_379");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let req = Request::builder()
            .uri("/healthz")
            .body(Body::empty())
            .unwrap();
        let (status, _) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::OK, "379: expected 200");
    });
}

#[test]
fn test_380_auth_create_invoice_telegram_token() {
    let guard = TempDbGuard::new("auth_380");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let req = Request::builder()
            .uri("/api/v1/invoices/create")
            .method("POST")
            .header("content-type", "application/json")
            .header("X-Telegram-Bot-Api-Secret-Token", "test-secret-token")
            .body(Body::from(
                r#"{"id":"INV-AUTH-TEST","reference_pubkey":"8xAZnR2pMQR3Qv5xK8c7mQ11rF4eG7hJ9kL2nP4s","usdc_amount":5.0}"#,
            ))
            .unwrap();
        let (status, body) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::CREATED, "380: expected 201");
        assert!(
            body.contains("INV-AUTH-TEST"),
            "380: response should contain invoice id"
        );
    });
}
