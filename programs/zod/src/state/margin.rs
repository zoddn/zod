use crate::state::*;
use crate::zodTypes::WrappedI80F48;
use anchor_lang::prelude::*;
use common::math::{safe_add_i80f48, safe_div_i80f48, safe_mul_i80f48};
use common::SafeOp;
use fixed::types::I80F48;
use std::cell::Ref;
use zo::config::DUST_THRESHOLD;
use zo::errors::ErrorCode;
use zo::{self, config::DEBUG_LOG, cpi::accounts::*, program::ZoAbi as Zo, *};

#[account(zero_copy)]
pub struct ZodMargin {
    pub nonce: u8,
    pub authority: Pubkey,
    pub collateral: [WrappedI80F48; 25], // mapped to the state collaterals array, divided by entry ir_index
    pub zod_token_account: Pubkey,
    pub zod_balance: WrappedI80F48,
}

#[derive(Clone, Copy)]
enum MfReturnOption {
    Imf,
    Mmf,
    Cancel,
    Both,
}

impl ZodMargin {
    pub fn bankrupt(&mut self) -> Result<(), ErrorCode> {
        self.zod_balance = WrappedI80F48::zero();
        Ok(())
    }

    pub fn mutate(
        &mut self,
        index: usize,
        amount: I80F48,
        supply_multiplier: I80F48,
        borrow_multiplier: I80F48,
    ) -> Result<(), ErrorCode> {
        msg!("Margin Instruction: mutating collateral");

        let initial_col: I80F48 = self.collateral[index].into();
        let actual_col = if initial_col > 0 {
            safe_mul_i80f48(initial_col, supply_multiplier)
        } else {
            safe_mul_i80f48(initial_col, borrow_multiplier)
        };
        msg!("Margin Instruction: amount of collateral before: {}", actual_col);
        let final_col = safe_add_i80f48(actual_col, amount);
        msg!("Margin Instruction: amount of collateral after: {}", final_col);
        let adjusted_final_col = if final_col > 0 {
            safe_div_i80f48(final_col, supply_multiplier)
        } else {
            safe_div_i80f48(final_col, borrow_multiplier)
        };
        self.collateral[index] = WrappedI80F48::from(adjusted_final_col);

        Ok(())
    }

    pub fn get_omf(
        &self,
        state: &Ref<State>,
        cache: &Ref<Cache>,
        zod_state: &Ref<ZodState>,
        is_weighted: bool,
        current_time: u64,
        _mock_col_price: Option<u64>,
    ) -> Result<I80F48, ErrorCode> {
        msg!("Margin Instruction: getting omf");
        let total_collateral_value = self.get_total_collateral_value(
            state,
            cache,
            is_weighted,
            current_time,
            _mock_col_price,
        )?;

        let zod_balance: I80F48 =
            self.get_actual_zod_balance(zod_state.soc_loss_multiplier.into())?;

        let omf = total_collateral_value
            .safe_sub(zod_balance)?
            .safe_mul(1000)?;
        msg!("Margin Instruction: omf: {}", omf);
        Ok(omf)
    }

    pub fn get_imf(&self, zod_state: &Ref<ZodState>) -> Result<i64, ErrorCode> {
        msg!("Margin Instruction: getting imf");
        let zod_base_imf = (SPOT_INITIAL_MARGIN_REQ as u32 / zod_state.zod_token_info.weight as u32)
            as u16
            - 1000u16;
        msg!("Margin Instruction: zod_base_imf: {}", zod_base_imf);

        let zod_balance: I80F48 =
            self.get_actual_zod_balance(zod_state.soc_loss_multiplier.into())?;

        let imf = (zod_base_imf as i64).safe_mul(zod_balance)?;
        msg!("Margin Instruction: imf: {}", imf);
        Ok(imf)
    }

    pub fn get_mmf(&self, zod_state: &Ref<ZodState>) -> Result<i64, ErrorCode> {
        msg!("Margin Instruction: getting mmf");
        let zod_base_mmf = (SPOT_MAINT_MARGIN_REQ as u32 / zod_state.zod_token_info.weight as u32)
            as u16
            - 1000u16;
        msg!("Margin Instruction: zod_base_mmf: {}", zod_base_mmf);

        let zod_balance: I80F48 =
            self.get_actual_zod_balance(zod_state.soc_loss_multiplier.into())?;

        let mmf = (zod_base_mmf as i64).safe_mul(zod_balance)?;
        msg!("Margin Instruction: mmf: {}", mmf);
        Ok(mmf)
    }

