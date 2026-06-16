use anyhow::Context;
use reqwest::Client;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::str::FromStr;

/// TronGrid REST API client for TRC-20 token transfer detection.
pub struct TronClient {
    client: Client,
    base_url: String,
}

impl TronClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("Failed to build TronClient HTTP client"),
            base_url,
        }
    }

    /// Fetch TRC-20 token transfers TO a given wallet address.
    /// Uses TronGrid: GET /v1/accounts/{address}/transactions/trc20
    pub async fn get_trc20_transfers_to(
        &self,
        wallet_address: &str,
        contract_address: &str,
        min_timestamp_ms: u64,
    ) -> anyhow::Result<Vec<Trc20Transfer>> {
        let url = format!(
            "{}/v1/accounts/{}/transactions/trc20?contract_address={}&only_to=true&min_timestamp={}&limit=50",
            self.base_url.trim_end_matches('/'),
            wallet_address,
            contract_address,
            min_timestamp_ms,
        );

        let response: Trc20Response = self
            .client
            .get(&url)
            .send()
            .await
            .context("TronGrid HTTP request failed")?
            .json()
            .await
            .context("Failed to parse TronGrid response")?;

        Ok(response.data)
    }
}

#[derive(Debug, Deserialize)]
pub struct Trc20Response {
    pub data: Vec<Trc20Transfer>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Trc20Transfer {
    /// Transaction hash
    pub transaction_id: String,
    /// Token contract address
    pub token_info: Trc20TokenInfo,
    /// Transfer amount as string (in smallest unit, e.g. sun for TRX)
    pub value: String,
    /// Recipient address
    pub to: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Trc20TokenInfo {
    pub address: String,
    pub decimals: u8,
    pub symbol: String,
}

impl Trc20Transfer {
    /// Convert raw value string to human-readable Decimal using token decimals.
    pub fn amount_decimal(&self) -> Option<Decimal> {
        let raw = Decimal::from_str(&self.value).ok()?;
        let divisor = Decimal::from(10u64.pow(self.token_info.decimals as u32));
        Some(raw / divisor)
    }
}
