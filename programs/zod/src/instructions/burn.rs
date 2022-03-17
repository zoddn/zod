use anchor_lang::prelude::*;
use anchor_lang::solana_program::log::sol_log_compute_units;
use anchor_spl::token::{self, *};
use common::SafeOp;
use common::{system_program_utils, time};
use fixed::types::I80F48;
use zo::{self, cpi::accounts::*, program::ZoAbi as Zo, *};

use crate::state::*;
use crate::zodTypes::WrappedI80F48;
use zo::errors::ErrorCode;

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct ZodBurn<'info> {
  #[account(
    mut,
    seeds = [b"zodv12".as_ref()],
    bump = zod_state.load()?.zod_state_nonce,
  )]
  pub zod_state: AccountLoader<'info, ZodState>,
  pub authority: Signer<'info>,
  #[account(
    mut,
    seeds = [authority.key.as_ref(), zod_state.key().as_ref(), b"zodmarginv2".as_ref()],
    bump = zod_margin.load()?.nonce
  )]
  pub zod_margin: AccountLoader<'info, ZodMargin>,
  #[account(
    mut,
    constraint = {token_account.owner == *authority.key},
  )]
  pub token_account: Account<'info, TokenAccount>,
  pub token_program: Program<'info, Token>,
  #[account(
    mut,
    constraint = {mint.key() == zod_state.load()?.zod_token_info.mint},
  )]
  pub mint: Account<'info, Mint>,
}

pub fn process(cx: Context<ZodBurn>, amount: u64) -> ProgramResult {
  msg!("Instruction ZodBurn");

  let zod_state = &cx.accounts.zod_state;
  let mut zod_margin = cx.accounts.zod_margin.load_mut()?;
  let zod_balance: I80F48 = zod_margin.get_actual_zod_balance(zod_state.load()?.soc_loss_multiplier.into())?;
  assert!(zod_balance > amount);
  let amount_i80: I80F48 = I80F48::from_num(amount);

  zod_margin.zod_mutate(-amount_i80, zod_state.load()?.soc_loss_multiplier.into())?;
  zod_state.load_mut()?.mutate_zod_borrowed(-amount_i80)?;

  let burn_cpi_program = cx.accounts.token_program.to_account_info();
  let burn_cpi_accounts = Burn {
    mint: cx.accounts.mint.to_account_info(),
    to: cx.accounts.token_account.to_account_info(),
    authority: cx.accounts.authority.to_account_info(),
  };

  let burn_cpi_ctx = CpiContext::new(burn_cpi_program, burn_cpi_accounts);

  token::burn(burn_cpi_ctx, amount)?;

  Ok(())
}
