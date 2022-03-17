use anchor_lang::prelude::*;
use fixed::types::I80F48;

#[derive(PartialEq, Copy, Clone)]
pub enum FractionType {
  Maintenance,
  Initial,
  Cancel,
}

#[derive(
  AnchorDeserialize, AnchorSerialize, Copy, Clone, PartialEq, Eq, Debug, PartialOrd, Ord, Default,
)]
pub struct Symbol {
  data: [u8; 24],
}

// Custom wrap type is required because current anchor IDL does not support native Fixed type
#[derive(
  AnchorDeserialize, AnchorSerialize, Copy, Clone, PartialEq, Eq, Debug, PartialOrd, Ord,
)]
pub struct WrappedI80F48 {
  pub data: i128,
}

impl Default for WrappedI80F48 {
  fn default() -> Self {
    Self {
      data: I80F48::ZERO.to_bits(),
    }
  }
}

impl std::fmt::Display for WrappedI80F48 {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
    I80F48::from_bits(self.data).fmt(f)
  }
}

impl WrappedI80F48 {
  pub fn zero() -> Self {
    Self::from(I80F48::ZERO)
  }
}

impl From<I80F48> for WrappedI80F48 {
  fn from(i: I80F48) -> Self {
    Self { data: i.to_bits() }
  }
}

impl From<WrappedI80F48> for I80F48 {
  fn from(i: WrappedI80F48) -> Self {
    Self::from_bits(i.data)
  }
}

macro_rules! from_impl {
  ( $( $T:ty ),* ) => {
      $(
          impl From<$T> for WrappedI80F48 {
              fn from(x: $T) -> Self {
                  Self::from(I80F48::from_num(x))
              }
          }

          impl From<WrappedI80F48> for $T {
              fn from(x: WrappedI80F48) -> $T {
                  I80F48::from_bits(x.data).to_num::<$T>()
              }
          }
      )*
  }
}

from_impl! { u8, i8, u16, i16, u32, i32, u64, i64, f32, f64 }

#[allow(non_snake_case)]
macro_rules! WrapI80F48 {
  ($a:expr) => {{
    WrappedI80F48::from(I80F48::from_num($a))
  }};
}
pub(crate) use WrapI80F48;
