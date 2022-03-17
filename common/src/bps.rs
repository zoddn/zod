use crate::SafeOp;
use fixed::types::I80F48;

pub fn get_bps(bps: u16) -> I80F48 {
    I80F48::from_num(bps)
        .safe_div(I80F48::from_num(10_000))
        .unwrap()
}
