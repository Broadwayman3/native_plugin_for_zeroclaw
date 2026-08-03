pub mod actions;
pub mod invoices;
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

use crate::config::AppConfig;

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
}

/// Builds the Axum router with all routes and middleware.
pub async fn build_router(config: &AppConfig) -> Router {
    let state = AppState {
        config: Arc::new(config.clone()),
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
        ])
        .max_age(std::time::Duration::from_secs(86400));

    // Payload limit middleware (1MB)
    let payload_limit = tower_http::limit::RequestBodyLimitLayer::new(1_048_576);

    Router::new()
        // Actions/Blinks endpoints
        .route("/actions.json", get(actions::handle_actions_spec))
        .route(
            "/api/v1/actions/pay_invoice",
            get(actions::handle_action_get)
                .post(actions::handle_action_post),
        )
        // Sales endpoints
        .route("/api/v1/sales/summary", get(sales::handle_sales_summary))
        // Invoice endpoints
        .route(
            "/api/v1/invoices",
            get(invoices::handle_get_invoices),
        )
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
        // Nonce endpoints
        .route(
            "/api/v1/nonce/allocate",
            post(nonce::handle_nonce_allocate),
        )
        .route(
            "/api/v1/nonce/release",
            post(nonce::handle_nonce_release),
        )
        // POS flow (create from order - replaces handle_text_message)
        .route(
            "/api/v1/pos/create-order",
            post(pos_flow::handle_create_order),
        )
        // x402 endpoint
        .route(
            "/api/v1/sales/premium_analytics",
            get(x402::handle_premium_analytics),
        )
        .layer(cors)
        .layer(payload_limit)
        .with_state(state)
}

/// Health check endpoint.
pub async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}
