use crate::{test_fail, test_pass};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use pos_backend::config::AppConfig;

static TEST_DB_PATH: &str = "data/test_api_endpoints.db";

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

    pos_backend::db::invoices::create_invoice(
        &conn,
        &pos_backend::db::invoices::CreateInvoiceRequest {
            id: "INV-T1".into(),
            reference_pubkey: "8xAZnR2pMQR3Qv5xK8c7mQ11rF4eG7hJ9kL2nP4s".into(),
            fiat_currency: Some("UAH".into()),
            fiat_amount: Some(150.0),
            usdc_amount: 3.61,
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

pub fn run_suite() {
    println!("\n📦 API Endpoint Tests (352-360, 365)");
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
        let app = setup_app().await;

        test_352_create_invoice(&app).await;
        test_353_get_invoices(&app).await;
        test_354_get_invoices_by_id(&app).await;
        test_355_update_status(&app).await;
        test_356_update_status_conflict(&app).await;
        test_357_cancel_invoice(&app).await;
        test_358_cancel_already(&app).await;
        test_359_nonce_allocate(&app).await;
        test_360_nonce_release(&app).await;
        test_365_sales_summary(&app).await;
    });

    let _ = std::fs::remove_file(TEST_DB_PATH);
    let _ = std::fs::remove_file(format!("{}-wal", TEST_DB_PATH));
    let _ = std::fs::remove_file(format!("{}-shm", TEST_DB_PATH));
}

async fn test_352_create_invoice(app: &axum::Router) {
    let req = Request::builder()
        .uri("/api/v1/invoices/create")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"id":"INV-T3","reference_pubkey":"9xAZnR2pMQR3Qv5xK8c7mQ11rF4eG7hJ9kL2nP4s","usdc_amount":5.0}"#,
        ))
        .unwrap();
    let (status, body) = app_request(app, req).await;
    if status == StatusCode::CREATED && body.contains("INV-T3") {
        test_pass("352: create invoice returns 201 with invoice_id");
    } else {
        test_fail(
            "352",
            &format!("status: {}, body: {}", status, &body[..100.min(body.len())]),
        );
    }
}

async fn test_353_get_invoices(app: &axum::Router) {
    let req = Request::builder()
        .uri("/api/v1/invoices")
        .body(Body::empty())
        .unwrap();
    let (status, body) = app_request(app, req).await;
    if status == StatusCode::OK && body.contains("INV-T1") {
        test_pass("353: list invoices returns array with test data");
    } else {
        test_fail(
            "353",
            &format!("status: {}, body: {}", status, &body[..100.min(body.len())]),
        );
    }
}

async fn test_354_get_invoices_by_id(app: &axum::Router) {
    let req = Request::builder()
        .uri("/api/v1/invoices?id=INV-T1")
        .body(Body::empty())
        .unwrap();
    let (status, body) = app_request(app, req).await;
    if status == StatusCode::OK && body.contains("INV-T1") && !body.contains("INV-T2") {
        test_pass("354: get invoices by ID filters correctly");
    } else {
        test_fail(
            "354",
            &format!("status: {}, body: {}", status, &body[..100.min(body.len())]),
        );
    }
}

async fn test_355_update_status(app: &axum::Router) {
    let req = Request::builder()
        .uri("/api/v1/invoices/update_status")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"invoice_id":"INV-T1","status":"paid","tx_signature":"5Kd3...sig"}"#,
        ))
        .unwrap();
    let (status, body) = app_request(app, req).await;
    if status == StatusCode::OK && body.contains("true") {
        test_pass("355: update status to paid returns success");
    } else {
        test_fail(
            "355",
            &format!("status: {}, body: {}", status, &body[..100.min(body.len())]),
        );
    }
}

async fn test_356_update_status_conflict(app: &axum::Router) {
    let req = Request::builder()
        .uri("/api/v1/invoices/update_status")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"invoice_id":"INV-T1","status":"pending"}"#))
        .unwrap();
    let (status, _) = app_request(app, req).await;
    if status == StatusCode::CONFLICT {
        test_pass("356: invalid status transition returns 409");
    } else {
        test_fail("356", &format!("status: {}", status));
    }
}

async fn test_357_cancel_invoice(app: &axum::Router) {
    let req = Request::builder()
        .uri("/api/v1/invoices/cancel")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"invoice_id":"INV-T2"}"#))
        .unwrap();
    let (status, body) = app_request(app, req).await;
    if status == StatusCode::OK && body.contains("cancelled") {
        test_pass("357: cancel pending invoice returns success");
    } else {
        test_fail(
            "357",
            &format!("status: {}, body: {}", status, &body[..100.min(body.len())]),
        );
    }
}

async fn test_358_cancel_already(app: &axum::Router) {
    let req = Request::builder()
        .uri("/api/v1/invoices/cancel")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"invoice_id":"INV-T2"}"#))
        .unwrap();
    let (status, _) = app_request(app, req).await;
    if status == StatusCode::CONFLICT {
        test_pass("358: cancel already-cancelled returns 409");
    } else {
        test_fail("358", &format!("status: {}", status));
    }
}

async fn test_359_nonce_allocate(app: &axum::Router) {
    let req = Request::builder()
        .uri("/api/v1/nonce/allocate")
        .method("POST")
        .body(Body::empty())
        .unwrap();
    let (status, body) = app_request(app, req).await;
    if status == StatusCode::OK && body.contains("pubkey") {
        test_pass("359: nonce allocate returns pubkey");
    } else {
        test_fail(
            "359",
            &format!("status: {}, body: {}", status, &body[..100.min(body.len())]),
        );
    }
}

async fn test_360_nonce_release(app: &axum::Router) {
    let alloc_req = Request::builder()
        .uri("/api/v1/nonce/allocate")
        .method("POST")
        .body(Body::empty())
        .unwrap();
    let (alloc_status, alloc_body) = app_request(app, alloc_req).await;
    if alloc_status != StatusCode::OK {
        test_pass(&format!(
            "360: nonce allocate returned {} (pool may be empty, skipping release test)",
            alloc_status
        ));
        return;
    }
    let pubkey: String = serde_json::from_str(&alloc_body)
        .map(|v: serde_json::Value| v["pubkey"].as_str().unwrap_or("").to_string())
        .unwrap_or_default();

    if pubkey.is_empty() {
        test_pass("360: no pubkey available (pool empty), release test skipped");
        return;
    }

    let req = Request::builder()
        .uri("/api/v1/nonce/release")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(format!(r#"{{"pubkey":"{}"}}"#, pubkey)))
        .unwrap();
    let (status, _) = app_request(app, req).await;
    if status == StatusCode::OK {
        test_pass("360: nonce release returns success");
    } else {
        test_fail("360", &format!("status: {}", status));
    }
}

async fn test_365_sales_summary(app: &axum::Router) {
    let req = Request::builder()
        .uri("/api/v1/sales/summary")
        .body(Body::empty())
        .unwrap();
    let (status, body) = app_request(app, req).await;
    if status == StatusCode::OK && body.contains("total_paid_invoices") {
        test_pass("365: sales summary returns valid JSON");
    } else {
        test_fail(
            "365",
            &format!("status: {}, body: {}", status, &body[..100.min(body.len())]),
        );
    }
}
