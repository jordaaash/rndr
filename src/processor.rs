//! Program state processor

use {
    crate::{
        error::RNDRError,
        helpers::{
            assert_rent_exempt, assert_uninitialized, spl_token_init_account, spl_token_transfer,
            TokenInitAccountParams, TokenTransferParams,
        },
        instruction::RNDRInstruction,
        state::{Escrow, InitEscrowParams, InitJobParams, Job},
    },
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        program_pack::{IsInitialized, Pack},
        pubkey::Pubkey,
        sysvar::{rent::Rent, Sysvar},
    },
    spl_token::state::Mint,
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
    let escrow_info = next_account_info(account_info_iter)?;
    let token_account_info = next_account_info(account_info_iter)?;
    let token_mint_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let token_program_id = next_account_info(account_info_iter)?;

    let (escrow_pubkey, _bump_seed) = Pubkey::find_program_address(
        &[
            b"escrow",
            token_account_info.key.as_ref(),
            token_program_id.key.as_ref(),
        ],
        program_id,
    );
    if &escrow_pubkey != escrow_info.key {
        msg!("Escrow program derived address does not match the escrow address provided");
        return Err(RNDRError::UnspecifiedError.into());
    }

    let rent = &Rent::from_account_info(rent_info)?;
    assert_rent_exempt(rent, escrow_info)?;

    let mut escrow = assert_uninitialized::<Escrow>(escrow_info)?;
    if escrow_info.owner != program_id {
        msg!("Escrow provided is not owned by the RNDR program");
        return Err(RNDRError::UnspecifiedError.into());
    }

    Mint::unpack(&token_mint_info.try_borrow_data()?).map_err(|_| RNDRError::UnspecifiedError)?;
    if token_mint_info.owner != token_program_id.key {
        msg!("RNDR token mint is not owned by the token program provided");
        return Err(RNDRError::UnspecifiedError.into());
    }

    spl_token_init_account(TokenInitAccountParams {
        account: token_account_info.clone(),
        mint: token_mint_info.clone(),
        owner: escrow_info.clone(),
        rent: rent_info.clone(),
        token_program: token_program_id.clone(),
    })?;

    escrow.init(InitEscrowParams { owner });
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
    let escrow_info = next_account_info(account_info_iter)?;
    let escrow_owner_info = next_account_info(account_info_iter)?;

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
    let source_token_info = next_account_info(account_info_iter)?;
    let destination_token_info = next_account_info(account_info_iter)?;
    let escrow_info = next_account_info(account_info_iter)?;
    let job_info = next_account_info(account_info_iter)?;
    let authority_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let token_program_id = next_account_info(account_info_iter)?;

    let (escrow_pubkey, _bump_seed) = Pubkey::find_program_address(
        &[
            b"escrow",
            destination_token_info.key.as_ref(),
            token_program_id.key.as_ref(),
        ],
        program_id,
    );
    if &escrow_pubkey != escrow_info.key {
        msg!("Escrow program derived address does not match the escrow address provided");
        return Err(RNDRError::UnspecifiedError.into());
    }

    let (job_pubkey, _bump_seed) = Pubkey::find_program_address(
        &[
            b"job",
            escrow_info.key.as_ref(),
            authority_info.key.as_ref(),
        ],
        program_id,
    );
    if &job_pubkey != job_info.key {
        msg!("Job program derived address does not match the job address provided");
        return Err(RNDRError::UnspecifiedError.into());
    }

    let mut escrow = Escrow::unpack(&escrow_info.try_borrow_data()?)?;
    if escrow_info.owner != program_id {
        msg!("Escrow provided is not owned by the RNDR program");
        return Err(RNDRError::UnspecifiedError.into());
    }

    let mut job = Job::unpack_unchecked(&job_info.try_borrow_data()?)?;
    if !job.is_initialized() {
        let rent = &Rent::from_account_info(rent_info)?;
        assert_rent_exempt(rent, job_info)?;

        job.init(InitJobParams {
            authority: *authority_info.key,
        });
    }
    if job_info.owner != program_id {
        msg!("Job provided is not owned by the RNDR program");
        return Err(RNDRError::UnspecifiedError.into());
    }

    spl_token_transfer(TokenTransferParams {
        source: source_token_info.clone(),
        destination: destination_token_info.clone(),
        amount,
        authority: authority_info.clone(),
        authority_signer_seeds: &[],
        token_program: token_program_id.clone(),
    })?;

    job.amount = job
        .amount
        .checked_add(amount)
        .ok_or(RNDRError::UnspecifiedError)?;
    escrow.amount = escrow
        .amount
        .checked_add(amount)
        .ok_or(RNDRError::UnspecifiedError)?;

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
    let source_token_info = next_account_info(account_info_iter)?;
    let destination_token_info = next_account_info(account_info_iter)?;
    let escrow_info = next_account_info(account_info_iter)?;
    let job_info = next_account_info(account_info_iter)?;
    let escrow_owner_info = next_account_info(account_info_iter)?;
    let token_program_id = next_account_info(account_info_iter)?;

    let authority_signer_seeds: &[&[u8]] = &[
        b"escrow",
        source_token_info.key.as_ref(),
        token_program_id.key.as_ref(),
    ];

    let (escrow_pubkey, _bump_seed) =
        Pubkey::find_program_address(authority_signer_seeds, program_id);
    if &escrow_pubkey != escrow_info.key {
        msg!("Escrow program derived address does not match the escrow address provided");
        return Err(RNDRError::UnspecifiedError.into());
    }

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
        return Err(RNDRError::UnspecifiedError.into());
    }

    job.amount = job
        .amount
        .checked_sub(amount)
        .ok_or(RNDRError::UnspecifiedError)?;
    escrow.amount = escrow
        .amount
        .checked_sub(amount)
        .ok_or(RNDRError::UnspecifiedError)?;

    Job::pack(job, &mut job_info.try_borrow_mut_data()?)?;
    Escrow::pack(escrow, &mut escrow_info.try_borrow_mut_data()?)?;

    spl_token_transfer(TokenTransferParams {
        source: source_token_info.clone(),
        destination: destination_token_info.clone(),
        amount,
        authority: escrow_info.clone(),
        authority_signer_seeds,
        token_program: token_program_id.clone(),
    })?;

    Ok(())
}
