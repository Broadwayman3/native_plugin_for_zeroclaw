use crate::{test_fail, test_pass};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use pos_backend::config::AppConfig;

static TEST_DB_PATH: &str = "data/test_rate_limit.db";

fn test_config() -> AppConfig {
    let _ = std::fs::remove_file(TEST_DB_PATH);
    let _ = std::fs::remove_file(format!("{}-wal", TEST_DB_PATH));
    let _ = std::fs::remove_file(format!("{}-shm", TEST_DB_PATH));

    AppConfig {
        manager_telegram_id: 12345,
        merchant_wallet_pubkey: "8xAZnR2pMQR3Qv5xK8c7mQ11rF4eG7hJ9kL2nP4s".into(),
        solana_rpc_url: "https://api.mainnet.solana.com".into(),
        fallback_rpc_url: "https://api.mainnet.solana.com".into(),
        usdc_mint_address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".into(),
        nonce_account_pubkey: "8xAZnR2pMQR3Qv5xK8c7mQ11rF4eG7hJ9kL2nP4s".into(),
        host: "127.0.0.1".into(),
        port: 8080,
        db_path: TEST_DB_PATH.into(),
        rate_limit_rps: 50,
        telegram_bot_secret_token: None,
        api_keys: vec![],
    }
}

async fn setup_app() -> axum::Router {
    let config = test_config();
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

pub fn run_suite() {
    println!("\n📦 Rate Limit Tests (370-373)");
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
        let app = setup_app().await;

        test_370_rate_limit_allows_normal_traffic(&app).await;
        test_371_rate_limit_health_check_works(&app).await;
        test_372_rate_limit_get_invoices_works(&app).await;
        test_373_rate_limit_exceeds_returns_429(&app).await;
    });

    let _ = std::fs::remove_file(TEST_DB_PATH);
    let _ = std::fs::remove_file(format!("{}-wal", TEST_DB_PATH));
    let _ = std::fs::remove_file(format!("{}-shm", TEST_DB_PATH));
}

async fn test_370_rate_limit_allows_normal_traffic(app: &axum::Router) {
    let req = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();
    let (status, _) = app_request(app, req).await;
    if status == StatusCode::OK {
        test_pass("370: normal traffic is allowed through rate limiter");
    } else {
        test_fail("370", &format!("status: {}", status));
    }
}

async fn test_371_rate_limit_health_check_works(app: &axum::Router) {
    for _ in 0..3 {
        let req = Request::builder()
            .uri("/healthz")
            .body(Body::empty())
            .unwrap();
        let (status, _) = app_request(app, req).await;
        if status != StatusCode::OK {
            test_fail("371", &format!("health check failed at status: {}", status));
            return;
        }
    }
    test_pass("371: multiple health checks pass within rate limit");
}

async fn test_372_rate_limit_get_invoices_works(app: &axum::Router) {
    let req = Request::builder()
        .uri("/api/v1/invoices")
        .body(Body::empty())
        .unwrap();
    let (status, _) = app_request(app, req).await;
    if status == StatusCode::OK {
        test_pass("372: GET /api/v1/invoices works with rate limiting");
    } else {
        test_fail("372", &format!("status: {}", status));
    }
}

async fn test_373_rate_limit_exceeds_returns_429(app: &axum::Router) {
    // Send many requests quickly to exceed the rate limit
    let mut got_429 = false;
    for _ in 0..60 {
        let req = Request::builder()
            .uri("/api/v1/invoices")
            .body(Body::empty())
            .unwrap();
        let (status, _) = app_request(app, req).await;
        if status == StatusCode::TOO_MANY_REQUESTS {
            got_429 = true;
            break;
        }
    }
    if got_429 {
        test_pass("373: rate limit exceeded returns 429");
    } else {
        test_fail("373", "no 429 received after 60 rapid requests");
    }
}
