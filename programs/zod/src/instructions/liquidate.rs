use anchor_lang::prelude::*;
use anchor_lang::solana_program::log::sol_log_compute_units;
use anchor_spl::token::{self, *};
use az::CheckedAs;
use common::SafeOp;
use common::{system_program_utils, time};
use fixed::types::I80F48;
use zo::config::{SPOT_INITIAL_MARGIN_REQ, SPOT_MAINT_MARGIN_REQ};

use crate::state::*;
use zo::errors::ErrorCode;
use zo::{self, config::DEBUG_LOG, cpi::accounts::*, program::ZoAbi as Zo, *};

#[derive(Accounts)]
pub struct LiquidateZodPosition<'info> {
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

pub fn process(
  cx: Context<LiquidateZodPosition>,
  asset_transfer_amount: u64,
  _mock_col_price: Option<u64>,
) -> ProgramResult {

  msg!("Instruction: LiquidateZodPosition");

  #[cfg(feature = "devnet")]
  msg!("mock collateral price: {:?}", _mock_col_price);

  let zo_program_state = cx.accounts.zo_program_state.load()?;
  let zod_state = &cx.accounts.zod_state;
  let cache = &cx.accounts.cache;

  let quote_col_index = zo_program_state
    .get_collateral_index(cx.accounts.quote_mint.to_account_info().key)
    .ok_or(ErrorCode::InvalidMint)?;

  let current_time = time::get_current_time()?;
  let liqee_margin = &cx.accounts.liqee_zod_margin;
  let liqor_margin = &cx.accounts.liqor_zod_margin;

  let zod_balance: I80F48 = liqee_margin
    .load()?
    .get_actual_zod_balance(zod_state.load()?.soc_loss_multiplier.into())?;
  assert!(zod_balance > 0);

  let omf = liqee_margin.load()?.get_omf(
    &zo_program_state,
    &cache.load()?,
    &zod_state.load()?,
    true,
    current_time,
    _mock_col_price,
  )?;

  let imf = liqee_margin.load()?.get_imf(&zod_state.load()?)?;

  let mmf = liqee_margin.load()?.get_mmf(&zod_state.load()?)?;

  //making sure that collateral can be liquidated
  assert!(omf < mmf);

  let liq_fee = ((1000 + zod_state.load()?.zod_token_info.liq_fee) as f64
    / (1000 - zo_program_state.collaterals[quote_col_index].liq_fee) as f64)
    - 1.0;
  let num_lf =
    -1000.0 + zo_program_state.collaterals[quote_col_index].weight as f64 * (1.0 + liq_fee);

  msg!("liq_fee: {}", liq_fee);

  let max_assets_transfer = liqee_margin.load()?.get_max_reducible(&zod_state.load()?, num_lf, imf, omf)?;

  msg!("asset_transfer_amount: {:?}", asset_transfer_amount);

  let mut assets_from_liqor = I80F48::from_num(max_assets_transfer)
    .min(zod_balance)
    .min(I80F48::from_num(asset_transfer_amount));

  // get quote to transfer
  // convert asset_price (smolUSD per asset) to asset_quote_price (quote per assets)
  let quote_oracle = cache
    .load()?
    .get_oracle(&zo_program_state.collaterals[quote_col_index].oracle_symbol)?
    .clone();
  let mut quote_price: I80F48 = quote_oracle.price.into();
  require!(!quote_oracle.is_stale(current_time), OracleCacheStale);

  #[cfg(feature = "devnet")]
  if let Some(_mock_col_price) = _mock_col_price {
    quote_price = I80F48::from_num(_mock_col_price as f64 / 1000.0);
  }
  msg!("quote_price: {}", quote_price);
  let asset_quote_price: I80F48 = I80F48::ONE.safe_div(quote_price)?;

  let pre_fee_quote = assets_from_liqor
    .safe_mul(asset_quote_price)?
    .floor()
    .to_num::<i64>();
  msg!("pre_fee_quote {}", pre_fee_quote);
  msg!("asset_quote_price {}", asset_quote_price);
  let mut quote_to_liqor: i64 = (pre_fee_quote as f64 * (1f64 + liq_fee))
    .checked_as()
    .unwrap();
  msg!("quote fee multiplier: {}", 1f64 + liq_fee as f64);
  //quote_to_liqor = -quote_to_liqor;
  msg!("quote_to_liqor {}", quote_to_liqor);

  // max amount of quote col that liqee has
  let max_quote_col: i64 = liqee_margin
    .load()?
    .get_actual_collateral(
      quote_col_index,
      cache.load()?.borrow_cache[quote_col_index]
        .supply_multiplier
        .into(),
    )?
    .floor()
    .to_num::<i64>();
  msg!(
    "max_quote_col and alice quote col before: {}",
    max_quote_col
  );

  if quote_to_liqor > max_quote_col {
    quote_to_liqor = max_quote_col;
    // todo: check if the fee math here is right
    msg!("changing assets_from_liqor");
    assets_from_liqor = I80F48::from_num(max_quote_col)
      .safe_div(asset_quote_price.safe_mul(I80F48::from_num(1f64 + liq_fee))?)?;
    msg!("assets_from_liqor_is_now_max_col: {}", assets_from_liqor);
  }

  msg!("assets_from_liqor {:?}", assets_from_liqor);

  //mutate margins
  liqee_margin.load_mut()?.zod_mutate(
    -assets_from_liqor,
    zod_state.load()?.soc_loss_multiplier.into(),
  )?;
  zod_state
    .load_mut()?
    .mutate_zod_borrowed(-assets_from_liqor)?;
  liqee_margin.load_mut()?.mutate(
    quote_col_index,
    -I80F48::from_num(quote_to_liqor),
    cache.load()?.borrow_cache[quote_col_index]
      .supply_multiplier
      .into(),
    cache.load()?.borrow_cache[quote_col_index]
      .borrow_multiplier
      .into(),
  )?;

  msg!(
    "bob zod token account amount before {:?}",
    cx.accounts.token_account.amount
  );
  let burn_cpi_program = cx.accounts.token_program.to_account_info();
  let burn_cpi_accounts = Burn {
    mint: cx.accounts.zod_mint.to_account_info(),
    to: cx.accounts.token_account.to_account_info(),
    authority: cx.accounts.liqor.to_account_info(),
  };

  let burn_cpi_ctx = CpiContext::new(burn_cpi_program, burn_cpi_accounts);

  let assets_from_liqor_u64 = assets_from_liqor.floor().to_num::<u64>();

  token::burn(burn_cpi_ctx, assets_from_liqor_u64)?;

  liqor_margin.load_mut()?.mutate(
    quote_col_index,
    I80F48::from_num(quote_to_liqor),
    cache.load()?.borrow_cache[quote_col_index]
      .supply_multiplier
      .into(),
    cache.load()?.borrow_cache[quote_col_index]
      .borrow_multiplier
      .into(),
  )?;

  msg!("mf after liquidation:");
  liqee_margin.load()?.get_imf(&zod_state.load()?)?;
  liqee_margin.load()?.get_mmf(&zod_state.load()?)?;
  liqee_margin.load()?.get_omf(
    &zo_program_state,
    &cache.load()?,
    &zod_state.load()?,
    true,
    current_time,
    _mock_col_price,
  )?;

  Ok(())
}
