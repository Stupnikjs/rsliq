use reqwest::Client;
use serde_json::{json, Value};

#[derive(Debug)]
pub enum ProviderError {
    HttpError(String),
    RpcError(String),
}

pub struct Connector {
    client: Client,
    rpc_url: String,
}

impl Connector {
    pub fn new(rpc_url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            rpc_url: rpc_url.into(),
        }
    }

    async fn send(
        &self,
        method: &str,
        params: Value,
    ) -> Result<Value, ProviderError> {
        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        });

        let response = self
            .client
            .post(&self.rpc_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| ProviderError::HttpError(e.to_string()))?;

        let body: Value = response
            .json()
            .await
            .map_err(|e| ProviderError::HttpError(e.to_string()))?;

        if let Some(error) = body.get("error") {
            return Err(ProviderError::RpcError(error.to_string()));
        }

        body.get("result")
            .cloned()
            .ok_or_else(|| ProviderError::RpcError("missing result".into()))
    }

    pub async fn eth_call_raw(
        &self,
        to: &str,
        data: &str,
        block: &str,
    ) -> Result<Vec<u8>, ProviderError> {
        let result = self
            .send(
                "eth_call",
                json!([
                    {
                        "to": to,
                        "data": data
                    },
                    block
                ]),
            )
            .await?;

        let hex = result.as_str().unwrap_or("0x");

        hex::decode(hex.trim_start_matches("0x"))
            .map_err(|_| ProviderError::RpcError("invalid hex".into()))
    }

    pub async fn block_number(&self) -> Result<u64, ProviderError> {
        let result = self.send("eth_blockNumber", json!([])).await?;

        let hex = result
            .as_str()
            .ok_or_else(|| ProviderError::RpcError("invalid response".into()))?;

        u64::from_str_radix(hex.trim_start_matches("0x"), 16)
            .map_err(|e| ProviderError::RpcError(e.to_string()))
    }

    pub async fn get_balance(
        &self,
        address: &str,
        block: &str,
    ) -> Result<u128, ProviderError> {
        let result = self
            .send(
                "eth_getBalance",
                json!([address, block]),
            )
            .await?;

        let hex = result
            .as_str()
            .ok_or_else(|| ProviderError::RpcError("invalid response".into()))?;

        u128::from_str_radix(hex.trim_start_matches("0x"), 16)
            .map_err(|e| ProviderError::RpcError(e.to_string()))
    }
}