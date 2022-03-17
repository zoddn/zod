use anchor_lang::prelude::*;
use anchor_lang::solana_program::log::sol_log_compute_units;
use anchor_spl::token::{Token, TokenAccount};
use common::{system_program_utils, time};
use fixed::types::I80F48;

use crate::state::*;
use crate::zodTypes::WrappedI80F48;
use zo::errors::ErrorCode;

use ::zo::cpi::accounts::Deposit;
use zo::{self, cpi::accounts::*, program::ZoAbi as Zo, *};

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct ZodDeposit<'info> {
  #[account(mut)]
  pub zod_state: AccountLoader<'info, ZodState>,
  #[account(mut)]
  pub zo_program_margin: UncheckedAccount<'info>,
  pub zo_program: Program<'info, Zo>,
  #[account(address = zod_state.load()?.zo_program_state)]
  pub zo_program_state: AccountLoader<'info, State>,
  pub state_signer: UncheckedAccount<'info>,
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
      constraint = token_account.amount >= amount,
  )]
  pub token_account: Account<'info, TokenAccount>,
  #[account(
    mut,
    constraint = {zo_vault.owner == *state_signer.to_account_info().key},
    constraint = zo_vault.mint == token_account.mint
  )]
  pub zo_vault: Account<'info, TokenAccount>,
  #[account(
    mut,
    constraint = {zod_vault.owner == *zod_state.to_account_info().key},
    constraint = zod_vault.mint == token_account.mint
  )]
  pub zod_vault: Account<'info, TokenAccount>,
  pub token_program: Program<'info, Token>,
}

pub fn process(cx: Context<ZodDeposit>, amount: u64) -> ProgramResult {
  msg!("Instruction: ZodDeposit");
  let zod_state = &cx.accounts.zod_state;
  let zo_program_state = &cx.accounts.zo_program_state;
  let token_acc = &cx.accounts.token_account;
  let zo_vault = &cx.accounts.zo_vault;
  let zod_vault = &cx.accounts.zod_vault;

  let col_index = zo_program_state
    .load()?
    .get_collateral_index(&token_acc.mint)
    .ok_or(ErrorCode::CollateralDoesNotExist)?;
  assert!(zo_program_state.load()?.vaults[col_index] == zo_vault.key());

  let amount_i80: I80F48 = I80F48::from_num(amount);
  {
    let zod_margin = &mut cx.accounts.zod_margin.load_mut()?;
    let cache = &mut cx.accounts.cache.load_mut()?;
    zod_margin.mutate(
      col_index,
      amount_i80,
      cache.borrow_cache[col_index].supply_multiplier.into(),
      cache.borrow_cache[col_index].borrow_multiplier.into(),
    )?;
  }
  {
    let amount_to_dep = amount_i80.ceil().to_num();
    system_program_utils::token_transfer(
      cx.accounts.token_program.to_account_info(),
      token_acc.to_account_info(),
      zod_vault.to_account_info(),
      cx.accounts.authority.to_account_info(),
      amount_to_dep,
      None,
    )?;
  }

  let cpi_program = cx.accounts.zo_program.to_account_info();
  let cpi_accounts = Deposit {
    state: cx.accounts.zo_program_state.to_account_info(),
    state_signer: cx.accounts.state_signer.to_account_info(),
    cache: cx.accounts.cache.to_account_info(),
    authority: cx.accounts.zod_state.to_account_info(),
    margin: cx.accounts.zo_program_margin.to_account_info(),
    token_account: cx.accounts.zod_vault.to_account_info(),
    vault: cx.accounts.zo_vault.to_account_info(),
    token_program: cx.accounts.token_program.to_account_info(),
  };

  let zod_state_seeds = &[b"zodv12".as_ref(), &[zod_state.load()?.zod_state_nonce]];

  let signers = &[&zod_state_seeds[..]];

  let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signers);

  zo::cpi::deposit(cpi_ctx, false, amount)?;

  Ok(())
}
