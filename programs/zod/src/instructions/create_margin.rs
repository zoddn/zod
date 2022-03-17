use anchor_lang::prelude::*;
use anchor_spl::token::{self, *};
use fixed::types::I80F48;
use std::mem::size_of;
use zo::{self, config::DEBUG_LOG, cpi::accounts::*, program::ZoAbi as Zo, *};

use crate::state::*;
use crate::zodTypes::WrappedI80F48;

#[derive(Accounts)]
#[instruction(nonce: u8)]
pub struct CreateZodMargin<'info> {
  pub zod_state: AccountLoader<'info, ZodState>,
  pub payer: Signer<'info>,
  pub authority: Signer<'info>,
  #[account(
      init,
      seeds = [authority.key.as_ref(), zod_state.key().as_ref(), b"zodmarginv2".as_ref()],
      bump = nonce,
      payer = payer,
      space = 8 + size_of::<ZodMargin>()
  )]
  pub margin: AccountLoader<'info, ZodMargin>,
  #[account(mut)]
  pub token_account: UncheckedAccount<'info>,
  pub token_program: Program<'info, Token>,
  #[account(
    mut,
    constraint = {mint.key() == zod_state.load()?.zod_token_info.mint},
  )]
  pub mint: Account<'info, Mint>,
  pub rent: Sysvar<'info, Rent>,
  pub system_program: Program<'info, System>,
}

pub fn process(cx: Context<CreateZodMargin>, nonce: u8) -> ProgramResult {
  msg!("Instruction: CreateZodMargin");

  let zod_margin = &mut cx.accounts.margin.load_init()?;
  zod_margin.nonce = nonce;
  zod_margin.authority = *cx.accounts.authority.to_account_info().key;
  zod_margin.collateral = [WrappedI80F48::from(I80F48::ZERO); MAX_COLLATERALS as usize];
  zod_margin.zod_balance = WrappedI80F48::from(0);

  let cpi_program = cx.accounts.token_program.to_account_info();
  let cpi_accounts = InitializeAccount {
    account: cx.accounts.token_account.to_account_info(),
    mint: cx.accounts.mint.to_account_info(),
    authority: cx.accounts.authority.to_account_info(),
    rent: cx.accounts.rent.to_account_info(),
  };
  let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
  token::initialize_account(cpi_ctx)?;

  zod_margin.zod_token_account = cx.accounts.token_account.key();

  msg!("{}/AUTH/{}", DEBUG_LOG, zod_margin.authority,);

  Ok(())
}
