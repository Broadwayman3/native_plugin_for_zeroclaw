use crate::common::TempDbGuard;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use pos_backend::config::AppConfig;

fn test_config(guard: &TempDbGuard) -> AppConfig {
    AppConfig {
        manager_telegram_id: 12345,
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
fn test_347_health_check() {
    let guard = TempDbGuard::new("http_347");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let req = Request::builder()
            .uri("/healthz")
            .body(Body::empty())
            .unwrap();
        let (status, _) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::OK, "347: expected 200");
    });
}

#[test]
fn test_348_actions_spec() {
    let guard = TempDbGuard::new("http_348");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let req = Request::builder()
            .uri("/actions.json")
            .body(Body::empty())
            .unwrap();
        let (status, body) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::OK, "348: expected 200");
        assert!(body.contains("rules"), "348: should contain rules");
    });
}

#[test]
fn test_349_action_get() {
    let guard = TempDbGuard::new("http_349");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let req = Request::builder()
            .uri("/api/v1/actions/pay_invoice?invoice_id=INV-1")
            .body(Body::empty())
            .unwrap();
        let (status, body) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::OK, "349: expected 200");
        assert!(
            body.contains("Pay Invoice"),
            "349: should contain Pay Invoice"
        );
    });
}

#[test]
fn test_350_action_post_valid() {
    let guard = TempDbGuard::new("http_350");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let conn = pos_backend::db::get_db_connection(guard.path()).unwrap();
        let inv = pos_backend::db::invoices::CreateInvoiceRequest {
            id: "INV-BLINK".into(),
            reference_pubkey: "7xRefKey11111111111111111111111111111111111".into(),
            fiat_currency: Some("UAH".into()),
            fiat_amount: Some(200.0),
            usdc_amount: 5.0,
        };
        pos_backend::db::invoices::create_invoice(&conn, &inv).unwrap();

        let req = Request::builder()
            .uri("/api/v1/actions/pay_invoice?invoice_id=INV-BLINK")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"account":"8xAZmQ1111111111111111111111111111111111111"}"#,
            ))
            .unwrap();
        let (status, body) = app_request(&app, req).await;
        assert_eq!(
            status,
            StatusCode::OK,
            "350: expected 200 OK for implemented Blink POST action"
        );
        assert!(
            body.contains("transaction"),
            "350: response should contain transaction payload"
        );
    });
}

#[test]
fn test_351_action_post_invalid() {
    let guard = TempDbGuard::new("http_351");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let req = Request::builder()
            .uri("/api/v1/actions/pay_invoice")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"account":"short"}"#))
            .unwrap();
        let (status, _) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "351: expected 400");
    });
}

#[test]
fn test_366_x402_no_header() {
    let guard = TempDbGuard::new("http_366");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let req = Request::builder()
            .uri("/api/v1/sales/premium_analytics")
            .body(Body::empty())
            .unwrap();
        let (status, body) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::OK, "366: expected 200");
        assert!(
            body.contains("headers_required"),
            "366: should contain headers_required"
        );
    });
}

#[test]
fn test_367_x402_with_header() {
    let guard = TempDbGuard::new("http_367");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let req = Request::builder()
            .uri("/api/v1/sales/premium_analytics")
            .header("X-ACCEPT-PAYMENT", "x402")
            .body(Body::empty())
            .unwrap();
        let (status, body) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::PAYMENT_REQUIRED, "367: expected 402");
        assert!(body.contains("pay_url"), "367: should contain pay_url");
    });
}

#[test]
fn test_368_cors_preflight() {
    let guard = TempDbGuard::new("http_368");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
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
        assert!(
            status == StatusCode::OK || status == StatusCode::NO_CONTENT,
            "368: expected 200 or 204, got {}",
            status
        );
        assert!(
            headers.contains_key("access-control-allow-origin"),
            "368: missing CORS headers"
        );
    });
}

#[test]
fn test_369_payload_too_large() {
    let guard = TempDbGuard::new("http_369");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let large_body = "x".repeat(1_100_000);
        let req = Request::builder()
            .uri("/api/v1/pos/create-order")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(large_body))
            .unwrap();
        let (status, _) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE, "369: expected 413");
    });
}

#[test]
fn test_386_x402_response_has_solana_pay_url() {
    let guard = TempDbGuard::new("http_386");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let req = Request::builder()
            .uri("/api/v1/sales/premium_analytics")
            .header("X-ACCEPT-PAYMENT", "x402")
            .body(Body::empty())
            .unwrap();
        let (status, body) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::PAYMENT_REQUIRED, "386: expected 402");
        assert!(
            body.contains("solana:"),
            "386: should contain solana: pay_url"
        );
    });
}

#[test]
fn test_387_x402_response_has_spec_field() {
    let guard = TempDbGuard::new("http_387");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let req = Request::builder()
            .uri("/api/v1/sales/premium_analytics")
            .header("X-ACCEPT-PAYMENT", "x402")
            .body(Body::empty())
            .unwrap();
        let (status, body) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::PAYMENT_REQUIRED, "387: expected 402");
        assert!(body.contains("x402_spec"), "387: should contain x402_spec");
    });
}

#[test]
fn test_388_x402_recipient_matches_config() {
    let guard = TempDbGuard::new("http_388");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let req = Request::builder()
            .uri("/api/v1/sales/premium_analytics")
            .header("X-ACCEPT-PAYMENT", "x402")
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let headers = resp.headers().clone();
        let recipient = headers
            .get("X-PAYMENT-RECIPIENT")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert_eq!(
            recipient, "8xAZnR2pMQR3Qv5xK8c7mQ11rF4eG7hJ9kL2nP4s",
            "388: X-PAYMENT-RECIPIENT mismatch"
        );
    });
}

#[test]
fn test_389_x402_amount_is_numeric() {
    let guard = TempDbGuard::new("http_389");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let req = Request::builder()
            .uri("/api/v1/sales/premium_analytics")
            .header("X-ACCEPT-PAYMENT", "x402")
            .body(Body::empty())
            .unwrap();
        let (status, body) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::PAYMENT_REQUIRED, "389: expected 402");
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert!(
            json.get("amount_usdc").and_then(|v| v.as_f64()).is_some(),
            "389: amount_usdc should be numeric"
        );
    });
}
