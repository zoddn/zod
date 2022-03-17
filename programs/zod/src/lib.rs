use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;
pub mod zodTypes;

use instructions::*;
use zo::errors::ErrorCode;

declare_id!("HjBqgYKdav882K1bbnoaSr3QmZ9mxQcpmAFrvrAKjrpL");

#[program]
pub mod zod {
    use super::*;
    pub fn init_zod_state(
        cx: Context<InitZodState>,
        zod_state_nonce: u8,
        zo_program_nonce: u8,
    ) -> ProgramResult {
        instructions::init_state::process(cx, zod_state_nonce, zo_program_nonce)
    }

    pub fn create_zod_margin(cx: Context<CreateZodMargin>, nonce: u8) -> ProgramResult {
        instructions::create_margin::process(cx, nonce)
    }

    pub fn add_vaults(cx: Context<AddVaults>) -> ProgramResult {
        instructions::add_vaults::process(cx)
    }

    pub fn zod_deposit(cx: Context<ZodDeposit>, amount: u64) -> ProgramResult {
        instructions::deposit::process(cx, amount)
    }

    pub fn zod_withdraw(cx: Context<ZodWithdraw>, amount: u64) -> ProgramResult {
        instructions::withdraw::process(cx, amount)
    }

    pub fn zod_mint(cx: Context<ZodMint>, amount: u64) -> ProgramResult {
        instructions::mint::process(cx, amount)
    }

    pub fn zod_burn(cx: Context<ZodBurn>, amount: u64) -> ProgramResult {
        instructions::burn::process(cx, amount)
    }

    pub fn liquidate_zod_position(cx: Context<LiquidateZodPosition>, amount: u64, _mock_col_price: Option<u64>) -> ProgramResult {
        instructions::liquidate::process(cx, amount, _mock_col_price)
    }

    pub fn zod_add_insurance(cx: Context<ZodAddInsurance>, amount: u64) -> ProgramResult {
        instructions::add_insurance::process(cx, amount)
    }

    pub fn zod_reduce_insurance(cx: Context<ZodReduceInsurance>, amount: u64) -> ProgramResult {
        instructions::reduce_insurance::process(cx, amount)
    }

    pub fn zod_settle_bankruptcy(cx: Context<SettleZodBankruptcy>, _mock_col_price: Option<u64>) -> ProgramResult {
        instructions::settle_bankruptcy::process(cx, _mock_col_price)
    }
}