    pub fn get_max_reducible(&self, zod_state: &Ref<ZodState>, num_lf: f64, imf: i64, omf: I80F48) -> Result<i64, ErrorCode> {
        //calculating max reducible, the amount liqor can buy in terms of zod to get liqee to imf (everything multpllied by total open position)
        //OMF increase = assets transfered * asset price - ( assets transfered * asset price / quote price * (1+liqfee) ) * quote price
        //             = assets transfered * (-fee)
        //IMF decrease = assets transfered * asset base IMF
        //aseets transferred needs to be such that,
        //IMF - IMF decrease = OMF + OMF increase
        //IMF - OMF = IMF decrease + OMF increase
        //          = assets transfered * asset base IMF - assets transfered * fee
        //thus,
        //assets transfered = (IMF - OMF) / (asset base IMF - fee)

        let zod_base_imf = (SPOT_INITIAL_MARGIN_REQ as u32
            / zod_state.zod_token_info.weight as u32) as u16
            - 1000u16;

        msg!("Margin Instruction: num_lf {}", num_lf);
        assert!(I80F48::from_num(zod_base_imf) > I80F48::from_num(num_lf));
        let numerator = imf.safe_sub(omf)?;
        msg!("Margin Instruction: numerator {}", numerator);
        let denominator = (I80F48::from_num(zod_base_imf) - I80F48::from_num(num_lf)).to_num::<i64>();
        msg!("Margin Instruction: denomintor {}", denominator);
        let max_assets_transfer = numerator.safe_div(denominator)?;

        msg!("Margin Instruction: max_assets_transfer: {:?}", max_assets_transfer);

        Ok(max_assets_transfer)

    }

    pub fn get_actual_collateral(
        &self,
        index: usize,
        supply_multiplier: I80F48,
    ) -> Result<I80F48, ErrorCode> {
        msg!("Margin Instruction: getting collateral amount");
        let initial_col: I80F48 = self.collateral[index].into();
        msg!(
            "Margin Instruction: initial_col: {}, supply_mutiplier: {}",
            initial_col,
            supply_multiplier
        );
        let actual_col = safe_mul_i80f48(initial_col, supply_multiplier);
        msg!("Margin Instruction: actual collateral: {}", actual_col);
        Ok(actual_col)
    }

    pub fn get_total_collateral_value(
        &self,
        state: &Ref<State>,
        cache: &Ref<Cache>,
        is_weighted: bool,
        current_time: u64,
        _mock_col_price: Option<u64>,
    ) -> Result<I80F48, ErrorCode> {
        msg!("Margin Instruction: getting total collateral value");

        let mut sum = I80F48::ZERO; // in smol usd

        let max_col = state.total_collaterals as usize;
        for (i, v) in { self.collateral }.iter().enumerate() {
            if !(i < max_col) {
                break;
            }

            //msg!("collateral_index {:?}", i);

            let info = &state.collaterals[i];
            let borrow = &cache.borrow_cache[i];

            if WrappedI80F48::zero() == *v || info.is_empty() {
                continue;
            }

            let v: I80F48 = self.get_actual_collateral(i, borrow.supply_multiplier.into())?;

            let oracle = cache.get_oracle(&info.oracle_symbol)?;
            let mut price: I80F48 = oracle.price.into();
            let stale = oracle.is_stale(current_time);
            assert!(!stale);

            #[cfg(feature = "devnet")]
            if let Some(_mock_col_price) = _mock_col_price {
                msg!("Margin Instruction: price before: {}", price);
                price = I80F48::from_num(_mock_col_price as f64 / 1000.0);
                msg!("Margin Instruction: mocking price: {}", price);
            }

            //msg!("weight {:?}", info.weight);
            // Price is only weighted when collateral is non-negative.
            let weighted_price = match is_weighted && v >= 0 {
                true => price.safe_mul(I80F48::from_num(info.weight as f64 / 1000.0))?,
                false => price,
            };

            let value = weighted_price.safe_mul(v)?;

            msg!(
                "Margin Instruction: collateral index: {}, collateral amount: {}, collateral value: {}",
                i,
                v,
                value
            );

            sum = sum.safe_add(value)?;
        }

        Ok(sum)
    }

