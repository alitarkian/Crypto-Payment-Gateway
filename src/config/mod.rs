use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub app_env: String,
    pub app_host: String,
    pub app_port: u16,
    pub database_url: String,
    #[allow(dead_code)]
    pub redis_url: String,
    pub solana_rpc_url: String,
    pub solana_usdc_mint: String,
    #[serde(default)]
    pub otlp_endpoint: Option<String>,
    #[serde(default = "default_shutdown_timeout")]
    pub shutdown_timeout_secs: u64,
    #[serde(default)]
    pub ethereum_rpc_url: Option<String>,
    #[serde(default)]
    pub ethereum_usdc_contract: Option<String>,

    // ── BSC ──────────────────────────────────────────────────────────────────
    #[serde(default)]
    pub bsc_rpc_url: Option<String>,
    /// BEP-20 token contract to monitor on BSC (e.g. USDT)
    #[serde(default)]
    pub bsc_token_contract: Option<String>,
    /// Asset symbol for the BSC token contract (default: "USDT")
    #[serde(default)]
    pub bsc_token_symbol: Option<String>,

    // ── Tron ─────────────────────────────────────────────────────────────────
    /// TronGrid base URL (e.g. https://api.trongrid.io)
    #[serde(default)]
    pub tron_rpc_url: Option<String>,
    /// TRC-20 contract address to monitor (e.g. USDT: TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t)
    #[serde(default)]
    pub tron_token_contract: Option<String>,
    /// Asset symbol for the Tron token contract (default: "USDT")
    #[serde(default)]
    pub tron_token_symbol: Option<String>,
}

fn default_shutdown_timeout() -> u64 {
    30
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();
        let cfg = config::Config::builder().add_source(config::Environment::default()).build()?;
        Ok(cfg.try_deserialize()?)
    }
}
