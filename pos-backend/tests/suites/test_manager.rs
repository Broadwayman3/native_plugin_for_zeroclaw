use crate::common::TempDbGuard;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use pos_backend::config::AppConfig;

fn test_config(guard: &TempDbGuard) -> AppConfig {
    AppConfig {
        manager_telegram_id: 99999,
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
        telegram_bot_secret_token: Some("test-secret".into()),
        telegram_webhook_url: None,
        api_keys: vec!["test-key".into()],
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
fn test_manager_approve_no_auth() {
    let guard = TempDbGuard::new("manager_no_auth");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let req = Request::builder()
            .uri("/api/v1/refund/approve")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"invoice_id":"INV-1"}"#))
            .unwrap();
        let (status, _) = app_request(&app, req).await;
        assert_eq!(
            status,
            StatusCode::UNAUTHORIZED,
            "expected 401 without auth"
        );
    });
}

#[test]
fn test_manager_approve_wrong_user() {
    let guard = TempDbGuard::new("manager_wrong_user");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let req = Request::builder()
            .uri("/api/v1/refund/approve")
            .method("POST")
            .header("content-type", "application/json")
            .header("X-Telegram-Bot-Api-Secret-Token", "test-secret")
            .header("X-Telegram-User-Id", "12345")
            .body(Body::from(r#"{"invoice_id":"INV-1"}"#))
            .unwrap();
        let (status, _) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "expected 403 for wrong user");
    });
}

#[test]
fn test_manager_approve_correct_user() {
    let guard = TempDbGuard::new("manager_approve");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let conn = pos_backend::db::get_db_connection(guard.path()).unwrap();
        pos_backend::db::invoices::create_invoice(
            &conn,
            &pos_backend::db::invoices::CreateInvoiceRequest {
                id: "INV-MGR-1".into(),
                reference_pubkey: "ref_mgr_1".into(),
                fiat_currency: Some("USD".into()),
                fiat_amount: Some(10.0),
                usdc_amount: 10.0,
                telegram_chat_id: None,
                telegram_msg_id: None,
            },
        )
        .unwrap();
        pos_backend::db::invoices::update_invoice_status(&conn, "INV-MGR-1", "paid", Some("sig"))
            .unwrap();
        drop(conn);

        let req = Request::builder()
            .uri("/api/v1/refund/approve")
            .method("POST")
            .header("content-type", "application/json")
            .header("X-Telegram-Bot-Api-Secret-Token", "test-secret")
            .header("X-Telegram-User-Id", "99999")
            .body(Body::from(r#"{"invoice_id":"INV-MGR-1"}"#))
            .unwrap();
        let (status, body) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::OK, "expected 200 for manager");
        assert!(
            body.contains("refund_proposed_squads_v4"),
            "should contain status"
        );
    });
}

#[test]
fn test_manager_reject_correct_user() {
    let guard = TempDbGuard::new("manager_reject");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;
        let conn = pos_backend::db::get_db_connection(guard.path()).unwrap();
        pos_backend::db::invoices::create_invoice(
            &conn,
            &pos_backend::db::invoices::CreateInvoiceRequest {
                id: "INV-MGR-2".into(),
                reference_pubkey: "ref_mgr_2".into(),
                fiat_currency: Some("USD".into()),
                fiat_amount: Some(10.0),
                usdc_amount: 10.0,
                telegram_chat_id: None,
                telegram_msg_id: None,
            },
        )
        .unwrap();
        pos_backend::db::invoices::update_invoice_status(&conn, "INV-MGR-2", "paid", Some("sig"))
            .unwrap();
        pos_backend::db::invoices::initiate_refund(&conn, "INV-MGR-2").unwrap();
        drop(conn);

        let req = Request::builder()
            .uri("/api/v1/refund/reject")
            .method("POST")
            .header("content-type", "application/json")
            .header("X-Telegram-Bot-Api-Secret-Token", "test-secret")
            .header("X-Telegram-User-Id", "99999")
            .body(Body::from(r#"{"invoice_id":"INV-MGR-2"}"#))
            .unwrap();
        let (status, body) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::OK, "expected 200 for reject");
        assert!(
            body.contains("\"status\":\"paid\""),
            "should revert to paid"
        );
    });
}

#[test]
fn test_manager_update_settings() {
    let guard = TempDbGuard::new("manager_settings");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = setup_app(&guard).await;

        let req = Request::builder()
            .uri("/api/v1/settings/update")
            .method("POST")
            .header("content-type", "application/json")
            .header("X-Telegram-Bot-Api-Secret-Token", "test-secret")
            .header("X-Telegram-User-Id", "99999")
            .body(Body::from(
                r#"{"quick_receipt_amount":250.0,"quick_receipt_currency":"UAH"}"#,
            ))
            .unwrap();
        let (status, body) = app_request(&app, req).await;
        assert_eq!(status, StatusCode::OK, "expected 200 for settings update");
        assert!(
            body.contains("250"),
            "should reflect updated quick receipt amount"
        );

        let req_get = Request::builder()
            .uri("/api/v1/settings")
            .method("GET")
            .body(Body::empty())
            .unwrap();
        let (status_get, body_get) = app_request(&app, req_get).await;
        assert_eq!(status_get, StatusCode::OK, "expected 200 for get settings");
        assert!(
            body_get.contains("250"),
            "GET /api/v1/settings should return updated amount"
        );
    });
}
