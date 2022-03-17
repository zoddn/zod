use anchor_lang::prelude::*;

pub fn get_current_time() -> Result<u64, ProgramError> {
    Ok(Clock::get()?.unix_timestamp as u64)
}
