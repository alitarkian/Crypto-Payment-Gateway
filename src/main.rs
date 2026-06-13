mod config;
mod infrastructure;
mod modules;

use std::sync::Arc;
use axum::{routing::get, Router};
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use infrastructure::merchant_repository::PostgresMerchantRepository;
use modules::merchant::{
    handlers::MerchantState,
    routes::merchant_routes,
    use_cases::MerchantUseCase,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = config::AppConfig::load()?;

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!(env = %cfg.app.env, "Starting crypto-payment-gateway");

    let db = infrastructure::database::connect(&cfg.database.url).await?;

    let merchant_repo = Arc::new(PostgresMerchantRepository::new(db));
    let merchant_use_case = MerchantUseCase::new(merchant_repo);
    let merchant_state = Arc::new(MerchantState { use_case: merchant_use_case });

    let app = Router::new()
        .route("/health", get(health_handler))
        .merge(merchant_routes(merchant_state));

    let addr: SocketAddr = format!("{}:{}", cfg.app.host, cfg.app.port).parse()?;

    info!(%addr, "Server listening");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_handler() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}