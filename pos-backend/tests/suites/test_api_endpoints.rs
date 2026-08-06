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
        stale_update_ttl_secs: 300,
    }
}

async fn setup_app(guard: &TempDbGuard) -> axum::Router {
    let config = test_config(guard);
    let conn = pos_backend::db::get_db_connection(&config.db_path).unwrap();
    pos_backend::db::schema::init_db(&conn, false).unwrap();

    pos_backend::db::invoices::create_invoice(
        &conn,
        &pos_backend::db::invoices::CreateInvoiceRequest {
            id: "INV-T1".into(),
            reference_pubkey: "8xAZnR2pMQR3Qv5xK8c7mQ11rF4eG7hJ9kL2nP4s".into(),
            fiat_currency: Some("UAH".into()),
            fiat_amount: Some(150.0),
            usdc_amount: 3.61,
            telegram_chat_id: None,
            telegram_msg_id: None,
        },
    )
    .unwrap();

    pos_backend::db::invoices::create_invoice(
        &conn,
        &pos_backend::db::invoices::CreateInvoiceRequest {
            id: "INV-T2".into(),
            reference_pubkey: "7xBYmQ1pNQR3Qv5xK8c7mQ11rF4eG7hJ9kL2nP4s".into(),
            fiat_currency: Some("USD".into()),
            fiat_amount: Some(10.0),
            usdc_amount: 10.0,
            telegram_chat_id: None,
            telegram_msg_id: None,
        },
    )
    .unwrap();

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
fn test_352_create_invoice() {
    let guard = TempDbGuard::new("api_352");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let req = Request::builder()
            .uri("/api/v1/invoices/create")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"id":"INV-T3","reference_pubkey":"9xAZnR2pMQR3Qv5xK8c7mQ11rF4eG7hJ9kL2nP4s","usdc_amount":5.0}"#,
            ))
            .unwrap();
        let (status, body) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::CREATED, "352: expected 201");
        assert!(
            body.contains("INV-T3"),
            "352: response should contain invoice id"
        );
    });
}

#[test]
fn test_353_get_invoices() {
    let guard = TempDbGuard::new("api_353");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let req = Request::builder()
            .uri("/api/v1/invoices")
            .body(Body::empty())
            .unwrap();
        let (status, body) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::OK, "353: expected 200");
        assert!(body.contains("INV-T1"), "353: should contain test data");
    });
}

#[test]
fn test_354_get_invoices_by_id() {
    let guard = TempDbGuard::new("api_354");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let req = Request::builder()
            .uri("/api/v1/invoices?id=INV-T1")
            .body(Body::empty())
            .unwrap();
        let (status, body) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::OK, "354: expected 200");
        assert!(body.contains("INV-T1"), "354: should contain INV-T1");
        assert!(!body.contains("INV-T2"), "354: should NOT contain INV-T2");
    });
}

#[test]
fn test_355_update_status() {
    let guard = TempDbGuard::new("api_355");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let req = Request::builder()
            .uri("/api/v1/invoices/update_status")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"invoice_id":"INV-T1","status":"paid","tx_signature":"5Kd3...sig"}"#,
            ))
            .unwrap();
        let (status, body) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::OK, "355: expected 200");
        assert!(body.contains("true"), "355: should return true");
    });
}

#[test]
fn test_356_update_status_conflict() {
    let guard = TempDbGuard::new("api_356");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        // First pay the invoice so we can test invalid transition
        let pay_req = Request::builder()
            .uri("/api/v1/invoices/update_status")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"invoice_id":"INV-T1","status":"paid","tx_signature":"sig"}"#,
            ))
            .unwrap();
        let _ = app_request(&app, pay_req).await;
        // Now try invalid transition: paid → pending
        let req = Request::builder()
            .uri("/api/v1/invoices/update_status")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"invoice_id":"INV-T1","status":"pending"}"#))
            .unwrap();
        let (status, _) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::CONFLICT, "356: expected 409");
    });
}

#[test]
fn test_357_cancel_invoice() {
    let guard = TempDbGuard::new("api_357");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let req = Request::builder()
            .uri("/api/v1/invoices/cancel")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"invoice_id":"INV-T2"}"#))
            .unwrap();
        let (status, body) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::OK, "357: expected 200");
        assert!(
            body.contains("cancelled"),
            "357: should contain 'cancelled'"
        );
    });
}

