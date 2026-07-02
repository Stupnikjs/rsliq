// tests/test_anvil.rs

mod anvil; // Importe ton fichier anvil.rs
use std::env::var;

// Importe ce dont tu as besoin directement
use anvil::AnvilInstance; // <- IMPORTANT : on précise que ça vient du mod anvil
use alloy::providers::{Provider, ProviderBuilder};
use alloy::{
    hex,
    primitives::{address, Bytes},
    rpc::types::TransactionRequest,
};



fn get_rpc_url() -> String {
    dotenvy::dotenv().ok();
    String::from(var("ALCHEMY_BASE_HTTP").expect("ALCHEMY_BASE_HTTP not set"))
}

#[tokio::test]
async fn test_anvil_basic() {
    let anvil = AnvilInstance::fork(&get_rpc_url(), None).unwrap();

    println!("{}", anvil.endpoint()); 
    let provider = ProviderBuilder::new()
        .connect_http(anvil.endpoint().parse().unwrap());

    let block = provider.get_block_number().await.unwrap();
    println!("✓ Block number: {}", block);

    let chain_id = provider.get_chain_id().await.unwrap();
    println!("✓ Chain ID: {}", chain_id);
    assert_eq!(chain_id, 8453);

}

#[tokio::test]
async fn test_anvil_fork_at_block() {
    let target_block = 19_500_000u64;
    let anvil = AnvilInstance::fork(&get_rpc_url(), Some(target_block)).unwrap();

    let provider = ProviderBuilder::new()
        .connect_http(anvil.endpoint().parse().expect("invalid endpoint"));

    let block = provider.get_block_number().await.unwrap();
    println!("Forked at block: {}", block);
    assert_eq!(block, target_block);
}

#[tokio::test]
async fn test_anvil_endpoints() {
    let anvil = AnvilInstance::fork(&get_rpc_url(), None).unwrap();

    println!("HTTP: {}", anvil.endpoint());
    println!("WS:   {}", anvil.ws_endpoint());

    assert!(anvil.endpoint().starts_with("http://127.0.0.1:"));
    assert!(anvil.ws_endpoint().starts_with("ws://127.0.0.1:"));
}






#[tokio::test]
async fn test_usdc_balance_raw() {
    let anvil = AnvilInstance::fork(&get_rpc_url(), None).unwrap();

    let provider = ProviderBuilder::new()
        .connect_http(anvil.endpoint().parse().unwrap());

    let usdc = address!("833589fCD6eDb6E08f4c7C32D4f71b54bdA02913");

    let owner = address!("4200000000000000000000000000000000000006");

    // selector balanceOf(address)
    let mut calldata = Vec::with_capacity(4 + 32);

    calldata.extend_from_slice(&hex!("70a08231"));

    // ABI padding
    calldata.extend_from_slice(&[0u8; 12]);
    calldata.extend_from_slice(owner.as_slice());

    let tx = TransactionRequest::default().to(usdc).input(calldata.into());
    let result = provider.call(tx).await.unwrap(); 
    println!("0x{}", hex::encode(&result));

    // balance est un uint256 big-endian
    let balance = alloy::primitives::U256::from_be_slice(&result);

    println!("Balance = {}", balance);
    println!("USDC = {}", balance / alloy::primitives::U256::from(1_000_000u64));
}