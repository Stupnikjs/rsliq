

use alloy::primitives::{keccak256, Address, Bytes, B256, U256};
use alloy::sol_types::{SolValue, Token};

/// Compute 4-byte function selector
pub fn selector(signature: &str) -> [u8; 4] {
    let hash = keccak256(signature.as_bytes());
    let mut sel = [0u8; 4];
    sel.copy_from_slice(&hash[..4]);
    sel
}

/// Encode calldata: selector + abi-encoded args
pub fn encode_calldata(selector: [u8; 4], args: &[Token]) -> Bytes {
    let mut data = Vec::with_capacity(4 + args.abi_encode().len());
    data.extend_from_slice(&selector);
    data.extend(args.abi_encode());
    data.into()
}

/// Decode ABI-encoded return value
pub fn decode_single_uint256(data: &Bytes) -> Result<U256, Box<dyn std::error::Error>> {
    let tokens = Token::decode_sequence(data, false)?;
    if let Token::Uint(val) = &tokens[0] {
        Ok(*val)
    } else {
        Err("expected uint256".into())
    }
}

pub fn decode_single_string(data: &Bytes) -> Result<String, Box<dyn std::error::Error>> {
    let tokens = Token::decode_sequence(data, false)?;
    if let Token::String(s) = &tokens[0] {
        Ok(s.clone())
    } else {
        Err("expected string".into())
    }
}

pub fn decode_single_address(data: &Bytes) -> Result<Address, Box<dyn std::error::Error>> {
    let tokens = Token::decode_sequence(data, false)?;
    if let Token::Address(addr) = &tokens[0] {
        Ok(*addr)
    } else {
        Err("expected address".into())
    }
}