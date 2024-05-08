predicate;

// 🔴 SELL PREDICATE

use std::u128::U128;
use std::outputs::{Output, output_asset_id, output_amount, output_count, output_type, output_asset_to};
use std::inputs::{input_amount, input_asset_id, input_count, input_coin_owner};
use std::constants::{ZERO_B256};

// Maker(Alice) wants to exchange BTC for USDC
// Taker(Bob) wants to exchange USDC for BTC 

configurable {
    BASE_ASSET: b256 = ZERO_B256, // Asset that provides maker(Alice)
    QUOTE_ASSET: b256 = ZERO_B256, // Asset that provides taker(Bob)
    MAKER: Address = Address::from(ZERO_B256), // Order owner
    PRICE: u64 = 0, 
    BASE_DECIMALS: u32 = 9,
    QUOTE_DECIMALS: u32 = 9,
    PRICE_DECIMALS: u32 = 9, // optional
    MIN_FULFILL_BASE_AMOUNT: u64 = 1, // optional
}

impl u64 {
    pub fn mul_div(self, mul_to: u64, div_to: u64) -> u64 {
        let mul_result = U128::from((0, self)) * U128::from((0, mul_to));
        let div_result = mul_result / U128::from((0, div_to));
        div_result.as_u64().unwrap()
    }
}

pub fn calc_price(base_amount: u64, quote_amount: u64) -> u64 {
    let exp = PRICE_DECIMALS + BASE_DECIMALS - QUOTE_DECIMALS;
    quote_amount.mul_div(10.pow(exp), base_amount)
}


fn main() -> bool {
    assert(PRICE > 0 && MAKER.into() != ZERO_B256);

    let mut i = 0u8;
    let inputs: u8 = input_count();
    while i < inputs  {
        if input_coin_owner(i.as_u64()).unwrap() == MAKER {
            return true;
        }
        i += 1u8;
    }
    
    assert(input_asset_id(0).unwrap().into() == BASE_ASSET);
    assert(output_asset_id(2).unwrap().into() == BASE_ASSET);
    assert(output_asset_id(0).unwrap().into() == QUOTE_ASSET);
    
    let base_output_amount = output_amount(2);

    let quote_output_amount = output_amount(0);
    let quote_output_to = output_asset_to(0).unwrap();
    
    let base_input_amount = input_amount(0).unwrap();
    
    assert(calc_price(base_output_amount, quote_output_amount) == PRICE);

    let limit = if base_input_amount >= MIN_FULFILL_BASE_AMOUNT {
        MIN_FULFILL_BASE_AMOUNT
    } else {
        base_input_amount
    };
    assert(base_output_amount >= limit);

    match output_type(0) {
        Output::Coin => (),
        _ => revert(0),
    };
    assert(quote_output_to == MAKER.into());
    true
}