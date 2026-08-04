use crate::{test_fail, test_pass};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use pos_backend::config::AppConfig;

static TEST_DB_PATH: &str = "data/test_pos_flow.db";

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
    println!("\n📦 POS Flow Tests (361-364)");
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
        let app = setup_app().await;

        test_361_order_no_price(&app).await;
        test_362_order_with_price(&app).await;
        test_363_order_empty(&app).await;
        test_364_order_zero_amount(&app).await;
    });

    let _ = std::fs::remove_file(TEST_DB_PATH);
    let _ = std::fs::remove_file(format!("{}-wal", TEST_DB_PATH));
    let _ = std::fs::remove_file(format!("{}-shm", TEST_DB_PATH));
}

async fn test_361_order_no_price(app: &axum::Router) {
    let req = Request::builder()
        .uri("/api/v1/pos/create-order")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"chat_id":1,"text":"coffee"}"#))
        .unwrap();
    let (status, body) = app_request(app, req).await;
    if status == StatusCode::OK && body.contains("prompt_price") {
        test_pass("361: order without price returns prompt_price");
    } else {
        test_fail(
            "361",
            &format!("status: {}, body: {}", status, &body[..100.min(body.len())]),
        );
    }
}

async fn test_362_order_with_price(app: &axum::Router) {
    let req = Request::builder()
        .uri("/api/v1/pos/create-order")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"chat_id":1,"text":"150 UAH"}"#))
        .unwrap();
    let (status, body) = app_request(app, req).await;
    if status == StatusCode::OK && body.contains("invoice_created") && body.contains("qr_url") {
        test_pass("362: order with price creates invoice + QR");
    } else {
        test_fail(
            "362",
            &format!("status: {}, body: {}", status, &body[..150.min(body.len())]),
        );
    }
}

async fn test_363_order_empty(app: &axum::Router) {
    let req = Request::builder()
        .uri("/api/v1/pos/create-order")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"chat_id":1,"text":""}"#))
        .unwrap();
    let (status, _) = app_request(app, req).await;
    if status == StatusCode::BAD_REQUEST {
        test_pass("363: empty order text returns 400");
    } else {
        test_fail("363", &format!("status: {}", status));
    }
}

async fn test_364_order_zero_amount(app: &axum::Router) {
    let req = Request::builder()
        .uri("/api/v1/pos/create-order")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"chat_id":1,"text":"0 UAH"}"#))
        .unwrap();
    let (status, body) = app_request(app, req).await;
    if status == StatusCode::OK && body.contains("prompt_price") {
        test_pass("364: zero amount returns prompt_price");
    } else {
        test_fail(
            "364",
            &format!("status: {}, body: {}", status, &body[..100.min(body.len())]),
        );
    }
}
