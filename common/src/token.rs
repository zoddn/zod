use std::{convert::TryFrom, io, ops};

use borsh::{de::BorshDeserialize, ser::BorshSerialize};

use crate::error::ErrorCode;

pub enum TokenType {
    Usdc = 0,
    LENGTH,
}

pub const TOKEN_COUNT: usize = TokenType::LENGTH as usize;

#[derive(Default, Clone)]
pub struct TokenMap<T>([T; TOKEN_COUNT]);

impl TryFrom<u8> for TokenType {
    type Error = ErrorCode;

    fn try_from(x: u8) -> Result<Self, Self::Error> {
        match x {
            0 => Ok(TokenType::Usdc),
            _ => Err(Self::Error::MathFailure),
        }
    }
}

impl From<TokenType> for u8 {
    fn from(x: TokenType) -> u8 {
        return x as u8;
    }
}

impl<T> From<[T; TOKEN_COUNT]> for TokenMap<T> {
    fn from(x: [T; TOKEN_COUNT]) -> Self {
        TokenMap::<T>(x)
    }
}

impl<T> ops::Index<TokenType> for TokenMap<T> {
    type Output = T;
    fn index(&self, idx: TokenType) -> &Self::Output {
        &self.0[u8::from(idx) as usize]
    }
}

impl<T> ops::IndexMut<TokenType> for TokenMap<T> {
    fn index_mut(&mut self, idx: TokenType) -> &mut Self::Output {
        &mut self.0[u8::from(idx) as usize]
    }
}

impl<T> BorshSerialize for TokenMap<T>
where
    T: BorshSerialize,
{
    fn serialize<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        self.0.serialize(writer)
    }
}

impl<T> BorshDeserialize for TokenMap<T>
where
    T: BorshDeserialize + Copy + Default,
{
    fn deserialize(buf: &mut &[u8]) -> io::Result<Self> {
        Ok(TokenMap::<T>(BorshDeserialize::deserialize(buf)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_try_from() -> Result<(), ErrorCode> {
        for i in 0..(TOKEN_COUNT as u8) {
            let x: u8 = TokenType::try_from(i)?.into();
            assert_eq!(i, x);
        }
        Ok(())
    }

    #[test]
    fn token_map() {
        let mut m = TokenMap::<u64>::default();
        assert_eq!(m[TokenType::Usdc], 0);

        m[TokenType::Usdc] = 10;
        assert_eq!(m[TokenType::Usdc], 10);
    }

    #[test]
    #[should_panic]
    fn token_map_out_of_bounds() {
        let m = TokenMap::<u64>::default();
        m[TokenType::LENGTH];
    }
}
