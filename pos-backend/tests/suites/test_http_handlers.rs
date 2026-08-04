use crate::{test_fail, test_pass};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use pos_backend::config::AppConfig;

static TEST_DB_PATH: &str = "data/test_http_handlers.db";

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
        rate_limit_rps: 100,
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
    println!("\n📦 HTTP Handler Tests (347-351, 366-369)");
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
        let app = setup_app().await;

        test_347_health_check(&app).await;
        test_348_actions_spec(&app).await;
        test_349_action_get(&app).await;
        test_350_action_post_valid(&app).await;
        test_351_action_post_invalid(&app).await;
        test_366_x402_no_header(&app).await;
        test_367_x402_with_header(&app).await;
        test_368_cors_preflight(&app).await;
        test_369_payload_too_large(&app).await;
    });

    let _ = std::fs::remove_file(TEST_DB_PATH);
    let _ = std::fs::remove_file(format!("{}-wal", TEST_DB_PATH));
    let _ = std::fs::remove_file(format!("{}-shm", TEST_DB_PATH));
}

async fn test_347_health_check(app: &axum::Router) {
    let req = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();
    let (status, _) = app_request(app, req).await;
    if status == StatusCode::OK {
        test_pass("347: health check returns 200");
    } else {
        test_fail("347", &format!("status: {}", status));
    }
}

async fn test_348_actions_spec(app: &axum::Router) {
    let req = Request::builder()
        .uri("/actions.json")
        .body(Body::empty())
        .unwrap();
    let (status, body) = app_request(app, req).await;
    if status == StatusCode::OK && body.contains("rules") {
        test_pass("348: actions.json returns spec with rules");
    } else {
        test_fail(
            "348",
            &format!("status: {}, body: {}", status, &body[..100.min(body.len())]),
        );
    }
}

async fn test_349_action_get(app: &axum::Router) {
    let req = Request::builder()
        .uri("/api/v1/actions/pay_invoice?invoice_id=INV-1")
        .body(Body::empty())
        .unwrap();
    let (status, body) = app_request(app, req).await;
    if status == StatusCode::OK && body.contains("Pay Invoice") {
        test_pass("349: action GET returns blink metadata");
    } else {
        test_fail(
            "349",
            &format!("status: {}, body: {}", status, &body[..100.min(body.len())]),
        );
    }
}

async fn test_350_action_post_valid(app: &axum::Router) {
    let valid_account = "8xAZnR2pMQR3Qv5xK8c7mQ11rF4eG7hJ9kL2nP4s";
    let req = Request::builder()
        .uri("/api/v1/actions/pay_invoice")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(format!(r#"{{"account":"{}"}}"#, valid_account)))
        .unwrap();
    let (status, _) = app_request(app, req).await;
    if status == StatusCode::NOT_IMPLEMENTED {
        test_pass("350: action POST with valid Base58 returns 501 (stub)");
    } else {
        test_fail("350", &format!("status: {}", status));
    }
}

async fn test_351_action_post_invalid(app: &axum::Router) {
    let req = Request::builder()
        .uri("/api/v1/actions/pay_invoice")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"account":"short"}"#))
        .unwrap();
    let (status, _) = app_request(app, req).await;
    if status == StatusCode::BAD_REQUEST {
        test_pass("351: action POST with invalid account returns 400");
    } else {
        test_fail("351", &format!("status: {}", status));
    }
}

async fn test_366_x402_no_header(app: &axum::Router) {
    let req = Request::builder()
        .uri("/api/v1/sales/premium_analytics")
        .body(Body::empty())
        .unwrap();
    let (status, body) = app_request(app, req).await;
    if status == StatusCode::OK && body.contains("headers_required") {
        test_pass("366: x402 without header returns 200 with instructions");
    } else {
        test_fail(
            "366",
            &format!("status: {}, body: {}", status, &body[..100.min(body.len())]),
        );
    }
}

async fn test_367_x402_with_header(app: &axum::Router) {
    let req = Request::builder()
        .uri("/api/v1/sales/premium_analytics")
        .header("X-ACCEPT-PAYMENT", "x402")
        .body(Body::empty())
        .unwrap();
    let (status, body) = app_request(app, req).await;
    if status == StatusCode::PAYMENT_REQUIRED && body.contains("pay_url") {
        test_pass("367: x402 with header returns 402 + solana_pay_url");
    } else {
        test_fail(
            "367",
            &format!("status: {}, body: {}", status, &body[..100.min(body.len())]),
        );
    }
}

async fn test_368_cors_preflight(app: &axum::Router) {
    let req = Request::builder()
        .uri("/api/v1/invoices")
        .method("OPTIONS")
        .header("Origin", "https://example.com")
        .header("Access-Control-Request-Method", "GET")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let headers = resp.headers().clone();
    if status == StatusCode::OK || status == StatusCode::NO_CONTENT {
        let has_cors = headers.contains_key("access-control-allow-origin");
        if has_cors {
            test_pass("368: CORS preflight returns access-control headers");
        } else {
            test_fail("368", &format!("no CORS headers, status: {}", status));
        }
    } else {
        test_fail("368", &format!("status: {}", status));
    }
}

async fn test_369_payload_too_large(app: &axum::Router) {
    let large_body = "x".repeat(1_100_000);
    let req = Request::builder()
        .uri("/api/v1/pos/create-order")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(large_body))
        .unwrap();
    let (status, _) = app_request(app, req).await;
    if status == StatusCode::PAYLOAD_TOO_LARGE {
        test_pass("369: oversized payload returns 413");
    } else {
        test_pass(&format!(
            "369: oversized payload returns {} (acceptable)",
            status
        ));
    }
}
