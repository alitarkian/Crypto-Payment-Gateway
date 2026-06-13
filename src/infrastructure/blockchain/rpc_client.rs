use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug, Clone)]
pub struct SolanaRpcClient {
    client: Client,
    url: String,
}

#[derive(Debug, Deserialize)]
pub struct RpcResponse<T> {
    pub result: T,
}

#[derive(Debug, Deserialize)]
pub struct SignaturesResult {
    pub signature: String,
    pub slot: u64,
    pub err: Option<serde_json::Value>,
    #[serde(rename = "blockTime")]
    pub block_time: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct TransactionResult {
    pub transaction: TransactionData,
    pub meta: Option<TransactionMeta>,
}

#[derive(Debug, Deserialize)]
pub struct TransactionData {
    pub message: TransactionMessage,
}

#[derive(Debug, Deserialize)]
pub struct TransactionMessage {
    #[serde(rename = "accountKeys")]
    pub account_keys: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct TransactionMeta {
    pub err: Option<serde_json::Value>,
    #[serde(rename = "preTokenBalances")]
    pub pre_token_balances: Option<Vec<TokenBalance>>,
    #[serde(rename = "postTokenBalances")]
    pub post_token_balances: Option<Vec<TokenBalance>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TokenBalance {
    #[serde(rename = "accountIndex")]
    pub account_index: u64,
    pub mint: String,
    pub owner: Option<String>,
    #[serde(rename = "uiTokenAmount")]
    pub ui_token_amount: UiTokenAmount,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UiTokenAmount {
    #[serde(rename = "uiAmountString")]
    pub ui_amount_string: String,
}

#[derive(Debug, Serialize)]
struct RpcRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: serde_json::Value,
}

impl SolanaRpcClient {
    pub fn new(url: String) -> Self {
        Self {
            client: Client::new(),
            url,
        }
    }

    pub async fn get_signatures_for_address(
        &self,
        address: &str,
        limit: u8,
    ) -> anyhow::Result<Vec<SignaturesResult>> {
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "getSignaturesForAddress".to_string(),
            params: serde_json::json!([
                address,
                { "limit": limit, "commitment": "confirmed" }
            ]),
        };

        debug!(address = %address, "Fetching signatures");

        let response: RpcResponse<Vec<SignaturesResult>> = self
            .client
            .post(&self.url)
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        Ok(response.result)
    }

    pub async fn get_transaction(
        &self,
        signature: &str,
    ) -> anyhow::Result<Option<TransactionResult>> {
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "getTransaction".to_string(),
            params: serde_json::json!([
                signature,
                { "encoding": "jsonParsed", "commitment": "confirmed", "maxSupportedTransactionVersion": 0 }
            ]),
        };

        debug!(signature = %signature, "Fetching transaction");

        let response: RpcResponse<Option<TransactionResult>> = self
            .client
            .post(&self.url)
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        Ok(response.result)
    }
}