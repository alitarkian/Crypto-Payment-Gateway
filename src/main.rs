mod config;
mod infrastructure;
mod middleware;
mod modules;
mod observability;
mod openapi;

use std::sync::Arc;
use axum::{ middleware as axum_middleware, routing::get, Router };
use std::net::SocketAddr;
use tokio::time::{ interval, Duration };
use tracing::{ error, info };

use infrastructure::blockchain::adapters::bsc::BscAdapter;
use infrastructure::blockchain::adapters::ethereum::EthereumAdapter;
use infrastructure::blockchain::adapters::solana::SolanaAdapter;
use infrastructure::blockchain::adapters::tron::TronAdapter;
use infrastructure::blockchain::ethereum_client::EthereumRpcClient;
use infrastructure::blockchain::multi_chain_watcher::MultiChainWatcher;
use infrastructure::blockchain::rpc_client::SolanaRpcClient;
use infrastructure::blockchain::tron_client::TronClient;
use infrastructure::invoice_repository::PostgresInvoiceRepository;
use infrastructure::merchant_repository::PostgresMerchantRepository;
use infrastructure::payment_repository::PostgresPaymentRepository;
use infrastructure::settlement_repository::PostgresSettlementRepository;
use infrastructure::vault::key_vault::WalletVault;
use infrastructure::wallet_key_repository::PostgresWalletKeyRepository;
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
use modules::wallet::{
    generate_use_case::GenerateWalletUseCase,
    handlers::WalletState,
    routes::wallet_routes,
    use_cases::WalletUseCase,
};
use modules::webhook::{ handlers::WebhookState, routes::webhook_routes, use_cases::WebhookUseCase };
use observability::metrics::{ init_metrics, metrics_router };
use openapi::openapi_router;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = config::AppConfig::load()?;

    // ─── Tracing ──────────────────────────────────────────────────────────────
    observability::tracing::init_tracing("crypto-payment-gateway", cfg.otlp_endpoint.as_deref());

    // ─── Metrics ──────────────────────────────────────────────────────────────
    init_metrics();

    info!(env = %cfg.app_env, "Starting crypto-payment-gateway");

    let db = infrastructure::database::connect(&cfg.database_url).await?;

    // ─── Vault ────────────────────────────────────────────────────────────────
    let vault = Arc::new(WalletVault::from_env().unwrap_or_else(|e| {
        // Warn and use a no-op placeholder in non-production environments.
        // In production, WALLET_MASTER_KEY must be set — startup will fail cleanly.
        tracing::warn!(error = %e, "WalletVault: WALLET_MASTER_KEY not set — managed wallet generation disabled");
        // Panic in production to prevent starting without key management
        if std::env::var("APP_ENV").as_deref() == Ok("production") {
            panic!("WALLET_MASTER_KEY must be set in production");
        }
        // Dev fallback: 32 zero bytes (insecure — dev only)
        // SAFETY: single-threaded startup, no other threads reading env yet
        unsafe { std::env::set_var("WALLET_MASTER_KEY", "0".repeat(64)); }
        WalletVault::from_env().expect("Dev fallback vault failed")
    }));

    // ─── Repositories ─────────────────────────────────────────────────────────
    let merchant_repo = Arc::new(PostgresMerchantRepository::new(db.clone()));
    let wallet_repo = Arc::new(PostgresWalletRepository::new(db.clone()));
    let wallet_key_repo = Arc::new(PostgresWalletKeyRepository::new(db.clone()));
    let invoice_repo = Arc::new(PostgresInvoiceRepository::new(db.clone()));
    let payment_repo = Arc::new(PostgresPaymentRepository::new(db.clone()));
    let webhook_repo = Arc::new(PostgresWebhookRepository::new(db.clone()));
    let settlement_repo = Arc::new(PostgresSettlementRepository::new(db.clone()));

    // ─── Use Cases ────────────────────────────────────────────────────────────
    let merchant_use_case = MerchantUseCase::new(merchant_repo.clone());
    let wallet_use_case = WalletUseCase::new(wallet_repo.clone());
    let generate_wallet_use_case = GenerateWalletUseCase::new(
        wallet_repo.clone(),
        wallet_key_repo.clone(),
        vault.clone(),
    );
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
    let wallet_state = Arc::new(WalletState {
        use_case: wallet_use_case,
        generate_use_case: generate_wallet_use_case,
    });
    let invoice_state = Arc::new(InvoiceState { use_case: invoice_use_case });
    let webhook_state = Arc::new(WebhookState { use_case: webhook_use_case.clone() });

    // ─── Multi-Chain Watcher ──────────────────────────────────────────────────
    let solana_adapter = Arc::new(
        SolanaAdapter::new(
            SolanaRpcClient::new(cfg.solana_rpc_url.clone()),
            cfg.solana_usdc_mint.clone()
        )
    );

    let mut adapters: Vec<Arc<dyn infrastructure::blockchain::chain_adapter::ChainAdapter>> = vec![
        solana_adapter
    ];

    if let (Some(eth_url), Some(eth_usdc)) = (
        cfg.ethereum_rpc_url.clone(),
        cfg.ethereum_usdc_contract.clone(),
    ) {
        let eth_adapter = Arc::new(EthereumAdapter::new(EthereumRpcClient::new(eth_url), eth_usdc));
        adapters.push(eth_adapter);
        info!("Ethereum adapter enabled");
    }

    if let (Some(bsc_url), Some(bsc_contract)) = (
        cfg.bsc_rpc_url.clone(),
        cfg.bsc_token_contract.clone(),
    ) {
        let symbol = cfg.bsc_token_symbol.clone().unwrap_or_else(|| "USDT".to_string());
        let bsc_adapter = Arc::new(BscAdapter::new(
            EthereumRpcClient::new(bsc_url),
            bsc_contract,
            symbol,
        ));
        adapters.push(bsc_adapter);
        info!("BSC adapter enabled");
    }

    if let (Some(tron_url), Some(tron_contract)) = (
        cfg.tron_rpc_url.clone(),
        cfg.tron_token_contract.clone(),
    ) {
        let symbol = cfg.tron_token_symbol.clone().unwrap_or_else(|| "USDT".to_string());
        let tron_adapter = Arc::new(TronAdapter::new(
            TronClient::new(tron_url),
            tron_contract,
            symbol,
        ));
        adapters.push(tron_adapter);
        info!("Tron adapter enabled");
    }

    let multi_watcher = MultiChainWatcher::new(
        adapters,
        invoice_repo.clone(),
        wallet_repo.clone(),
        payment_repo.clone(),
        payment_use_case,
    );

    tokio::spawn(async move { multi_watcher.run().await });
    info!("MultiChainWatcher spawned");

    // ─── Webhook Dispatcher ───────────────────────────────────────────────────
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

    // ─── Settlement Processor ─────────────────────────────────────────────────
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

    // ─── Protected Routes (require x-api-key) ─────────────────────────────────
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
