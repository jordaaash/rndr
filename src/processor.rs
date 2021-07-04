//! Program state processor

use {
    crate::{
        error::RNDRError,
        instruction::RNDRInstruction,
        state::{Escrow, InitEscrowParams, InitJobParams, Job},
    },
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        instruction::{AccountMeta, Instruction},
        msg,
        program::{invoke, invoke_signed},
        program_error::ProgramError,
        program_pack::Pack,
        pubkey::Pubkey,
        system_instruction,
        sysvar::{rent::Rent, Sysvar},
    },
    spl_associated_token_account::get_associated_token_address,
};

/// Processes an instruction
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    let instruction = RNDRInstruction::unpack(input)?;
    match instruction {
        RNDRInstruction::InitEscrow { owner } => {
            msg!("Instruction: InitEscrow");
            process_init_escrow(program_id, owner, accounts)
        }
        RNDRInstruction::SetEscrowOwner { new_owner } => {
            msg!("Instruction: SetEscrowOwner");
            process_set_escrow_owner(program_id, new_owner, accounts)
        }
        RNDRInstruction::FundJob { amount } => {
            msg!("Instruction: FundJob");
            process_fund_job(program_id, amount, accounts)
        }
        RNDRInstruction::DisburseFunds { amount } => {
            msg!("Instruction: DisburseFunds");
            process_disburse_funds(program_id, amount, accounts)
        }
    }
}

#[inline(never)] // avoid stack frame limit
fn process_init_escrow(
    program_id: &Pubkey,
    owner: Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    // RNDR token mint
    let token_mint_info = next_account_info(account_info_iter)?;
    // Source accounts
    let funder_info = next_account_info(account_info_iter)?;
    // Destination accounts
    let escrow_info = next_account_info(account_info_iter)?;
    let escrow_associated_token_info = next_account_info(account_info_iter)?;
    // Sysvars
    let rent_info = next_account_info(account_info_iter)?;
    // Programs
    let system_program_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let associated_token_program_info = next_account_info(account_info_iter)?;

    let mut escrow_seeds: Vec<&[_]> = vec![
        b"escrow",
        token_mint_info.key.as_ref(),
        token_program_info.key.as_ref(),
    ];

    let (escrow_address, bump_seed) = Pubkey::find_program_address(&escrow_seeds, program_id);
    if &escrow_address != escrow_info.key {
        msg!("Escrow program derived address does not match the escrow address provided");
        return Err(ProgramError::InvalidSeeds);
    }

    let bump_seed = &[bump_seed];
    escrow_seeds.push(bump_seed);

    invoke(
        &Instruction {
            program_id: *associated_token_program_info.key,
            accounts: vec![
                AccountMeta::new(*funder_info.key, true),
                AccountMeta::new(*escrow_associated_token_info.key, false),
                AccountMeta::new_readonly(escrow_address, false),
                AccountMeta::new_readonly(*token_mint_info.key, false),
                AccountMeta::new_readonly(*system_program_info.key, false),
                AccountMeta::new_readonly(*token_program_info.key, false),
                AccountMeta::new_readonly(*rent_info.key, false),
            ],
            data: vec![],
        },
        &[
            funder_info.clone(),
            escrow_associated_token_info.clone(),
            escrow_info.clone(),
            token_mint_info.clone(),
            system_program_info.clone(),
            token_program_info.clone(),
            rent_info.clone(),
        ],
    )?;

    let rent = &Rent::from_account_info(rent_info)?;
    let required_lamports = rent
        .minimum_balance(Escrow::LEN)
        .max(1)
        .saturating_sub(escrow_info.lamports());
    if required_lamports > 0 {
        invoke(
            &system_instruction::transfer(funder_info.key, escrow_info.key, required_lamports),
            &[
                funder_info.clone(),
                escrow_info.clone(),
                system_program_info.clone(),
            ],
        )?;
    }

    invoke_signed(
        &system_instruction::allocate(escrow_info.key, Escrow::LEN as u64),
        &[escrow_info.clone(), system_program_info.clone()],
        &[&escrow_seeds],
    )?;

    invoke_signed(
        &system_instruction::assign(escrow_info.key, program_id),
        &[escrow_info.clone(), system_program_info.clone()],
        &[&escrow_seeds],
    )?;

    let escrow = Escrow::new(InitEscrowParams { owner });
    Escrow::pack(escrow, &mut escrow_info.try_borrow_mut_data()?)?;

    Ok(())
}

