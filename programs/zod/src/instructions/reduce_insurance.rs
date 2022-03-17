use anchor_lang::prelude::*;
use anchor_lang::solana_program::log::sol_log_compute_units;
use anchor_spl::token::{Token, TokenAccount, *};
use common::SafeOp;
use common::{system_program_utils, time};
use fixed::types::I80F48;
use zo::config::SPOT_INITIAL_MARGIN_REQ;

use crate::state::*;
use crate::zodTypes::WrappedI80F48;

use ::zo::cpi::accounts::Withdraw;
use zo::{self, config::DEBUG_LOG, cpi::accounts::*, program::ZoAbi as Zo, *};
use zo::errors::ErrorCode;

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct ZodReduceInsurance<'info> {
  #[account(mut)]
  pub zo_program_margin: AccountLoader<'info, Margin>,
  pub zo_program: Program<'info, Zo>,
  #[account(mut)]
  pub zod_state: AccountLoader<'info, ZodState>,
  #[account(mut)]
  pub zo_program_state: AccountLoader<'info, State>,
  #[account(mut)]
  pub state_signer: UncheckedAccount<'info>,
  #[account(mut, address = zo_program_state.load()?.cache)]
  pub cache: AccountLoader<'info, Cache>,
  #[account(address = zod_state.load()?.admin)]
  pub admin: Signer<'info>,
  #[account(mut, address = zo_program_margin.load()?.control)]
  pub control: AccountLoader<'info, Control>,
  #[account(
    mut,
    constraint = {msg!("test1"); token_account.owner == *admin.key},
  )]
  pub token_account: Account<'info, TokenAccount>,
  #[account(
    mut,
    constraint = {msg!("test2"); zo_vault.owner == *state_signer.to_account_info().key},
    constraint = zo_vault.mint == token_account.mint
  )]
  pub zo_vault: Account<'info, TokenAccount>,
  #[account(
    mut,
    constraint = {msg!("test3"); zod_vault.owner == *zod_state.to_account_info().key},
    constraint = zod_vault.mint == token_account.mint
  )]
  pub zod_vault: Account<'info, TokenAccount>,
  pub token_program: Program<'info, Token>,
}

pub fn process(cx: Context<ZodReduceInsurance>, amount: u64) -> ProgramResult {
  msg!("Instruction: ZodReduceInsurance");

  let zo_program_state = &cx.accounts.zo_program_state;
  let zod_state = &cx.accounts.zod_state;
  let token_acc = &cx.accounts.token_account;

  let col_index = zo_program_state
    .load()?
    .get_collateral_index(&token_acc.mint)
    .ok_or(ErrorCode::CollateralDoesNotExist)?;
  assert!(zo_program_state.load()?.vaults[col_index] == cx.accounts.zo_vault.key());
  assert!(col_index == 0);

  let amount_i80: I80F48 = I80F48::from_num(amount);

  {
    zod_state.load_mut()?.mutate_insurance(-(amount as i64))?;
  }

  let amount_to_withdraw: u64 = amount_i80.floor().to_num();

  let zod_state_seeds = &[b"zodv12".as_ref(), &[zod_state.load()?.zod_state_nonce]];

  let signer = &[&zod_state_seeds[..]];

  let cpi_program = cx.accounts.zo_program.to_account_info();

  let cpi_accounts = Withdraw {
    state: cx.accounts.zo_program_state.to_account_info(),
    state_signer: cx.accounts.state_signer.to_account_info(),
    cache: cx.accounts.cache.to_account_info(),
    authority: cx.accounts.zod_state.to_account_info(),
    margin: cx.accounts.zo_program_margin.to_account_info(),
    control: cx.accounts.control.to_account_info(),
    token_account: cx.accounts.zod_vault.to_account_info(),
    vault: cx.accounts.zo_vault.to_account_info(),
    token_program: cx.accounts.token_program.to_account_info(),
  };

  let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

  zo::cpi::withdraw(cpi_ctx, false, amount)?;

  system_program_utils::token_transfer(
    cx.accounts.token_program.to_account_info().clone(),
    cx.accounts.zod_vault.to_account_info().clone(),
    cx.accounts.token_account.to_account_info().clone(),
    cx.accounts.zod_state.to_account_info().clone(),
    amount_to_withdraw,
    Some(signer),
  )?;

  Ok(())
}
