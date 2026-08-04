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

    // Initialize database
    let conn = pos_backend::db::get_db_connection(&db_path)?;
    pos_backend::db::init_db(&conn, true)?;
    drop(conn);

    let app = pos_backend::api::build_router(&config).await;

    let host = config.host.clone();
    let port = config.port;
    let addr = std::net::SocketAddr::new(host.parse()?, port);

    tracing::info!(
        port = config.port,
        db_path = %config.db_path,
        rate_limit_rps = config.rate_limit_rps,
        "ZeroClaw POS Backend operational (WAL mode)"
    );

    // Port fallback logic with graceful shutdown
    let listener = tokio::net::TcpListener::bind(addr).await;
    let (listener, used_addr) = match listener {
        Ok(l) => (l, addr),
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
            let fallback_port = port + 1;
            tracing::warn!(
                port = port,
                fallback_port = fallback_port,
                "Port busy, retrying on fallback"
            );
            let fallback_addr = std::net::SocketAddr::new(config.host.parse()?, fallback_port);
            let l = tokio::net::TcpListener::bind(fallback_addr).await?;
            (l, fallback_addr)
        }
        Err(e) => return Err(e.into()),
    };

    tracing::info!(addr = %used_addr, "Server listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Server shutdown complete");
    Ok(())
}
