use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

use crate::error::ErrorCode;
use crate::ids::memo_program_id;

pub fn attach_price_memo(memo_price: u64, memo_program: AccountInfo) -> Result<(), ErrorCode> {
    let memo_string = format!("{}", memo_price);
    let memo = memo_string.as_bytes();
    let memo_ix = build_memo_ix(memo, &[]);
    anchor_lang::solana_program::program::invoke(&memo_ix, &[memo_program])
        .map_err(|_| ErrorCode::MemoFailure)?;
    Ok(())
}

/// Build a memo instruction, possibly signed
///
/// Accounts expected by this instruction:
///
///   0. ..0+N. `[signer]` Expected signers; if zero provided, instruction will be processed as a
///     normal, unsigned spl-memo
///
fn build_memo_ix(memo: &[u8], signer_pubkeys: &[&Pubkey]) -> Instruction {
    let id = memo_program_id();
    Instruction {
        program_id: id,
        accounts: signer_pubkeys
            .iter()
            .map(|&pubkey| AccountMeta::new_readonly(*pubkey, true))
            .collect(),
        data: memo.to_vec(),
    }
}
