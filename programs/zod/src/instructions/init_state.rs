use anchor_lang::prelude::*;
use anchor_spl::token::{self, *};
use std::mem::size_of;

use std::convert::TryInto;

use crate::state::*;
use anchor_spl::token::*;
use zo::cpi::accounts::CreateMargin;
use zo::{self, config::DEBUG_LOG, cpi::accounts::*, program::ZoAbi as Zo, *};

#[derive(Accounts)]
#[instruction(zod_state_nonce: u8, zo_margin_nonce: u8)]
pub struct InitZodState<'info> {
  #[account(mut)]
  pub admin: Signer<'info>,
  #[account(
    init_if_needed,
    seeds = [b"zodv12".as_ref()],
    bump = zod_state_nonce,
    payer = admin,
    space = 8 + size_of::<ZodState>()
  )]
  pub zod_state: AccountLoader<'info, ZodState>,
  #[account(mut)]
  pub zo_program_state: AccountInfo<'info>,
  #[account(mut)]
  pub zo_program_margin: UncheckedAccount<'info>,
  pub zo_program: Program<'info, Zo>,
  #[account(mut)]
  pub control: UncheckedAccount<'info>,
  pub rent: Sysvar<'info, Rent>,
  pub zo_program_margin_rent: Sysvar<'info, Rent>,
  pub system_program: Program<'info, System>,
  pub token_program: Program<'info, Token>,
  #[account(mut)]
  pub mint: UncheckedAccount<'info>,
}

pub fn process(
  cx: Context<InitZodState>,
  zod_state_nonce: u8,
  zo_margin_nonce: u8,
) -> ProgramResult {
  msg!("Instruction: InitZodState");
  {
    let st = &mut cx.accounts.zod_state.load_init()?;
    st.zod_state_nonce = zod_state_nonce;
    st.zo_margin_nonce = zo_margin_nonce;
    st.admin = *cx.accounts.admin.to_account_info().key;
    st.zo_program_state = cx.accounts.zo_program_state.key();
    st.insurance = 0;
    st.total_collaterals = 25;
    st.zo_program_margin = cx.accounts.zo_program_margin.key();
    st.total_zod_borrowed = WrappedI80F48::from(0i8);
    st.soc_loss_multiplier = WrappedI80F48::from(1i8);
  }

  let zod_state_seeds = &[b"zodv12".as_ref(), &[zod_state_nonce]];

  //creating associated zo program margin
  let margin_cpi_program = cx.accounts.zo_program.to_account_info();
  let margin_cpi_accounts = CreateMargin {
    state: cx.accounts.zo_program_state.to_account_info(),
    payer: cx.accounts.admin.to_account_info(),
    authority: cx.accounts.zod_state.to_account_info(),
    margin: cx.accounts.zo_program_margin.to_account_info(),
    control: cx.accounts.control.to_account_info(),
    rent: cx.accounts.zo_program_margin_rent.to_account_info(),
    system_program: cx.accounts.system_program.to_account_info(),
  };

  let signers = &[&zod_state_seeds[..]];

  let margin_cpi_ctx =
    CpiContext::new_with_signer(margin_cpi_program, margin_cpi_accounts, signers);

  zo::cpi::create_margin(margin_cpi_ctx, zo_margin_nonce)?;

  let oracle_symbol: Symbol = String::from("ZOD").into();
  let weight: u16 = 900;
  let optimal_util: u16 = 700;
  let optimal_rate: u16 = 100;
  let max_rate: u16 = 1000;
  let liq_fee: u16 = 20;
  let og_fee: u16 = 10;
  let decimals: u8 = 6;

  let mint_cpi_program = cx.accounts.token_program.to_account_info();
  let mint_cpi_accounts = InitializeMint {
    mint: cx.accounts.mint.to_account_info(),
    rent: cx.accounts.rent.to_account_info(),
  };

  let mint_cpi_ctx = CpiContext::new_with_signer(mint_cpi_program, mint_cpi_accounts, signers);

  token::initialize_mint(
    mint_cpi_ctx,
    6,
    cx.accounts.zod_state.to_account_info().key,
    Some(cx.accounts.zod_state.to_account_info().key),
  )?;

  let zod_state = &mut cx.accounts.zod_state.load_init()?;
  zod_state.zod_token_info = ZodCollateralInfo {
    mint: *cx.accounts.mint.to_account_info().key,
    decimals,
    weight,
    oracle_symbol,
    optimal_util,
    optimal_rate,
    max_rate,
    liq_fee,
    serum_open_orders: Default::default(),
    og_fee,
  };

  msg!("{}/ZOD_STATE_INITIALIZED", DEBUG_LOG);

  Ok(())
}
