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
        rate_limit_rps: 50,
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
fn test_370_rate_limit_allows_normal_traffic() {
    let guard = TempDbGuard::new("rate_370");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let req = Request::builder()
            .uri("/healthz")
            .body(Body::empty())
            .unwrap();
        let (status, _) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::OK, "370: expected 200");
    });
}

#[test]
fn test_371_rate_limit_health_check_works() {
    let guard = TempDbGuard::new("rate_371");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        for _ in 0..3 {
            let req = Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .unwrap();
            let (status, _) = app_request(&app, req).await;
            assert_eq!(status, StatusCode::OK, "371: health check failed");
        }
    });
}

#[test]
fn test_372_rate_limit_get_invoices_works() {
    let guard = TempDbGuard::new("rate_372");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let req = Request::builder()
            .uri("/api/v1/invoices")
            .body(Body::empty())
            .unwrap();
        let (status, _) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::OK, "372: expected 200");
    });
}

#[test]
fn test_373_rate_limit_exceeds_returns_429() {
    let guard = TempDbGuard::new("rate_373");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let mut got_429 = false;
        for _ in 0..60 {
            let req = Request::builder()
                .uri("/api/v1/invoices")
                .body(Body::empty())
                .unwrap();
            let (status, _) = app_request(&app, req).await;
            if status == StatusCode::TOO_MANY_REQUESTS {
                got_429 = true;
                break;
            }
        }
        assert!(got_429, "373: no 429 received after 60 rapid requests");
    });
}

#[test]
fn test_375_rate_limit_burst_triggers_429() {
    let guard = TempDbGuard::new("rate_375");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let mut count_429 = 0;
        for _ in 0..100 {
            let req = Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .unwrap();
            let (status, _) = app_request(&app, req).await;
            if status == StatusCode::TOO_MANY_REQUESTS {
                count_429 += 1;
            }
        }
        assert!(
            count_429 > 0,
            "375: burst rate limit should trigger HTTP 429 response"
        );
    });
}
