// src/swap/uniswap.rs
use std::time::{Duration, Instant};
use alloy::primitives::{Address, U256, Bytes};
use crate::swap::PoolEdge;
use crate::connector::Connector;
use crate::onchain::encode::{encode_address, encode_uint256,selector}; 

const UNI_FEES: [u32; 4] = [100, 500, 3000, 10000];

pub struct UniswapV3 {
    pub quoter: Address,
    pub router: Address,
    pub rate_limit: Duration,
    pub name: String,
}

impl UniswapV3 {
    pub fn new(quoter: Address, router: Address, rate_limit: Duration, name: String) -> Self {
        Self { quoter, router, rate_limit, name }
    }

    // binary search sur amountIn comme le Go
    pub async fn best_amount_in(
        &self,
        connector: &Connector,
        token_in: Address,
        token_out: Address,
        amount_in: U256,
        oracle_price: U256,
        max_slippage: f64,
    ) -> Option<PoolEdge> {
        let mut lo = U256::from(1u64);
        let mut hi = amount_in;
        let mut best: Option<PoolEdge> = None;

        for _ in 0..12 {
            if lo > hi { break; }
            let mid = (lo + hi) >> 1;

            match self.uni_quote(connector, token_in, token_out, mid, oracle_price, max_slippage).await {
                Some(edge) => {
                    best = Some(edge);
                    lo = mid + U256::from(1u64);
                }
                None => {
                    if mid == U256::ZERO { break; }
                    hi = mid - U256::from(1u64);
                }
            }
        }

        best
    }

    // essaie tous les fee tiers, retourne le meilleur amountOut
    async fn uni_quote(
        &self,
        connector: &Connector,
        token_in: Address,
        token_out: Address,
        amount_in: U256,
        oracle_price: U256,
        max_slippage: f64,
    ) -> Option<PoolEdge> {
        let mut best: Option<PoolEdge> = None;

        for &fee in &UNI_FEES {
            tokio::time::sleep(self.rate_limit).await;

            let edge = self.quote_call(
                connector, token_in, token_out, amount_in, oracle_price, fee,
            ).await?;

            if edge.wc_slippage > max_slippage {
                continue;
            }

            match &best {
                None => best = Some(edge),
                Some(b) if edge.wc_amount_out > b.wc_amount_out => best = Some(edge),
                _ => {}
            }
        }

        best
    }

    // appel RPC — à implémenter avec ABI manuel
    async fn quote_call(
        &self,
        connector: &Connector,
        token_in: Address,
        token_out: Address,
        amount_in: U256,
        oracle_price: U256,
        fee: u32,
    ) -> Option<PoolEdge> {
        // TODO: encoder quoteExactInputSingle + eth_call + décoder amountOut
        let amount_out = self.eth_call_quote(connector, token_in, token_out, amount_in, fee).await?;

        Some(PoolEdge {
            token_in,
            token_out,
            quoter: self.quoter,
            router: self.router,
            fee,
            wc_slippage: compute_slippage(amount_in, amount_out, oracle_price),
            wc_amount_in: amount_in,
            wc_amount_out: amount_out,
            calibrated_at: Instant::now(),
            dex_name: self.name.clone(),
            amount_in_offset: 164, // uniswap offset
            price_at_quote: oracle_price,
        })
    }

    async fn eth_call_quote(
        &self,
        connector: &Connector,
        token_in: Address,
        token_out: Address,
        amount_in: U256,
        fee: u32,
    ) -> Option<U256> {
        // TODO ABI manuel
        todo!()
    }
}

pub fn compute_slippage(amount_in: U256, amount_out: U256, oracle_price: U256) -> f64 {
    if oracle_price.is_zero() { return 0.0; }

    // expected_out = amount_in * oracle_price / 1e36
    let scale = U256::from(10u64).pow(U256::from(36u64));
    let expected_out = amount_in * oracle_price / scale;

    if expected_out.is_zero() { return 0.0; }

    if amount_out >= expected_out { return 0.0; }

    let diff = expected_out - amount_out;

    // conversion en f64
    let diff_f: f64 = diff.to_string().parse().unwrap_or(0.0);
    let exp_f: f64 = expected_out.to_string().parse().unwrap_or(1.0);

    (diff_f / exp_f) * 100.0
}





// ── encoders DEX ─────────────────────────────────────────────────

// exactInputSingle(address,address,uint24,address,uint256,uint256,uint160)
// amountIn placeholder = 0, patché on-chain via amountInOffset
pub fn encode_exact_input_single_uni(
    token_in: Address,
    token_out: Address,
    fee: u32,
    recipient: Address,
) -> Bytes {
    let sel = selector("exactInputSingle((address,address,uint24,address,uint256,uint256,uint160))");
    let mut args = Vec::with_capacity(7 * 32);
    args.extend_from_slice(&encode_address(token_in));
    args.extend_from_slice(&encode_address(token_out));
    args.extend_from_slice(&encode_uint256(U256::from(fee)));
    args.extend_from_slice(&encode_address(recipient));
    args.extend_from_slice(&encode_uint256(U256::ZERO)); // amountIn placeholder
    args.extend_from_slice(&encode_uint256(U256::ZERO)); // amountOutMinimum
    args.extend_from_slice(&encode_uint256(U256::ZERO)); // sqrtPriceLimitX96

    let mut out = Vec::with_capacity(4 + args.len());
    out.extend_from_slice(&sel);
    out.extend_from_slice(&args);
    out.into()
}

// exactInputSingle(address,address,uint24,address,uint256,uint256,uint256,uint160) — avec deadline
pub fn encode_exact_input_single_pancake(
    token_in: Address,
    token_out: Address,
    fee: u32,
    recipient: Address,
) -> Bytes {
    let sel = selector("exactInputSingle((address,address,uint24,address,uint256,uint256,uint256,uint160))");
    let mut args = Vec::with_capacity(8 * 32);
    args.extend_from_slice(&encode_address(token_in));
    args.extend_from_slice(&encode_address(token_out));
    args.extend_from_slice(&encode_uint256(U256::from(fee)));
    args.extend_from_slice(&encode_address(recipient));
    args.extend_from_slice(&encode_uint256(U256::ZERO)); // deadline placeholder
    args.extend_from_slice(&encode_uint256(U256::ZERO)); // amountIn placeholder
    args.extend_from_slice(&encode_uint256(U256::ZERO)); // amountOutMinimum
    args.extend_from_slice(&encode_uint256(U256::ZERO)); // sqrtPriceLimitX96

    let mut out = Vec::with_capacity(4 + args.len());
    out.extend_from_slice(&sel);
    out.extend_from_slice(&args);
    out.into()
}