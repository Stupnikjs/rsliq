use alloy::primitives::{Address, Bytes, U256};
use alloy::providers::{DynProvider, ProviderBuilder, Provider};
use alloy::rpc::types::TransactionRequest;
use alloy::sol_types::{SolCall, SolValue};  // ← LES DEUX
use alloy::sol;

sol! {
    function balanceOf(address owner) external view returns (uint256);
    function name() external view returns (string);
}

pub struct Rpc {
    pub provider: DynProvider,
}

impl Rpc {
    pub fn http(url: &str) -> Self {
        Self {
            provider: ProviderBuilder::new()
                .connect_http(url.parse().unwrap())
                .erased(),
        }
    }

    pub async fn balance_of(
        &self,
        token: Address,
        owner: Address,
    ) -> Result<U256, Box<dyn std::error::Error + Send + Sync>> {
        let call = balanceOfCall { owner };
        let calldata = call.abi_encode();  // SolCall

        let tx = TransactionRequest::default()
            .to(token)
            .input(calldata.into());

        let raw: Bytes = self.provider.call(tx).await?;
        
        let result = nameCall::abi_decode_returns(&raw)?;  // SolValue
        Ok(result)
    }

    pub async fn token_name(
        &self,
        token: Address,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let call = nameCall {};
        let calldata = call.abi_encode();  // SolCall

        let tx = TransactionRequest::default()
            .to(token)
            .input(calldata.into());

        let raw: Bytes = self.provider.call(tx).await?;
        
        let result = nameCall::abi_decode_returns(&raw)?;   // SolValue
        Ok(result)
    }
}