#[test]
fn test_358_cancel_already() {
    let guard = TempDbGuard::new("api_358");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        // First cancel
        let req = Request::builder()
            .uri("/api/v1/invoices/cancel")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"invoice_id":"INV-T2"}"#))
            .unwrap();
        let _ = app_request(&app, req).await;
        // Second cancel
        let req = Request::builder()
            .uri("/api/v1/invoices/cancel")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"invoice_id":"INV-T2"}"#))
            .unwrap();
        let (status, body) = app_request(&app, req).await;
        assert_eq!(
            status,
            StatusCode::OK,
            "358: expected 200 OK for idempotent cancel"
        );
        assert!(
            body.contains("\"already_cancelled\":true"),
            "358: expected already_cancelled to be true"
        );
    });
}

#[test]
fn test_359_nonce_allocate() {
    let guard = TempDbGuard::new("api_359");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let req = Request::builder()
            .uri("/api/v1/nonce/allocate")
            .method("POST")
            .body(Body::empty())
            .unwrap();
        let (status, body) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::OK, "359: expected 200");
        assert!(body.contains("pubkey"), "359: should contain pubkey");
    });
}

#[test]
fn test_360_nonce_release() {
    let guard = TempDbGuard::new("api_360");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let req = Request::builder()
            .uri("/api/v1/nonce/allocate")
            .method("POST")
            .body(Body::empty())
            .unwrap();
        let (alloc_status, alloc_body) = app_request(&app, req).await;
        if alloc_status != StatusCode::OK {
            return; // Pool empty, skip
        }
        let pubkey: String = serde_json::from_str(&alloc_body)
            .map(|v: serde_json::Value| v["pubkey"].as_str().unwrap_or("").to_string())
            .unwrap_or_default();
        if pubkey.is_empty() {
            return;
        }
        let req = Request::builder()
            .uri("/api/v1/nonce/release")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(format!(r#"{{"pubkey":"{}"}}"#, pubkey)))
            .unwrap();
        let (status, _) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::OK, "360: expected 200");
    });
}

#[test]
fn test_365_sales_summary() {
    let guard = TempDbGuard::new("api_365");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let req = Request::builder()
            .uri("/api/v1/sales/summary")
            .body(Body::empty())
            .unwrap();
        let (status, body) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::OK, "365: expected 200");
        assert!(
            body.contains("total_paid_invoices"),
            "365: should contain sales fields"
        );
    });
}

#[test]
fn test_366_verify_transaction_endpoint() {
    let guard = TempDbGuard::new("api_366");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;

        let payload = serde_json::json!({
            "invoice_id": "INV-T1",
            "tx_json": {
                "meta": { "err": null },
                "transaction": { "message": { "accountKeys": [] } }
            }
        });

        let req = Request::builder()
            .uri("/api/v1/invoices/verify-transaction")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let (status, body) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::OK, "366: expected 200");
        assert!(
            body.contains("is_valid"),
            "366: response should contain is_valid key"
        );
    });
}

#[test]
fn test_369_actions_post_payment_transaction() {
    let guard = TempDbGuard::new("api_369");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let conn = pos_backend::db::get_db_connection(guard.path()).unwrap();
        let inv = pos_backend::db::invoices::CreateInvoiceRequest {
            id: "INV-BLINK-369".into(),
            reference_pubkey: "7xRefKey11111111111111111111111111111111111".into(),
            fiat_currency: Some("UAH".into()),
            fiat_amount: Some(200.0),
            usdc_amount: 5.0,
            telegram_chat_id: None,
            telegram_msg_id: None,
        };
        pos_backend::db::invoices::create_invoice(&conn, &inv).unwrap();

        let payload = serde_json::json!({
            "account": "8xAZmQ1111111111111111111111111111111111111"
        });

        let req = Request::builder()
            .uri("/api/v1/actions/pay_invoice?invoice_id=INV-BLINK-369")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let (status, body) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::OK, "369: expected 200");
        assert!(
            body.contains("transaction"),
            "369: response should contain base64 transaction"
        );
    });
}
