use pos_backend::config::AppConfig;

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutdown signal received, starting graceful shutdown");
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = AppConfig::from_env()?;
    tracing::info!(
        port = config.port,
        db_path = %config.db_path,
        rate_limit_rps = config.rate_limit_rps,
        "Starting ZeroClaw POS Backend"
    );
    let db_path = config.db_path.clone();

    // Initialize database with WAL pragmas & migrations
    let conn = pos_backend::db::get_db_connection(&db_path)?;
    pos_backend::db::init_db(&conn, true)?;
    drop(conn);

    // Start background Telegram long-poller and Solana RPC verifier workers with single shared DB pool
    let db_pool = pos_backend::db::create_db_pool(&db_path).ok();
    let cancel_token = tokio_util::sync::CancellationToken::new();
    let telegram_handles = pos_backend::api::telegram::start_telegram_services(
        std::sync::Arc::new(config.clone()),
        db_pool,
        cancel_token.clone(),
    );

    let app = pos_backend::api::build_router(&config).await;

    let host = config.host.clone();
    let port = config.port;
    let addr = std::net::SocketAddr::new(host.parse()?, port);

    // Fail-fast TCP binding (prevent silent port drift in Docker / production)
    let listener = tokio::net::TcpListener::bind(addr).await.map_err(|e| {
        tracing::error!(port = port, error = %e, "Fail-Fast Error: TCP port is busy or inaccessible");
        e
    })?;

    tracing::info!(
        port = config.port,
        db_path = %config.db_path,
        rate_limit_rps = config.rate_limit_rps,
        "ZeroClaw POS Backend operational (WAL mode)"
    );
    tracing::info!(addr = %addr, "Server listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    // Graceful shutdown: signal workers to cancel and await completion within 16 seconds timeout
    cancel_token.cancel();
    if let Some(handles) = telegram_handles {
        handles.shutdown_with_timeout(16).await;
    }

    // Graceful shutdown: flush in-memory Telegram update offset to SQLite
    pos_backend::api::telegram::state::flush_offset_to_db(&db_path);

    tracing::info!("Server shutdown complete");
    Ok(())
}
