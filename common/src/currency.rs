//! Language convention when referring to a token/ currency
//!
//! SOL / <symbol of currency> / bigs - refer to the visual representation of the currency
//! lamports / satoshis / smols - refers to the lowest possible representation
//!
//! decimals - refers to the number of decimals in a currency's lowest representation
//!
//! Example for BTC:
//! 1 SOL / big = 1 BTC = 100_000_000 lamports
//! 1 lamport / smol = 0.000_000_01 BTC
//!
//! Examples for USDC:
//! 1 USDC / big = 1_000_000 lamports / smol
//! 1 lamport / smol of USDC = 0.000_001 USDC / big

use crate::math::{safe_div_i80f48, safe_mul_i80f48};
use fixed::types::I80F48;

/// Converts SOL -> lamports
pub fn big_to_smol(unit: I80F48, decimals: u32) -> I80F48 {
    safe_mul_i80f48(unit, I80F48::from_num(10u64.pow(decimals)))
}

/// Converts lamports -> SOL
pub fn smol_to_big(decimal: I80F48, decimals: u32) -> I80F48 {
    safe_div_i80f48(decimal, I80F48::from_num(10u64.pow(decimals)))
}

/// Converts price denominated in main big/ big -> main smol/ smol (USDC / SOL -> smol USDC / lamports)
pub fn big_price_to_smol_price(big: I80F48, main_decimal: u32, other_decimal: u32) -> I80F48 {
    let step_one = safe_mul_i80f48(big, I80F48::from_num(10u64.pow(main_decimal)));
    safe_div_i80f48(step_one, I80F48::from_num(10u64.pow(other_decimal)))
}

/// Converts price denominated in main smol/ smol -> main big/ big (smol USDC / lamports -> USDC / SOL)
pub fn smol_price_to_big_price(smol: I80F48, main_decimal: u32, other_decimal: u32) -> I80F48 {
    let step_one = safe_mul_i80f48(smol, I80F48::from_num(10u64.pow(other_decimal)));
    safe_div_i80f48(step_one, I80F48::from_num(10u64.pow(main_decimal)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_big_to_smol() {
        {
            let big = I80F48::from_num(1); // 1 SOL
            let decimals = 9;

            let smol = big_to_smol(big, decimals);

            assert_eq!(smol, 1_000_000_000);
        }
        {
            let big = I80F48::from_num(5); // 5 BTC
            let decimals = 8;

            let smol = big_to_smol(big, decimals);

            assert_eq!(smol, 5_0000_0000);
        }
        {
            let big = I80F48::from_num(72); // 72 USDC
            let decimals = 6;

            let smol = big_to_smol(big, decimals);

            assert_eq!(smol, 72 * 1_000_000);
        }
    }

    #[test]
    fn test_smol_to_big() {
        {
            let smol = I80F48::from_num(1_000_000_000); // 1_000_000_000 lamports
            let decimals = 9;

            let big = smol_to_big(smol, decimals);

            assert_eq!(big, 1);
        }
        {
            let smol = I80F48::from_num(5 * 1_0000_0000); // 5 * 1_0000_0000 satoshis
            let decimals = 8;

            let big = smol_to_big(smol, decimals);

            assert_eq!(big, 5);
        }
        {
            let smol = I80F48::from_num(72 * 1_000_000); // 72 * 1_000_000 smol usdc
            let decimals = 6;

            let big = smol_to_big(smol, decimals);

            assert_eq!(big, 72);
        }
    }

    #[test]
    fn test_big_price_to_smol_price() {
        {
            let big_price = I80F48::from_num(50_000.000); // 50_000 USD/BTC
            let main_decimal = 6;
            let other_decimal = 9;

            let smol_price = big_price_to_smol_price(big_price, main_decimal, other_decimal);

            assert_eq!(smol_price, 50);
        }
        {
            let big_price = I80F48::from_num(50_000.000); // 50_000 USD/ something
            let main_decimal = 6;
            let other_decimal = 6;

            let smol_price = big_price_to_smol_price(big_price, main_decimal, other_decimal);

            assert_eq!(smol_price, 50_000);
        }
    }

    #[test]
    fn test_smol_price_to_big_price() {
        {
            let smol_price = I80F48::from_num(50.000); // 50 smol USD/ satoshis
            let main_decimal = 6;
            let other_decimal = 9;

            let big_price = smol_price_to_big_price(smol_price, main_decimal, other_decimal);

            assert_eq!(big_price, 50_000);
        }
        {
            let smol_price = I80F48::from_num(50.000); // 50 smol USD/ something
            let main_decimal = 6;
            let other_decimal = 6;

            let big_price = smol_price_to_big_price(smol_price, main_decimal, other_decimal);

            assert_eq!(big_price, 50);
        }
    }
}
