use anchor_lang::prelude::*;
use anchor_lang::solana_program::log::sol_log_compute_units;
use anchor_spl::token::{self, *};
use common::SafeOp;
use common::{system_program_utils, time};
use fixed::types::I80F48;
use zo::config::{SPOT_INITIAL_MARGIN_REQ, SPOT_MAINT_MARGIN_REQ};

use crate::state::*;
use zo::{self, config::DEBUG_LOG, cpi::accounts::*, program::ZoAbi as Zo, *};

#[derive(Accounts)]
pub struct SettleZodBankruptcy<'info> {
  #[account(mut)]
  pub zod_state: AccountLoader<'info, ZodState>,
  #[account(
    mut,
    constraint = {zo_program_state.key() == zod_state.load()?.zo_program_state},
  )]
  pub zo_program_state: AccountLoader<'info, State>,
  #[account(mut, address = zo_program_state.load()?.cache)]
  pub cache: AccountLoader<'info, Cache>,
  pub liqor: Signer<'info>,
  #[account(
    mut,
    seeds = [liqor.key.as_ref(), zod_state.key().as_ref(), b"zodmarginv2".as_ref()],
    bump = liqor_zod_margin.load()?.nonce
  )]
  pub liqor_zod_margin: AccountLoader<'info, ZodMargin>,
  #[account(
    mut,
    seeds = [liqee_zod_margin.load()?.authority.as_ref(), zod_state.key().as_ref(), b"zodmarginv2".as_ref()],
    bump =liqee_zod_margin.load()?.nonce
  )]
  pub liqee_zod_margin: AccountLoader<'info, ZodMargin>,
  #[account(
    mut,
    constraint = {zod_mint.key() == zod_state.load()?.zod_token_info.mint},
  )]
  pub zod_mint: Account<'info, Mint>,
  pub quote_mint: Account<'info, Mint>,
  #[account(
    mut,
    constraint = {token_account.owner == *liqor.key},
  )]
  pub token_account: Account<'info, TokenAccount>,
  pub token_program: Program<'info, Token>,
}

pub fn process(cx: Context<SettleZodBankruptcy>, _mock_col_price: Option<u64>) -> ProgramResult {
  #[cfg(feature = "devnet")]
  msg!("mock collateral price: {:?}", _mock_col_price);

  let mut zod_state = cx.accounts.zod_state.load_mut()?;
  let zo_program_state = cx.accounts.zo_program_state.load_mut()?;
  let cache = &cx.accounts.cache;
  let liqee_margin = &cx.accounts.liqee_zod_margin;
  let liqor_margin = &cx.accounts.liqor_zod_margin;
  let current_time = Clock::get()?.unix_timestamp as u64;

  let below_dust = liqee_margin.load()?.has_no_col_above_dust(
    &zo_program_state.collaterals,
    zo_program_state.total_collaterals as usize,
    &cache.load()?,
    current_time,
    _mock_col_price,
  )?;

  msg!("below_dust: {:?}", below_dust);

  assert!(below_dust);

  let assets_from_liqor = liqee_margin
    .load()?
    .get_actual_zod_balance(zod_state.soc_loss_multiplier.into())?;

  let pre_fee_quote = (assets_from_liqor).floor().to_num::<i64>();

  let quote_to_liqor = pre_fee_quote
    .safe_mul(1000i64 + zo_program_state.collaterals[0].liq_fee as i64)?
    .safe_div(1000i64)?;

  //do mutations
  liqee_margin.load_mut()?.bankrupt()?;

  zod_state.mutate_zod_borrowed(assets_from_liqor)?;
  let burn_cpi_program = cx.accounts.token_program.to_account_info();
  let burn_cpi_accounts = Burn {
    mint: cx.accounts.zod_mint.to_account_info(),
    to: cx.accounts.token_account.to_account_info(),
    authority: cx.accounts.liqor.to_account_info(),
  };

  let burn_cpi_ctx = CpiContext::new(burn_cpi_program, burn_cpi_accounts);

  let assets_to_liqor_u64: u64 = assets_from_liqor.to_num::<u64>();

  msg!("assets_to_liqor_u64: {}", assets_to_liqor_u64);

  token::burn(burn_cpi_ctx, assets_to_liqor_u64)?;

  liqor_margin.load_mut()?.mutate(
    0,
    I80F48::from_num(quote_to_liqor),
    cache.load()?.borrow_cache[0].supply_multiplier.into(),
    cache.load()?.borrow_cache[0].borrow_multiplier.into(),
  )?;

  msg!("quote_to_liqor {}", quote_to_liqor);

  // check max insurance fund amount
  if quote_to_liqor > zod_state.insurance as i64 {
    // socialize losses
    let insurance = zod_state.insurance as i64;
    zod_state.mutate_insurance(-insurance)?;
    msg!("Insurance refunded {}", insurance);

    // for every dollar supplied, socialize loss
    let zod_borrowed = zod_state.get_actual_zod_borrowed();
    msg!("insurance {}", insurance);
    msg!("zod_borrowed {}", zod_borrowed);
    let socialize_amount: I80F48 = I80F48::from_num(quote_to_liqor - insurance) / zod_borrowed;
    require!(socialize_amount < I80F48::ONE, MathFailure);

    // decrease supply multiplier

    zod_state.socialize_loss(socialize_amount)?;
    msg!("Socialized loss of {}", socialize_amount);
  } else {
    zod_state.mutate_insurance(-quote_to_liqor)?;
    msg!("Insurance refunded {}", quote_to_liqor);
  }

  Ok(())
}
