use reqwest::Client;
use rust_decimal::Decimal;
use serde::{ Deserialize, Serialize };
use tracing::debug;

/// ERC-20 Transfer event topic
const TRANSFER_TOPIC: &str = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

/// USDC has 6 decimals on Ethereum
const USDC_DECIMALS: u32 = 6;

#[derive(Clone)]
pub struct EthereumRpcClient {
    client: Client,
    url: String,
}

#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: &'static str,
    id: u64,
    method: &'static str,
    params: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponse<T> {
    result: Option<T>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EthLog {
    #[serde(rename = "transactionHash")]
    pub transaction_hash: String,
    #[allow(dead_code)]
    pub topics: Vec<String>,
    pub data: String,
    pub removed: Option<bool>,
}

impl EthereumRpcClient {
    pub fn new(url: String) -> Self {
        Self {
            client: Client::new(),
            url,
        }
    }

    async fn call<T: serde::de::DeserializeOwned>(
        &self,
        method: &'static str,
        params: serde_json::Value
    ) -> anyhow::Result<Option<T>> {
        let req = JsonRpcRequest {
            jsonrpc: "2.0",
            id: 1,
            method,
            params,
        };

        let resp: JsonRpcResponse<T> = self.client
            .post(&self.url)
            .json(&req)
            .send().await?
            .json().await?;

        Ok(resp.result)
    }

    /// ERC-20 Transfer logs به سمت یک آدرس خاص
    pub async fn get_usdc_transfers_to(
        &self,
        wallet_address: &str,
        usdc_contract: &str,
        from_block: &str
    ) -> anyhow::Result<Vec<EthLog>> {
        let address_topic = format!("0x000000000000000000000000{}", &wallet_address[2..]);

        debug!(
            wallet = %wallet_address,
            "Ethereum: fetching USDC transfers"
        );

        let logs = self
            .call::<Vec<EthLog>>(
                "eth_getLogs",
                serde_json::json!([{
                    "fromBlock": from_block,
                    "toBlock": "latest",
                    "address": usdc_contract,
                    "topics": [
                        TRANSFER_TOPIC,
                        null,
                        address_topic
                    ]
                }])
            ).await?
            .unwrap_or_default();

        Ok(logs)
    }

    pub async fn get_block_number(&self) -> anyhow::Result<u64> {
        let hex: Option<String> = self.call("eth_blockNumber", serde_json::json!([])).await?;
        let hex = hex.unwrap_or_else(|| "0x0".to_string());
        Ok(u64::from_str_radix(hex.trim_start_matches("0x"), 16)?)
    }

    /// hex data از ERC-20 Transfer رو به Decimal تبدیل می‌کنه
    pub fn decode_transfer_amount(data: &str) -> Option<Decimal> {
        let hex = data.trim_start_matches("0x");
        if hex.len() < 64 {
            return None;
        }
        let amount_hex = &hex[..64];
        let amount_u128 = u128::from_str_radix(amount_hex, 16).ok()?;
        let divisor = Decimal::from((10u64).pow(USDC_DECIMALS));
        Some(Decimal::from(amount_u128) / divisor)
    }
}
