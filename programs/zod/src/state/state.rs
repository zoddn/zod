use anchor_lang::prelude::*;
use common::SafeOp;
use fixed::types::I80F48;
use fixed_macro::types::I80F48;
use zo::{self, config::DEBUG_LOG, cpi::accounts::*, program::ZoAbi as Zo, *, accounts::*};
use zo::errors::ErrorCode;

#[account(zero_copy)]
pub struct ZodState {
  pub zod_state_nonce: u8,
  pub zo_margin_nonce: u8,
  pub admin: Pubkey,
  pub zo_program_state: Pubkey,
  pub zo_program_margin: Pubkey,
  pub insurance: u64,          // in smol usd
  pub fees_accrued: [u64; 25], // in smol usd
  pub total_collaterals: u16,
  pub vaults: [Pubkey; 25],
  pub zod_token_mint: Pubkey,
  pub zod_token_info: ZodCollateralInfo,
  pub soc_loss_multiplier: WrappedI80F48,
  pub total_zod_borrowed: WrappedI80F48,
}

//had trouble getting collateral info from zo
#[zero_copy]
#[derive(PartialEq, Default)]
pub struct ZodCollateralInfo {
  pub mint: Pubkey,
  pub oracle_symbol: Symbol,
  pub decimals: u8,
  pub weight: u16,  //  in permil
  pub liq_fee: u16, // in permil

  // borrow lending info
  pub optimal_util: u16, // in permil
  pub optimal_rate: u16, // in permil
  pub max_rate: u16,     // in permil
  pub og_fee: u16,       // in bps

  // swap info
  pub serum_open_orders: Pubkey,
}

impl ZodCollateralInfo {
  pub fn is_empty(&self) -> bool {
    self.mint == Pubkey::default()
  }
}

impl ZodState {
  pub fn mutate_insurance(&mut self, amount: i64) -> Result<(), ErrorCode> {
    msg!("Zod State Instruction: mutating insurance");
    msg!("Zod State Instruction: insurance amount before: {}", self.insurance);
    let initial_insurance = self.insurance as i64;
    require!(initial_insurance >= -amount, InsufficientInsurance);
    self.insurance = (initial_insurance + amount) as u64;
    msg!("Zod State Instruction: insurance amount after: {}", self.insurance);
    Ok(())
  }

  pub fn socialize_loss(&mut self, loss_per_zod_borrowed: I80F48) -> Result<(), ErrorCode> {
    msg!("Zod State Instruction: Socializing loss");
    let initial_soc_loss_multiplier: I80F48 = self.soc_loss_multiplier.into();
    self.soc_loss_multiplier = WrappedI80F48::from(
      (I80F48::ONE + loss_per_zod_borrowed).safe_mul(initial_soc_loss_multiplier)?,
    );
    Ok(())
  }

  pub fn get_actual_zod_borrowed(&self) -> I80F48 {
    let borrow: I80F48 = self.total_zod_borrowed.into();
    let multiplier: I80F48 = self.soc_loss_multiplier.into();
    borrow.safe_mul(multiplier).unwrap()
  }

  pub fn mutate_zod_borrowed(&mut self, amount: I80F48) -> Result<(), ErrorCode> {
    msg!("Zod State Instruction: mutating total zod borrowed");
    let initial_bor: I80F48 = self.total_zod_borrowed.into();
    let bor_multiplier: I80F48 = self.soc_loss_multiplier.into();
    let actual_bor = initial_bor.safe_mul(bor_multiplier)?;
    msg!("Zod State Instruction: total zod borrowed before: {}", actual_bor);
    let final_bor = actual_bor + amount.max(-actual_bor);
    msg!("Zod State Instruction: total zod borrowed after: {}", final_bor);
    let adjusted_final_bor = final_bor.safe_div(bor_multiplier)?;
    self.total_zod_borrowed = WrappedI80F48::from(adjusted_final_bor);
    Ok(())
  }

  pub fn vaults(&self) -> [Pubkey; 25] {
    self.vaults
  }

  fn key(&self) {
    self.key();
  }
}
