use crate::{test_fail, test_pass};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use pos_backend::config::AppConfig;

static TEST_DB_PATH: &str = "data/test_auth.db";

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
        telegram_bot_secret_token: Some("test-secret-token".to_string()),
        api_keys: vec!["test-api-key-1".to_string(), "test-api-key-2".to_string()],
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
    println!("\n📦 Auth Tests (373-380)");
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
        let app = setup_app().await;

        test_373_auth_no_header_rejected(&app).await;
        test_374_auth_telegram_token_accepted(&app).await;
        test_375_auth_api_key_accepted(&app).await;
        test_376_auth_invalid_token_rejected(&app).await;
        test_377_auth_invalid_api_key_rejected(&app).await;
        test_378_auth_read_route_no_auth_needed(&app).await;
        test_379_auth_health_check_no_auth(&app).await;
        test_380_auth_create_invoice_telegram_token(&app).await;
    });

    let _ = std::fs::remove_file(TEST_DB_PATH);
    let _ = std::fs::remove_file(format!("{}-wal", TEST_DB_PATH));
    let _ = std::fs::remove_file(format!("{}-shm", TEST_DB_PATH));
}

async fn test_373_auth_no_header_rejected(app: &axum::Router) {
    let req = Request::builder()
        .uri("/api/v1/pos/create-order")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"chat_id":1,"text":"150 UAH"}"#))
        .unwrap();
    let (status, _) = app_request(app, req).await;
    if status == StatusCode::UNAUTHORIZED {
        test_pass("373: mutating route without auth returns 401");
    } else {
        test_fail("373", &format!("status: {}", status));
    }
}

async fn test_374_auth_telegram_token_accepted(app: &axum::Router) {
    let req = Request::builder()
        .uri("/api/v1/pos/create-order")
        .method("POST")
        .header("content-type", "application/json")
        .header("X-Telegram-Bot-Api-Secret-Token", "test-secret-token")
        .body(Body::from(r#"{"chat_id":1,"text":"coffee"}"#))
        .unwrap();
    let (status, _) = app_request(app, req).await;
    if status == StatusCode::OK {
        test_pass("374: Telegram secret token accepted");
    } else {
        test_fail("374", &format!("status: {}", status));
    }
}

async fn test_375_auth_api_key_accepted(app: &axum::Router) {
    let req = Request::builder()
        .uri("/api/v1/pos/create-order")
        .method("POST")
        .header("content-type", "application/json")
        .header("X-Api-Key", "test-api-key-1")
        .body(Body::from(r#"{"chat_id":1,"text":"coffee"}"#))
        .unwrap();
    let (status, _) = app_request(app, req).await;
    if status == StatusCode::OK {
        test_pass("375: API key accepted");
    } else {
        test_fail("375", &format!("status: {}", status));
    }
}

async fn test_376_auth_invalid_token_rejected(app: &axum::Router) {
    let req = Request::builder()
        .uri("/api/v1/pos/create-order")
        .method("POST")
        .header("content-type", "application/json")
        .header("X-Telegram-Bot-Api-Secret-Token", "wrong-token")
        .body(Body::from(r#"{"chat_id":1,"text":"coffee"}"#))
        .unwrap();
    let (status, _) = app_request(app, req).await;
    if status == StatusCode::UNAUTHORIZED {
        test_pass("376: invalid Telegram token rejected");
    } else {
        test_fail("376", &format!("status: {}", status));
    }
}

async fn test_377_auth_invalid_api_key_rejected(app: &axum::Router) {
    let req = Request::builder()
        .uri("/api/v1/pos/create-order")
        .method("POST")
        .header("content-type", "application/json")
        .header("X-Api-Key", "wrong-key")
        .body(Body::from(r#"{"chat_id":1,"text":"coffee"}"#))
        .unwrap();
    let (status, _) = app_request(app, req).await;
    if status == StatusCode::UNAUTHORIZED {
        test_pass("377: invalid API key rejected");
    } else {
        test_fail("377", &format!("status: {}", status));
    }
}

async fn test_378_auth_read_route_no_auth_needed(app: &axum::Router) {
    let req = Request::builder()
        .uri("/api/v1/invoices")
        .body(Body::empty())
        .unwrap();
    let (status, _) = app_request(app, req).await;
    if status == StatusCode::OK {
        test_pass("378: read-only route works without auth");
    } else {
        test_fail("378", &format!("status: {}", status));
    }
}

async fn test_379_auth_health_check_no_auth(app: &axum::Router) {
    let req = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();
    let (status, _) = app_request(app, req).await;
    if status == StatusCode::OK {
        test_pass("379: health check works without auth");
    } else {
        test_fail("379", &format!("status: {}", status));
    }
}

async fn test_380_auth_create_invoice_telegram_token(app: &axum::Router) {
    let req = Request::builder()
        .uri("/api/v1/invoices/create")
        .method("POST")
        .header("content-type", "application/json")
        .header("X-Telegram-Bot-Api-Secret-Token", "test-secret-token")
        .body(Body::from(
            r#"{"id":"INV-AUTH-TEST","reference_pubkey":"8xAZnR2pMQR3Qv5xK8c7mQ11rF4eG7hJ9kL2nP4s","usdc_amount":5.0}"#,
        ))
        .unwrap();
    let (status, body) = app_request(app, req).await;
    if status == StatusCode::CREATED && body.contains("INV-AUTH-TEST") {
        test_pass("380: create invoice with Telegram token succeeds");
    } else {
        test_fail(
            "380",
            &format!("status: {}, body: {}", status, &body[..100.min(body.len())]),
        );
    }
}
