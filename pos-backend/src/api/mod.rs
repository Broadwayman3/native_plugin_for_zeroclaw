pub mod actions;
pub mod invoices;
pub mod middleware;
pub mod nonce;
pub mod pos_flow;
pub mod sales;
pub mod x402;

use axum::http::{header, Method, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::config::AppConfig;
use middleware::{AuthConfig, AuthLayer, ManagerLayer, RateLimitLayer};

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub http_client: reqwest::Client,
}

/// Builds the Axum router with all routes and middleware.
pub async fn build_router(config: &AppConfig) -> Router {
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .unwrap_or_default();

    let state = AppState {
        config: Arc::new(config.clone()),
        http_client,
    };

    // CORS configuration matching Python's router.py
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::OPTIONS,
            Method::PUT,
            Method::DELETE,
        ])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            "X-ACCEPT-PAYMENT".parse().unwrap(),
            "X-Telegram-Bot-Api-Secret-Token".parse().unwrap(),
            "X-Api-Key".parse().unwrap(),
            "X-Telegram-User-Id".parse().unwrap(),
            "Content-Encoding".parse().unwrap(),
            "Accept-Encoding".parse().unwrap(),
        ])
        .max_age(std::time::Duration::from_secs(86400));

    // Payload limit middleware (1MB)
    let payload_limit = tower_http::limit::RequestBodyLimitLayer::new(1_048_576);

    // Rate limiting
    let limiter = middleware::create_rate_limiter(config.rate_limit_rps);

    // Auth config for mutating routes
    let auth_config = AuthConfig {
        telegram_bot_secret_token: config.telegram_bot_secret_token.clone(),
        api_keys: config.api_keys.clone(),
        manager_telegram_id: if config.manager_telegram_id != 0 {
            Some(config.manager_telegram_id)
        } else {
            None
        },
    };

    // Manager-only routes (refund approve/reject, settings update)
    let manager_routes = Router::new()
        .route(
            "/api/v1/refund/approve",
            post(invoices::handle_refund_approve),
        )
        .route(
            "/api/v1/refund/reject",
            post(invoices::handle_refund_reject),
        )
        .route(
            "/api/v1/settings/update",
            post(invoices::handle_update_settings),
        )
        .layer(ManagerLayer::new(auth_config.manager_telegram_id))
        .layer(AuthLayer::new(auth_config.clone()));

    // Routes that require auth (mutating)
    let mutating_routes = Router::new()
        .route(
            "/api/v1/invoices/create",
            post(invoices::handle_create_invoice),
        )
        .route(
            "/api/v1/invoices/update_status",
            post(invoices::handle_update_invoice_status),
        )
        .route(
            "/api/v1/invoices/cancel",
            post(invoices::handle_cancel_invoice),
        )
        .route(
            "/api/v1/invoices/verify-transaction",
            post(invoices::handle_verify_transaction),
        )
        .route(
            "/api/v1/pos/create-order",
            post(pos_flow::handle_create_order),
        )
        .route("/api/v1/nonce/allocate", post(nonce::handle_nonce_allocate))
        .route("/api/v1/nonce/release", post(nonce::handle_nonce_release))
        .layer(AuthLayer::new(auth_config));

    // Read-only routes (no auth required)
    let read_routes = Router::new()
        .route("/healthz", get(health_check))
        .route("/actions.json", get(actions::handle_actions_spec))
        .route(
            "/api/v1/actions/pay_invoice",
            get(actions::handle_action_get).post(actions::handle_action_post),
        )
        .route("/api/v1/sales/summary", get(sales::handle_sales_summary))
        .route("/api/v1/invoices", get(invoices::handle_get_invoices))
        .route("/api/v1/settings", get(invoices::handle_get_settings))
        .route(
            "/api/v1/sales/premium_analytics",
            get(x402::handle_premium_analytics),
        );

    Router::new()
        .merge(manager_routes)
        .merge(mutating_routes)
        .merge(read_routes)
        .layer(RateLimitLayer::new(limiter))
        .layer(TraceLayer::new_for_http())
        .layer(payload_limit)
        .layer(cors)
        .with_state(state)
}

/// Health check endpoint.
pub async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}
