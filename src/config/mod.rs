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
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();
        let cfg = config::Config::builder().add_source(config::Environment::default()).build()?;
        Ok(cfg.try_deserialize()?)
    }
}