#[inline(never)] // avoid stack frame limit
fn process_set_escrow_owner(
    program_id: &Pubkey,
    new_owner: Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    // Accounts
    let escrow_info = next_account_info(account_info_iter)?;
    let current_owner_info = next_account_info(account_info_iter)?;

    let mut escrow = Escrow::unpack(&escrow_info.try_borrow_data()?)?;
    if escrow_info.owner != program_id {
        msg!("Escrow provided is not owned by the RNDR program");
        return Err(RNDRError::UnspecifiedError.into());
    }
    if &escrow.owner != current_owner_info.key {
        msg!("Escrow owner does not match the current owner provided");
        return Err(RNDRError::UnspecifiedError.into());
    }
    if !current_owner_info.is_signer {
        msg!("Current owner provided must be a signer");
        return Err(RNDRError::UnspecifiedError.into());
    }

    escrow.owner = new_owner;
    Escrow::pack(escrow, &mut escrow_info.try_borrow_mut_data()?)?;

    Ok(())
}

#[inline(never)] // avoid stack frame limit
fn process_fund_job(program_id: &Pubkey, amount: u64, accounts: &[AccountInfo]) -> ProgramResult {
    if amount == 0 {
        msg!("Amount of tokens to fund can't be zero");
        return Err(RNDRError::UnspecifiedError.into());
    }

    let account_info_iter = &mut accounts.iter();
    // RNDR token mint
    let token_mint_info = next_account_info(account_info_iter)?;
    // Source accounts
    let funder_info = next_account_info(account_info_iter)?;
    let source_token_info = next_account_info(account_info_iter)?;
    let authority_info = next_account_info(account_info_iter)?;
    // Destination accounts
    let escrow_info = next_account_info(account_info_iter)?;
    let escrow_associated_token_info = next_account_info(account_info_iter)?;
    let job_info = next_account_info(account_info_iter)?;
    // Sysvars
    let rent_info = next_account_info(account_info_iter)?;
    // Programs
    let system_program_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;

    let (escrow_address, _bump_seed) = Pubkey::find_program_address(
        &[
            b"escrow",
            token_mint_info.key.as_ref(),
            token_program_info.key.as_ref(),
        ],
        program_id,
    );
    if &escrow_address != escrow_info.key {
        msg!("Escrow program derived address does not match the escrow address provided");
        return Err(ProgramError::InvalidSeeds);
    }

    let mut escrow = Escrow::unpack(&escrow_info.try_borrow_data()?)?;
    if escrow_info.owner != program_id {
        msg!("Escrow provided is not owned by the RNDR program");
        return Err(RNDRError::UnspecifiedError.into());
    }

    let escrow_associated_token_address =
        get_associated_token_address(&escrow_address, token_mint_info.key);
    if &escrow_associated_token_address != escrow_associated_token_info.key {
        msg!(
            "Escrow associated token address does not match the associated token address provided"
        );
        return Err(ProgramError::InvalidSeeds);
    }

    let mut job_seeds: Vec<&[_]> = vec![
        b"job",
        escrow_info.key.as_ref(),
        authority_info.key.as_ref(),
    ];

    let (job_pubkey, bump_seed) = Pubkey::find_program_address(&job_seeds, program_id);
    if &job_pubkey != job_info.key {
        msg!("Job program derived address does not match the job address provided");
        return Err(ProgramError::InvalidSeeds);
    }

    let mut job = if job_info.try_data_is_empty()? {
        let bump_seed = &[bump_seed];
        job_seeds.push(bump_seed);

        let rent = &Rent::from_account_info(rent_info)?;
        let required_lamports = rent
            .minimum_balance(Job::LEN)
            .max(1)
            .saturating_sub(job_info.lamports());
        if required_lamports > 0 {
            invoke(
                &system_instruction::transfer(funder_info.key, job_info.key, required_lamports),
                &[
                    funder_info.clone(),
                    job_info.clone(),
                    system_program_info.clone(),
                ],
            )?;
        }

        invoke_signed(
            &system_instruction::allocate(job_info.key, Job::LEN as u64),
            &[job_info.clone(), system_program_info.clone()],
            &[&job_seeds],
        )?;

        invoke_signed(
            &system_instruction::assign(job_info.key, program_id),
            &[job_info.clone(), system_program_info.clone()],
            &[&job_seeds],
        )?;

        Job::new(InitJobParams {
            authority: *authority_info.key,
        })
    } else if job_info.owner != program_id {
        msg!("Job provided is not owned by the RNDR program");
        return Err(RNDRError::UnspecifiedError.into());
    } else {
        Job::unpack(&job_info.try_borrow_data()?)?
    };

    invoke(
        &spl_token::instruction::transfer(
            token_program_info.key,
            source_token_info.key,
            escrow_associated_token_info.key,
            authority_info.key,
            &[],
            amount,
        )?,
        &[
            source_token_info.clone(),
            escrow_associated_token_info.clone(),
            authority_info.clone(),
            token_program_info.clone(),
        ],
    )?;

    job.amount = job.amount.checked_add(amount).ok_or(RNDRError::MathError)?;
    escrow.amount = escrow
        .amount
        .checked_add(amount)
        .ok_or(RNDRError::MathError)?;

    Job::pack(job, &mut job_info.try_borrow_mut_data()?)?;
    Escrow::pack(escrow, &mut escrow_info.try_borrow_mut_data()?)?;

    Ok(())
}

