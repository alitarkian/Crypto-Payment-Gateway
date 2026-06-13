use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub app: AppSettings,
    pub database: DatabaseSettings,
    pub redis: RedisSettings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppSettings {
    pub env: String,
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseSettings {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedisSettings {
    pub url: String,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let cfg = config::Config::builder()
            .add_source(config::Environment::default().separator("_"))
            .build()?;

        Ok(cfg.try_deserialize()?)
    }
}