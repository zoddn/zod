use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program::{invoke, invoke_signed};
use anchor_lang::solana_program::system_instruction::{create_account, SystemInstruction};
use anchor_spl::token::{self, MintTo, Transfer};

pub fn new_account<'info>(
    from_info: &AccountInfo<'info>,
    to_info: &AccountInfo<'info>,
    lamports: u64,
    space: u64,
    owner: &AccountInfo<'info>,
    _system_program: &AccountInfo<'info>,
) -> ProgramResult {
    let create_acc_ix = create_account(from_info.key, to_info.key, lamports, space, owner.key);
    invoke(&create_acc_ix, &[from_info.clone(), to_info.clone()])?;
    Ok(())
}

pub fn transfer_lamports<'info>(
    lamports: u64,
    from_pubkey: &AccountInfo<'info>,
    to_pubkey: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
    signer: Option<&[&[&[u8]]; 1]>,
) -> ProgramResult {
    let account_metas = vec![
        AccountMeta::new(*from_pubkey.key, true),
        AccountMeta::new(*to_pubkey.key, false),
    ];
    let transfer_ix = Instruction::new_with_bincode(
        *system_program.key,
        &SystemInstruction::Transfer { lamports },
        account_metas,
    );

    match signer {
        Some(seeds) => invoke_signed(
            &transfer_ix,
            &[
                from_pubkey.clone(),
                to_pubkey.clone(),
                system_program.clone(),
            ],
            seeds,
        )?,
        None => invoke(
            &transfer_ix,
            &[
                from_pubkey.clone(),
                to_pubkey.clone(),
                system_program.clone(),
            ],
        )?,
    };
    Ok(())
}

pub fn token_transfer<'info>(
    token_program: AccountInfo<'info>,
    from_account: AccountInfo<'info>,
    to_account: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    amount: u64,
    signer: Option<&[&[&[u8]]]>,
) -> ProgramResult {
    let cpi_program = token_program;
    let cpi_accounts = Transfer {
        from: from_account,
        to: to_account,
        authority,
    };

    let cpi_ctx;
    match signer {
        Some(seeds) => {
            cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, seeds);
        }
        None => {
            cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        }
    }
    token::transfer(cpi_ctx, amount)?;

    Ok(())
}

pub fn mint_to<'a>(
    token_program: AccountInfo<'a>,
    mint: AccountInfo<'a>,
    to: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    amount: u64,
    signer: Option<&[&[&[u8]]]>,
) -> ProgramResult {
    let accs = MintTo {
        mint,
        to,
        authority,
    };
    let cpi_ctx;
    match signer {
        Some(seeds) => {
            cpi_ctx = CpiContext::new_with_signer(token_program, accs, seeds);
        }
        None => {
            cpi_ctx = CpiContext::new(token_program, accs);
        }
    }
    token::mint_to(cpi_ctx, amount)?;

    Ok(())
}
