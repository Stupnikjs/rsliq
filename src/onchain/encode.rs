use alloy::primitives::{keccak256, Address, Bytes, U256};
use alloy::sol_types::{SolType, SolValue};
use alloy::sol_types::sol_data;

pub fn selector(signature: &str) -> [u8; 4] {
    let hash = keccak256(signature.as_bytes());
    let mut sel = [0u8; 4];
    sel.copy_from_slice(&hash[..4]);
    sel
}

pub fn encode_calldata(sel: [u8; 4], args: &impl SolValue) -> Bytes {
    let mut data = Vec::with_capacity(4 + args.abi_encoded_size());
    data.extend_from_slice(&sel);
    data.extend(args.abi_encode());
    data.into()
}

pub fn decode_uint(data: &[u8]) -> Result<U256, Box<dyn std::error::Error>> {
    Ok(sol_data::Uint::<256>::abi_decode(data)?)
}

pub fn decode_address(data: &[u8]) -> Result<Address, Box<dyn std::error::Error>> {
    Ok(sol_data::Address::abi_decode(data)?)
}

pub fn decode_string(data: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
    Ok(sol_data::String::abi_decode(data)?)
}