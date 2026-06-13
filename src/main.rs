mod config;
mod infrastructure;
mod modules;

use std::sync::Arc;
use axum::{routing::get, Router};
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use infrastructure::blockchain::rpc_client::SolanaRpcClient;
use infrastructure::blockchain::transaction_watcher::TransactionWatcher;
use infrastructure::invoice_repository::PostgresInvoiceRepository;
use infrastructure::merchant_repository::PostgresMerchantRepository;
use infrastructure::payment_repository::PostgresPaymentRepository;
use infrastructure::wallet_repository::PostgresWalletRepository;
use modules::invoice::{handlers::InvoiceState, routes::invoice_routes, use_cases::InvoiceUseCase};
use modules::merchant::{handlers::MerchantState, routes::merchant_routes, use_cases::MerchantUseCase};
use modules::payment::use_cases::PaymentUseCase;
use modules::wallet::{handlers::WalletState, routes::wallet_routes, use_cases::WalletUseCase};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = config::AppConfig::load()?;

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!(env = %cfg.app_env, "Starting crypto-payment-gateway");

    let db = infrastructure::database::connect(&cfg.database_url).await?;

    let merchant_repo = Arc::new(PostgresMerchantRepository::new(db.clone()));
    let merchant_use_case = MerchantUseCase::new(merchant_repo);
    let merchant_state = Arc::new(MerchantState { use_case: merchant_use_case });

    let wallet_repo = Arc::new(PostgresWalletRepository::new(db.clone()));
    let wallet_use_case = WalletUseCase::new(wallet_repo.clone());
    let wallet_state = Arc::new(WalletState { use_case: wallet_use_case });

    let invoice_repo = Arc::new(PostgresInvoiceRepository::new(db.clone()));
    let invoice_use_case = InvoiceUseCase::new(invoice_repo.clone());
    let invoice_state = Arc::new(InvoiceState { use_case: invoice_use_case });

    let payment_repo = Arc::new(PostgresPaymentRepository::new(db.clone()));
    let payment_use_case = Arc::new(PaymentUseCase::new(payment_repo, invoice_repo.clone()));

    let rpc = SolanaRpcClient::new(cfg.solana_rpc_url.clone());
    let watcher = TransactionWatcher::new(
        rpc,
        invoice_repo.clone(),
        wallet_repo.clone(),
        payment_use_case,
        cfg.solana_usdc_mint.clone(),
    );

    tokio::spawn(async move {
        watcher.run().await;
    });

    info!("Transaction watcher spawned");

    let app = Router::new()
        .route("/health", get(health_handler))
        .merge(merchant_routes(merchant_state))
        .merge(wallet_routes(wallet_state))
        .merge(invoice_routes(invoice_state));

    let addr: SocketAddr = format!("{}:{}", cfg.app_host, cfg.app_port).parse()?;
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