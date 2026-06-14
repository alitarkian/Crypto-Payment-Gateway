use reqwest::{Client, StatusCode};
use uuid::Uuid;

use crate::error::SdkError;
use crate::types::*;

#[derive(Clone)]
pub struct GatewayClient {
    client: Client,
    base_url: String,
    api_key: Option<String>,
}

impl GatewayClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into().trim_end_matches('/').to_string(),
            api_key: None,
        }
    }

    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    fn authenticated(&self) -> reqwest::RequestBuilder {
        let builder = self.client.get(""); // placeholder — overridden per call
        builder
    }

    async fn handle_response<T: serde::de::DeserializeOwned>(
        response: reqwest::Response,
    ) -> Result<T, SdkError> {
        let status = response.status();
        if status.is_success() {
            Ok(response.json::<T>().await?)
        } else {
            let message = response
                .json::<serde_json::Value>()
                .await
                .ok()
                .and_then(|v| v["error"].as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| status.to_string());
            Err(SdkError::Api {
                status: status.as_u16(),
                message,
            })
        }
    }

    // ─── Merchants ────────────────────────────────────────────────────────────

    pub async fn create_merchant(
        &self,
        req: CreateMerchantRequest,
    ) -> Result<Merchant, SdkError> {
        let response = self
            .client
            .post(self.url("/api/v1/merchants"))
            .json(&req)
            .send()
            .await?;
        Self::handle_response(response).await
    }

    pub async fn get_merchant(&self, id: Uuid) -> Result<Merchant, SdkError> {
        let response = self
            .client
            .get(self.url(&format!("/api/v1/merchants/{id}")))
            .send()
            .await?;
        Self::handle_response(response).await
    }

    // ─── Wallets ──────────────────────────────────────────────────────────────

    pub async fn create_wallet(&self, req: CreateWalletRequest) -> Result<Wallet, SdkError> {
        let mut builder = self
            .client
            .post(self.url("/api/v1/wallets"))
            .json(&req);
        if let Some(key) = &self.api_key {
            builder = builder.header("x-api-key", key);
        }
        let response = builder.send().await?;
        Self::handle_response(response).await
    }

    pub async fn get_wallet(&self, id: Uuid) -> Result<Wallet, SdkError> {
        let mut builder = self.client.get(self.url(&format!("/api/v1/wallets/{id}")));
        if let Some(key) = &self.api_key {
            builder = builder.header("x-api-key", key);
        }
        let response = builder.send().await?;
        Self::handle_response(response).await
    }

    // ─── Invoices ─────────────────────────────────────────────────────────────

    pub async fn create_invoice(&self, req: CreateInvoiceRequest) -> Result<Invoice, SdkError> {
        let mut builder = self
            .client
            .post(self.url("/api/v1/invoices"))
            .json(&req);
        if let Some(key) = &self.api_key {
            builder = builder.header("x-api-key", key);
        }
        let response = builder.send().await?;
        Self::handle_response(response).await
    }

    pub async fn get_invoice(&self, id: Uuid) -> Result<Invoice, SdkError> {
        let mut builder = self.client.get(self.url(&format!("/api/v1/invoices/{id}")));
        if let Some(key) = &self.api_key {
            builder = builder.header("x-api-key", key);
        }
        let response = builder.send().await?;
        Self::handle_response(response).await
    }

    // ─── Webhooks ─────────────────────────────────────────────────────────────

    pub async fn register_webhook(
        &self,
        req: RegisterWebhookRequest,
    ) -> Result<Webhook, SdkError> {
        let mut builder = self
            .client
            .post(self.url("/api/v1/webhooks"))
            .json(&req);
        if let Some(key) = &self.api_key {
            builder = builder.header("x-api-key", key);
        }
        let response = builder.send().await?;
        Self::handle_response(response).await
    }
}