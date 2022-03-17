use anchor_lang::prelude::*;
use anchor_lang::solana_program::log::sol_log_compute_units;
use anchor_spl::token::{Token, TokenAccount, *};
use common::SafeOp;
use common::{system_program_utils, time};
use fixed::types::I80F48;
use zo::config::SPOT_INITIAL_MARGIN_REQ;

use crate::state::*;
use crate::zodTypes::WrappedI80F48;
use zo::errors::ErrorCode;
use zo::{self, config::DEBUG_LOG, cpi::accounts::*, program::ZoAbi as Zo, *};

use ::zo::cpi::accounts::Withdraw;

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct ZodWithdraw<'info> {
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
  #[account(mut)]
  pub authority: Signer<'info>,
  #[account(
    mut,
    seeds = [authority.key.as_ref(), zod_state.key().as_ref(), b"zodmarginv2".as_ref()],
    bump = zod_margin.load()?.nonce
  )]
  pub zod_margin: AccountLoader<'info, ZodMargin>,
  #[account(mut, address = zo_program_margin.load()?.control)]
  pub control: AccountLoader<'info, Control>,
  #[account(
    mut,
    constraint = {token_account.owner == *authority.key},
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
    constraint = {token_account.owner == *authority.key},
  )]
  pub zod_account: Box<Account<'info, TokenAccount>>,
  #[account(
    mut,
    constraint = {zod_vault.owner == *zod_state.to_account_info().key},
    constraint = zod_vault.mint == token_account.mint
  )]
  pub zod_vault: Account<'info, TokenAccount>,
  pub token_program: Program<'info, Token>,
}

pub fn process(cx: Context<ZodWithdraw>, amount: u64) -> ProgramResult {
  msg!("Instruction: ZodWithdraw");

  let zo_program_state = &cx.accounts.zo_program_state;
  let zod_state = &cx.accounts.zod_state;
  let token_acc = &cx.accounts.token_account;
  let zod_margin = &cx.accounts.zod_margin;
  let cache = &cx.accounts.cache;
  let zod_token_acc = &cx.accounts.zod_account;
  let current_time = time::get_current_time()?;

  let col_index = zo_program_state
    .load()?
    .get_collateral_index(&token_acc.mint)
    .ok_or(ErrorCode::CollateralDoesNotExist)?;
  assert!(zo_program_state.load()?.vaults[col_index] == cx.accounts.zo_vault.key());

  let actual_col = zod_margin.load()?.get_actual_collateral(
    col_index,
    cache.load()?.borrow_cache[col_index]
      .supply_multiplier
      .into(),
  )?;

  let amount_i80: I80F48 = I80F48::from_num(amount);
  assert!(actual_col > amount_i80);

  zod_margin.load_mut()?.mutate(
    col_index,
    -amount_i80,
    cache.load()?.borrow_cache[col_index]
      .supply_multiplier
      .into(),
    cache.load()?.borrow_cache[col_index]
      .borrow_multiplier
      .into(),
  )?;

  let zod_balance: I80F48 = zod_margin
  .load()?
  .get_actual_zod_balance(zod_state.load()?.soc_loss_multiplier.into())?;
  msg!("zod_balance: {}", zod_balance);

  let omf = zod_margin.load()?.get_omf(
    &zo_program_state.load()?,
    &cache.load()?,
    &zod_state.load()?,
    true,
    current_time,
    None,
  )?;

  let imf = zod_margin.load()?.get_imf(&zod_state.load()?)?;

  assert!(omf > imf);

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
