#![allow(dead_code, unused_variables, unused_imports)]

use std::result;


use crate::morpho::types::MarketParam;
use crate::cache::{init_cache}; 
use alloy::providers::{ProviderBuilder, Provider};
use alloy::network::AnyNetwork;


mod morpho;
mod api;
mod cache; 
mod connector;


#[tokio::main]
async fn main() {
    let primary = "https://eth.drpc.org";
    let secondary = "https://go.getblock.us/27eb23f40b964c9bb71b62f721e594e7";
}

