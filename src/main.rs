mod config;
mod infrastructure;
mod middleware;
mod modules;
mod observability;
mod openapi;

use modules::webhook::{ handlers::WebhookState, routes::webhook_routes };
use openapi::openapi_router;

use std::sync::Arc;
use axum::{ middleware as axum_middleware, routing::get, Router };
use std::net::SocketAddr;
use tokio::time::{ interval, Duration };
use tracing::{ error, info };

use infrastructure::blockchain::rpc_client::SolanaRpcClient;
use infrastructure::blockchain::transaction_watcher::TransactionWatcher;
use infrastructure::invoice_repository::PostgresInvoiceRepository;
use infrastructure::merchant_repository::PostgresMerchantRepository;
use infrastructure::payment_repository::PostgresPaymentRepository;
use infrastructure::settlement_repository::PostgresSettlementRepository;
use infrastructure::wallet_repository::PostgresWalletRepository;
use infrastructure::webhook_repository::PostgresWebhookRepository;
use middleware::auth::AuthState;
use middleware::request_id::request_id_middleware;
use modules::admin::audit::AuditLogger;
use modules::admin::routes::admin_routes;
use modules::admin::use_cases::AdminUseCase;
use modules::invoice::{ handlers::InvoiceState, routes::invoice_routes, use_cases::InvoiceUseCase };
use modules::merchant::{
    handlers::MerchantState,
    routes::merchant_routes,
    use_cases::MerchantUseCase,
};
use modules::payment::use_cases::PaymentUseCase;
use modules::settlement::use_cases::SettlementUseCase;
use modules::wallet::{ handlers::WalletState, routes::wallet_routes, use_cases::WalletUseCase };
use modules::webhook::use_cases::WebhookUseCase;
use observability::metrics::{ init_metrics, metrics_router };

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = config::AppConfig::load()?;

    // ─── Tracing ──────────────────────────────────────────────────────────────
    observability::tracing::init_tracing("crypto-payment-gateway", cfg.otlp_endpoint.as_deref());

    // ─── Metrics ──────────────────────────────────────────────────────────────
    init_metrics();

    info!(env = %cfg.app_env, "Starting crypto-payment-gateway");

    let db = infrastructure::database::connect(&cfg.database_url).await?;

    // ─── Repositories ─────────────────────────────────────────────────────────
    let merchant_repo = Arc::new(PostgresMerchantRepository::new(db.clone()));
    let wallet_repo = Arc::new(PostgresWalletRepository::new(db.clone()));
    let invoice_repo = Arc::new(PostgresInvoiceRepository::new(db.clone()));
    let payment_repo = Arc::new(PostgresPaymentRepository::new(db.clone()));
    let webhook_repo = Arc::new(PostgresWebhookRepository::new(db.clone()));
    let settlement_repo = Arc::new(PostgresSettlementRepository::new(db.clone()));

    // ─── Use Cases ────────────────────────────────────────────────────────────
    let merchant_use_case = MerchantUseCase::new(merchant_repo.clone());
    let wallet_use_case = WalletUseCase::new(wallet_repo.clone());
    let invoice_use_case = InvoiceUseCase::new(invoice_repo.clone());
    let webhook_use_case = Arc::new(WebhookUseCase::new(webhook_repo.clone()));
    let settlement_use_case = Arc::new(SettlementUseCase::new(settlement_repo.clone()));
    let payment_use_case = Arc::new(
        PaymentUseCase::new(
            payment_repo.clone(),
            invoice_repo.clone(),
            webhook_use_case.clone(),
            settlement_use_case.clone()
        )
    );

    // ─── Admin ────────────────────────────────────────────────────────────────
    let audit_logger = AuditLogger::new(db.clone());
    let admin_use_case = Arc::new(AdminUseCase {
        merchant_repo: merchant_repo.clone(),
        invoice_repo: invoice_repo.clone(),
        payment_repo: payment_repo.clone(),
        settlement_repo: settlement_repo.clone(),
        settlement_use_case: settlement_use_case.clone(),
        webhook_repo: webhook_repo.clone(),
        webhook_use_case: webhook_use_case.clone(),
        audit: audit_logger,
    });

    // ─── Auth State ───────────────────────────────────────────────────────────
    let auth_state = AuthState {
        merchant_repo: merchant_repo.clone(),
    };

    // ─── HTTP States ──────────────────────────────────────────────────────────
    let merchant_state = Arc::new(MerchantState { use_case: merchant_use_case });
    let wallet_state = Arc::new(WalletState { use_case: wallet_use_case });
    let invoice_state = Arc::new(InvoiceState { use_case: invoice_use_case });
    let webhook_state = Arc::new(WebhookState { use_case: webhook_use_case.clone() });

    // ─── Background Workers ───────────────────────────────────────────────────
    let rpc = SolanaRpcClient::new(cfg.solana_rpc_url.clone());
    let watcher = TransactionWatcher::new(
        rpc,
        invoice_repo.clone(),
        wallet_repo.clone(),
        payment_use_case,
        cfg.solana_usdc_mint.clone()
    );
    tokio::spawn(async move { watcher.run().await });
    info!("Transaction watcher spawned");

    let webhook_dispatcher = webhook_use_case.clone();
    tokio::spawn(async move {
        let mut tick = interval(Duration::from_secs(15));
        loop {
            tick.tick().await;
            if let Err(e) = webhook_dispatcher.dispatch_pending().await {
                error!(error = %e, "Webhook dispatcher error");
            }
        }
    });
    info!("Webhook dispatcher spawned");

    let settlement_processor = settlement_use_case.clone();
    tokio::spawn(async move {
        let mut tick = interval(Duration::from_secs(60));
        loop {
            tick.tick().await;
            match settlement_processor.process_pending().await {
                Ok(n) if n > 0 => info!(count = n, "Settlements processed"),
                Ok(_) => {}
                Err(e) => error!(error = %e, "Settlement processor error"),
            }
        }
    });
    info!("Settlement processor spawned");

    // ─── Invoice Expiry Job ───────────────────────────────────────────────────
    let invoice_expiry = Arc::new(InvoiceUseCase::new(invoice_repo.clone()));
    tokio::spawn(async move {
        let mut tick = interval(Duration::from_secs(60));
        loop {
            tick.tick().await;
            match invoice_expiry.expire_pending().await {
                Ok(n) if n > 0 => info!(count = n, "Invoices expired"),
                Ok(_) => {}
                Err(e) => error!(error = %e, "Invoice expiry error"),
            }
        }
    });
    info!("Invoice expiry job spawned");

    // ─── Protected API routes (require x-api-key) ─────────────────────────────
    let protected = Router::new()
        .merge(wallet_routes(wallet_state))
        .merge(invoice_routes(invoice_state))
        .layer(
            axum_middleware::from_fn_with_state(auth_state.clone(), middleware::auth::api_key_auth)
        );

    // ─── Router ───────────────────────────────────────────────────────────────
    let app = Router::new()
        .route("/health", get(health_handler))
        .merge(merchant_routes(merchant_state))
        .merge(protected)
        .merge(webhook_routes(webhook_state))
        .merge(admin_routes(admin_use_case))
        .merge(metrics_router())
        .merge(openapi_router())
        .layer(axum_middleware::from_fn(request_id_middleware));

    // ─── Graceful Shutdown ────────────────────────────────────────────────────
    let addr: SocketAddr = format!("{}:{}", cfg.app_host, cfg.app_port).parse()?;
    info!(%addr, "Server listening");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    let shutdown_timeout = Duration::from_secs(cfg.shutdown_timeout_secs);

    axum::serve(listener, app).with_graceful_shutdown(shutdown_signal(shutdown_timeout)).await?;

    observability::tracing::shutdown_tracer();
    info!("Server shutdown complete");

    Ok(())
}

async fn shutdown_signal(timeout: Duration) {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c().await.expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix
            ::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv().await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("Received Ctrl+C"),
        _ = terminate => info!("Received SIGTERM"),
    }

    info!(timeout_secs = timeout.as_secs(), "Shutdown signal received — draining connections");
    tokio::time::sleep(timeout).await;
}

async fn health_handler() -> axum::Json<serde_json::Value> {
    axum::Json(
        serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    })
    )
}