    pub fn get_total_collateral_value_i64(
        &self,
        state: &Ref<State>,
        cache: &Ref<Cache>,
        is_weighted: bool,
        current_time: u64,
        _mock_col_price: Option<u64>,
    ) -> Result<i64, ErrorCode> {
        self.get_total_collateral_value(state, cache, is_weighted, current_time, _mock_col_price)
            .map(|x| x.floor().checked_to_num().unwrap())
    }

    pub fn zod_mutate(
        &mut self,
        amount: I80F48,
        borrow_multiplier: I80F48,
    ) -> Result<(), ErrorCode> {
        msg!("Margin Instruction: mutating zod balance");
        let actual_amount_borrowed = safe_mul_i80f48(self.zod_balance.into(), borrow_multiplier);
        msg!("Margin Instruction: zod balance before {}", actual_amount_borrowed);
        let final_amount_borrowed = safe_add_i80f48(actual_amount_borrowed, amount);
        msg!("Margin Instruction: zod balance after {}", final_amount_borrowed);
        let adjusted_final_amount_borrowed =
            safe_div_i80f48(final_amount_borrowed, borrow_multiplier);
        self.zod_balance = WrappedI80F48::from(adjusted_final_amount_borrowed);
        Ok(())
    }

    pub fn has_no_col_above_dust(
        &self,
        col_infos: &[CollateralInfo; 25],
        max_col: usize,
        cache: &Ref<Cache>,
        current_time: u64,
        _mock_col_price: Option<u64>,
    ) -> Result<bool, ErrorCode> {
        msg!("Margin Instruction: checking if there is collateral above dust");
        let mut has_no_col_above_dust = true;

        for (i, v) in { self.collateral }.iter().enumerate() {
            msg!("Margin Instruction: collateral index: {}", i);
            if !(i < max_col) {
                break;
            }

            let info = &col_infos[i];
            let borrow = &cache.borrow_cache[i];

            if WrappedI80F48::zero() == *v || info.is_empty() {
                continue;
            }

            let v: I80F48 = self.get_actual_collateral(i, borrow.supply_multiplier.into())?;

            // todo: should this be weighted or no? currently not weighted
            let oracle = cache.get_oracle(&info.oracle_symbol)?;
            let mut price: I80F48 = oracle.price.into();
            require!(!oracle.is_stale(current_time), OracleCacheStale);

            #[cfg(feature = "devnet")]
            if let Some(_mock_col_price) = _mock_col_price {
                msg!("Margin Instruction: price before: {}", price);
                price = I80F48::from_num(_mock_col_price as f64 / 1000.0);
                msg!("Margin Instruction: mocking price: {}", price);
            }

            let value = price.safe_mul(v)?.floor().to_num::<i64>();

            msg!(
                "Margin Instruction: collateral index: {}, collateral amount: {}, collateral value: {}",
                i,
                v,
                value
            );

            if value > DUST_THRESHOLD {
                has_no_col_above_dust = false;
                break;
            }
        }

        Ok(has_no_col_above_dust)
    }

    pub fn get_actual_zod_balance(&self, soc_loss_multiplier: I80F48) -> Result<I80F48, ErrorCode> {
        msg!("Margin Instruction: getting total zod balance");

        msg!(
            "Margin Instruction: initial_zod_balance: {}, soc_loss_mutiplier: {}",
            self.zod_balance,
            soc_loss_multiplier
        );

        let balance = safe_mul_i80f48(
            self.zod_balance.into(),
            soc_loss_multiplier,
        );

        msg!("Margin Instruction: zod_balance: {}", balance);

        Ok(balance)
    }
}

#[test]
fn get_state_size() {
    use std::mem::size_of;

    println!("size of state: {}", size_of::<ZodMargin>());
}
