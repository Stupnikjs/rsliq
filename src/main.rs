#![allow(dead_code, unused_variables, unused_imports)]





mod morpho;
mod api;
mod cache; 
pub mod connector;
mod onchain;
mod config;
mod runner; 
pub mod swap;


use std::sync::Arc;
use alloy::providers::Provider;
use tokio::time::{sleep, Duration};

use crate::runner::Runner;





#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conf; 
    let chain = std::env::args().nth(1).unwrap_or_else(|| {
    eprintln!("usage: rsliq <chain>");
    std::process::exit(1);
    });
    let chainint:u64 = chain.parse()?;
    let runner: Runner<impl Provider> = runner::Runner::new(8453)?; 
    runner.init()?; 
    runner.run();
    // garde le main en vie
    tokio::signal::ctrl_c().await?;
    Ok(())
}



