use anchor_lang::prelude::*;

#[error]
pub enum ErrorCode {
    #[msg("A problem occurred when running the memo program.")]
    MemoFailure,
    #[msg("Oopsie doopsie, math problem encountered.")]
    MathFailure,
    #[msg("Shucks, something didn't convert properly.")]
    ConversionFailure,
    #[msg("Uh oh, oracle encountered an issue when fetching accurate price.")]
    PriceOracleIssue,
    #[msg("The oracle is invalid.")]
    InvalidOracle,
}
