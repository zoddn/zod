use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};
use std::convert::TryInto;

use crate::state::*;
use zo::{self, program::ZoAbi as Zo, cpi::accounts::*, *};

#[derive(Accounts)]
pub struct AddVaults<'info> {
  pub admin: Signer<'info>,
  pub zo_state: AccountLoader<'info, State>,
  #[account(mut)]
  pub zod_state: AccountLoader<'info, ZodState>,
  #[account(
        mut,
        constraint = vault.owner == *zod_state.to_account_info().key,
    )]
  pub vault: Box<Account<'info, TokenAccount>>,
  #[account(address = vault.mint)]
  pub mint: Account<'info, Mint>,
}

pub fn process(cx: Context<AddVaults>) -> ProgramResult {
  msg!("instruction AddVaults");

  let zod_state = &mut cx.accounts.zod_state.load_mut()?;
  let zo_state = &mut cx.accounts.zo_state.load_mut()?;
  let mint = &cx.accounts.mint;

  assert!(!zo_state.get_collateral_index(&mint.key()).is_none());
  assert!(
    zod_state.vaults[zo_state.get_collateral_index(&mint.key()).unwrap()] == Pubkey::default()
  );

  let index = zo_state
    .collaterals
    .iter()
    .position(|col_info| *&col_info.is_empty())
    .unwrap();

  zod_state.vaults[index] = *cx.accounts.vault.to_account_info().key;

  Ok(())
}
