use alloy_primitives::{Address, FixedBytes, U256, address};

// Constante vérifiée à la compilation
pub const IRM: Address = address!("0x46415998764C29aB2a25CbeA6254146D50D22687");

#[derive(Debug, Clone, Default)]
pub struct MarketParams {
    pub id: FixedBytes<32>,
    pub loan_token: Address,
    pub collateral_token: Address,
    pub oracle: Address,
    pub irm: Address,
    pub lltv: U256, // Remplacé par U256
    pub loan_token_str: String,
    pub collateral_token_str: String,
    pub chain_id: u32,
    pub loan_token_decimals: u16,
    pub collateral_token_decimals: u16,
}

#[derive(Debug, Clone, Default)]
pub struct MarketContractParams {
    pub loan_token: Address,
    pub collateral_token: Address,
    pub oracle: Address,
    pub irm: Address,
    pub lltv: U256, // Remplacé par U256
}

impl MarketParams {
    pub fn to_market_contract_params(&self) -> MarketContractParams {
        MarketContractParams {
            loan_token: self.loan_token,
            collateral_token: self.collateral_token,
            oracle: self.oracle,
            irm: self.irm,
            lltv: self.lltv, // Copie directe (U256 implémente Copy/Clone)
        }
    }

    pub fn is_eth_correlated(&self) -> bool {
        self.collateral_token_str.contains("ETH") && self.loan_token_str.contains("ETH")
    }

    pub fn is_btc_correlated(&self) -> bool {
        self.collateral_token_str.contains("BTC") && self.loan_token_str.contains("BTC")
    }

    pub fn get_pair(&self) -> String {
        format!("{}/{}", self.collateral_token_str, self.loan_token_str)
    }

    pub fn get_collateral_token(&self) -> Address {
        self.collateral_token
    }

    pub fn get_loan_token(&self) -> Address {
        self.loan_token
    }

    pub fn get_lltv(&self) -> U256 {
        self.lltv
    }

    pub fn max_slippage(&self) -> f64 {
        // Le LLTV de Morpho possède 18 décimales (ex: 90% = 0.9 * 1e18 = 900000000000000000)
        // On peut le convertir en f64 de manière sécurisée en le divisant par 1e14 pour obtenir un pourcentage direct
        let lltv_factor: u64 = 100_000_000_000_000; // 1e14
        
        // On divise le U256 puis on extrait le résultat en u64 avant de passer en f64
        let lltv_pct_scaled = (self.lltv / U256::from(lltv_factor)).to::<u64>();
        
        // On divise par 10000.0 pour retrouver le vrai pourcentage (ex: 900000 / 10000.0 = 90.0%)
        let lltv_pct = lltv_pct_scaled as f64 / 10000.0;
        
        let bonus = 100.0 - lltv_pct;
        const GAS_CUSHION: f64 = 0.1;
        
        bonus - GAS_CUSHION
    }
}