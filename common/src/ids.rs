use std::str::FromStr;

use anchor_lang::prelude::*;

pub fn memo_program_id() -> Pubkey {
    Pubkey::from_str("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr").unwrap()
}
