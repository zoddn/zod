use anchor_lang::prelude::*;
use anchor_lang::solana_program::log::sol_log_compute_units;
use anchor_spl::token::{self, *};
use common::SafeOp;
use common::{system_program_utils, time};
use fixed::types::I80F48;
use zo::config::SPOT_INITIAL_MARGIN_REQ;

use crate::state::*;
use crate::zodTypes::WrappedI80F48;
use zo::{self, config::DEBUG_LOG, cpi::accounts::*, program::ZoAbi as Zo, *};

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct ZodMint<'info> {
  #[account(
    mut,
    seeds = [b"zodv12".as_ref()],
    bump = zod_state.load()?.zod_state_nonce,
  )]
  pub zod_state: AccountLoader<'info, ZodState>,
  #[account(
    mut,
    constraint = {zo_program_state.key() == zod_state.load()?.zo_program_state},
  )]
  pub zo_program_state: AccountLoader<'info, State>,
  #[account(mut, address = zo_program_state.load()?.cache)]
  pub cache: AccountLoader<'info, Cache>,
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

pub fn process(cx: Context<ZodMint>, amount: u64) -> ProgramResult {
  msg!("Instruction ZodMint");

  let mut zod_margin = cx.accounts.zod_margin.load_mut()?;
  let zod_state = &cx.accounts.zod_state;
  let zo_program_state = &cx.accounts.zo_program_state;
  let cache = &cx.accounts.cache;
  let current_time = time::get_current_time()?;
  let amount_i80: I80F48 = I80F48::from_num(amount);

  zod_margin.zod_mutate(amount_i80, zod_state.load()?.soc_loss_multiplier.into())?;
  zod_state.load_mut()?.mutate_zod_borrowed(amount_i80)?;

  let zod_balance: I80F48 =
    zod_margin.get_actual_zod_balance(zod_state.load()?.soc_loss_multiplier.into())?;

  let omf = zod_margin.get_omf(
    &zo_program_state.load()?,
    &cache.load()?,
    &zod_state.load()?,
    true,
    current_time,
    None,
  )?;

  let imf = zod_margin.get_imf(&zod_state.load()?)?;

  assert!(omf > imf);

  msg!(
    "total_zod_borrowed after: {}",
    zod_state.load()?.get_actual_zod_borrowed()
  );

  let cpi_program = cx.accounts.token_program.to_account_info();
  let cpi_accounts = MintTo {
    mint: cx.accounts.mint.to_account_info(),
    to: cx.accounts.token_account.to_account_info(),
    authority: cx.accounts.zod_state.to_account_info(),
  };

  let zod_state_seeds = &[b"zodv12".as_ref(), &[zod_state.load()?.zod_state_nonce]];

  let signers = &[&zod_state_seeds[..]];

  let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signers);

  token::mint_to(cpi_ctx, amount)?;

  Ok(())
}
