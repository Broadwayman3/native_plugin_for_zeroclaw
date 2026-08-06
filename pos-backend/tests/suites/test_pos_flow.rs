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
        telegram_bot_secret_token: None,
        telegram_webhook_url: None,
        api_keys: vec![],
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
fn test_361_order_no_price() {
    let guard = TempDbGuard::new("pos_361");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let req = Request::builder()
            .uri("/api/v1/pos/create-order")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"chat_id":1,"text":"coffee"}"#))
            .unwrap();
        let (status, body) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::OK, "361: expected 200, got {}", status);
        assert!(
            body.contains("prompt_price"),
            "361: expected prompt_price in body"
        );
    });
}

#[test]
fn test_362_order_with_price() {
    let guard = TempDbGuard::new("pos_362");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let req = Request::builder()
            .uri("/api/v1/pos/create-order")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"chat_id":1,"text":"150 UAH"}"#))
            .unwrap();
        let (status, body) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::OK, "362: expected 200, got {}", status);
        assert!(
            body.contains("invoice_created"),
            "362: expected invoice_created"
        );
        assert!(body.contains("qr_url"), "362: expected qr_url");
    });
}

#[test]
fn test_363_order_empty() {
    let guard = TempDbGuard::new("pos_363");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let req = Request::builder()
            .uri("/api/v1/pos/create-order")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"chat_id":1,"text":""}"#))
            .unwrap();
        let (status, _) = app_request(&app, req).await;
        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "363: expected 400, got {}",
            status
        );
    });
}

#[test]
fn test_364_order_zero_amount() {
    let guard = TempDbGuard::new("pos_364");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let req = Request::builder()
            .uri("/api/v1/pos/create-order")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"chat_id":1,"text":"0 UAH"}"#))
            .unwrap();
        let (status, body) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::OK, "364: expected 200, got {}", status);
        assert!(body.contains("prompt_price"), "364: expected prompt_price");
    });
}

#[test]
fn test_367_circuit_breaker_offline_warning() {
    let guard = TempDbGuard::new("pos_367");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let req = Request::builder()
            .uri("/api/v1/pos/create-order")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"chat_id":1,"text":"100 UAH"}"#))
            .unwrap();
        let (status, body) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::OK, "367: expected 200, got {}", status);
        assert!(
            body.contains("offline_warning"),
            "367: should include offline_warning key in response"
        );
    });
}

#[test]
fn test_368_prompt_price_includes_force_reply() {
    let guard = TempDbGuard::new("pos_368");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let req = Request::builder()
            .uri("/api/v1/pos/create-order")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"chat_id":1,"text":"Latte"}"#))
            .unwrap();
        let (status, body) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::OK, "368: expected 200, got {}", status);
        assert!(
            body.contains("force_reply") && body.contains("selective"),
            "368: response should contain force_reply and selective reply_markup"
        );
    });
}
