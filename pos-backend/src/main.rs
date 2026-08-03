use pos_backend::config::AppConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = AppConfig::from_env()?;
    let db_path = config.db_path.clone();

    // Initialize database
    let conn = pos_backend::db::get_db_connection(&db_path)?;
    pos_backend::db::init_db(&conn, true)?;
    drop(conn);

    let app = pos_backend::api::build_router(&config).await;

    let host = config.host.clone();
    let port = config.port;
    let addr = std::net::SocketAddr::new(host.parse()?, port);

    println!("=================================================================");
    println!("🚀 ZeroClaw Solana POS REST API Backend Server (Rust)");
    println!("=================================================================");
    println!("• Status       : OPERATIONAL (WAL Mode)");
    println!("• Listening    : http://{}", addr);
    println!("• Database     : {}", db_path);
    println!("• Endpoints    : /actions.json, /api/v1/actions/pay_invoice,");
    println!("                 /api/v1/sales/summary, /api/v1/invoices,");
    println!("                 /api/v1/invoices/create, /api/v1/invoices/cancel,");
    println!("                 /api/v1/nonce/allocate, /api/v1/nonce/release");
    println!("=================================================================");

    // Port fallback logic
    match axum::serve(
        tokio::net::TcpListener::bind(addr).await?,
        app,
    )
    .await
    {
        Ok(_) => Ok(()),
        Err(e) if e.raw_os_error() == Some(98) => {
            let fallback_port = port + 1;
            eprintln!(
                "⚠️ [POS Server] Port {} is busy. Retrying on PORT={}...",
                port, fallback_port
            );
            let fallback_addr =
                std::net::SocketAddr::new(config.host.parse()?, fallback_port);
            let app = pos_backend::api::build_router(&config).await;
            axum::serve(
                tokio::net::TcpListener::bind(fallback_addr).await?,
                app,
            )
            .await?;
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}