#[inline(never)] // avoid stack frame limit
fn process_disburse_funds(
    program_id: &Pubkey,
    amount: u64,
    accounts: &[AccountInfo],
) -> ProgramResult {
    if amount == 0 {
        msg!("Amount of tokens to dispurse can't be zero");
        return Err(RNDRError::UnspecifiedError.into());
    }

    let account_info_iter = &mut accounts.iter();
    // RNDR token mint
    let token_mint_info = next_account_info(account_info_iter)?;
    // Source accounts
    let escrow_info = next_account_info(account_info_iter)?;
    let escrow_owner_info = next_account_info(account_info_iter)?;
    let escrow_associated_token_info = next_account_info(account_info_iter)?;
    let job_info = next_account_info(account_info_iter)?;
    // Destination accounts
    let destination_token_info = next_account_info(account_info_iter)?;
    // Programs
    let token_program_info = next_account_info(account_info_iter)?;

    let mut escrow_seeds: Vec<&[_]> = vec![
        b"escrow",
        token_mint_info.key.as_ref(),
        token_program_info.key.as_ref(),
    ];

    let (escrow_address, bump_seed) = Pubkey::find_program_address(&escrow_seeds, program_id);
    if &escrow_address != escrow_info.key {
        msg!("Escrow program derived address does not match the escrow address provided");
        return Err(ProgramError::InvalidSeeds);
    }

    let bump_seed = &[bump_seed];
    escrow_seeds.push(bump_seed);

    let mut escrow = Escrow::unpack(&escrow_info.try_borrow_data()?)?;
    if escrow_info.owner != program_id {
        msg!("Escrow provided is not owned by the RNDR program");
        return Err(RNDRError::UnspecifiedError.into());
    }
    if &escrow.owner != escrow_owner_info.key {
        msg!("Escrow owner does not match the escrow owner provided");
        return Err(RNDRError::UnspecifiedError.into());
    }
    if !escrow_owner_info.is_signer {
        msg!("Escrow owner provided must be a signer");
        return Err(RNDRError::UnspecifiedError.into());
    }

    let escrow_associated_token_address =
        get_associated_token_address(&escrow_address, token_mint_info.key);
    if &escrow_associated_token_address != escrow_associated_token_info.key {
        msg!(
            "Escrow associated token address does not match the associated token address provided"
        );
        return Err(ProgramError::InvalidSeeds);
    }

    let mut job = Job::unpack(&job_info.try_borrow_data()?)?;
    if job_info.owner != program_id {
        msg!("Job provided is not owned by the RNDR program");
        return Err(RNDRError::UnspecifiedError.into());
    }

    let (job_pubkey, _bump_seed) = Pubkey::find_program_address(
        &[b"job", escrow_info.key.as_ref(), job.authority.as_ref()],
        program_id,
    );
    if &job_pubkey != job_info.key {
        msg!("Job program derived address does not match the job address provided");
        return Err(ProgramError::InvalidSeeds);
    }

    job.amount = job.amount.checked_sub(amount).ok_or(RNDRError::MathError)?;
    escrow.amount = escrow
        .amount
        .checked_sub(amount)
        .ok_or(RNDRError::MathError)?;

    Job::pack(job, &mut job_info.try_borrow_mut_data()?)?;
    Escrow::pack(escrow, &mut escrow_info.try_borrow_mut_data()?)?;

    invoke_signed(
        &spl_token::instruction::transfer(
            token_program_info.key,
            escrow_associated_token_info.key,
            destination_token_info.key,
            escrow_info.key,
            &[],
            amount,
        )?,
        &[
            escrow_associated_token_info.clone(),
            destination_token_info.clone(),
            escrow_info.clone(),
            token_program_info.clone(),
        ],
        &[&escrow_seeds],
    )?;

    Ok(())
}
