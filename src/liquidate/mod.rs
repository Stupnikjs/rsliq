mod encode;
mod build; 
use crate::{cache::positions::BorrowPosition, morpho::types::MarketParam, swap::routes::SwapRoute};
use crate::connector::Connector;

pub async fn liquidate(conn: &Connector, pos: BorrowPosition, route: SwapRoute, mparam: MarketParam) { 
   // let calldata = encode()
   // simulate 
   // send 
}