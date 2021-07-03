//! Program state processor

use {
    crate::{
        error::RNDRError,
        instruction::RNDRInstruction,
        state::{Escrow, InitEscrowParams, InitJobParams, Job},
    },
    num_traits::FromPrimitive,
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        decode_error::DecodeError,
        entrypoint::ProgramResult,
        instruction::Instruction,
        msg,
        program::{invoke, invoke_signed},
        program_error::{PrintProgramError, ProgramError},
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
            process_disburse(program_id, amount, accounts)
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

    let rent = &Rent::from_account_info(rent_info)?;
    assert_rent_exempt(rent, escrow_info)?;

    let mut escrow = assert_uninitialized::<Escrow>(escrow_info)?;
    if escrow_info.owner != program_id {
        msg!("Escrow provided is not owned by the RNDR program");
        return Err(RNDRError::UnspecifiedError.into());
    }

    let (escrow_pubkey, bump_seed) = Pubkey::find_program_address(&[b"escrow"], program_id);
    if &escrow_pubkey != escrow_info.key {
        msg!("Derived escrow address does not match the escrow address provided");
        return Err(RNDRError::UnspecifiedError.into());
    }

    Mint::unpack(&token_mint_info.try_borrow_data()?).map_err(|_| RNDRError::UnspecifiedError)?;
    if token_mint_info.owner != token_program_id.key {
        msg!("RNDR token mint is not owned by the token program provided");
        return Err(RNDRError::UnspecifiedError.into());
    }

    spl_token_init_account(TokenInitializeAccountParams {
        account: token_account_info.clone(),
        mint: token_mint_info.clone(),
        owner: escrow_info.clone(),
        rent: rent_info.clone(),
        token_program: token_program_id.clone(),
    })?;

    escrow.init(InitEscrowParams {
        bump_seed,
        owner,
        token_account: *token_account_info.key,
        token_mint: *token_mint_info.key,
        token_program_id: *token_program_id.key,
    });
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

    let mut escrow = Escrow::unpack(&escrow_info.try_borrow_data()?)?;
    if escrow_info.owner != program_id {
        msg!("Escrow provided is not owned by the RNDR program");
        return Err(RNDRError::UnspecifiedError.into());
    }
    if &escrow.token_account != destination_token_info.key {
        msg!("Escrow token account does not match the destination token provided");
        return Err(RNDRError::UnspecifiedError.into());
    }
    if &escrow.token_program_id != token_program_id.key {
        msg!("Escrow token program does not match the token program provided");
        return Err(RNDRError::UnspecifiedError.into());
    }

    // @TODO: should this bump seed be set on the job on init?
    let (job_pubkey, _bump_seed) =
        Pubkey::find_program_address(&[b"job", authority_info.key.as_ref()], program_id);
    if &job_pubkey != job_info.key {
        msg!("Derived job address does not match the job address provided");
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

    spl_token_transfer(TokenTransferParams {
        source: source_token_info.clone(),
        destination: destination_token_info.clone(),
        amount,
        authority: authority_info.clone(),
        authority_signer_seeds: &[],
        token_program: token_program_id.clone(),
    })?;

    Ok(())
}

#[inline(never)] // avoid stack frame limit
fn process_disburse(program_id: &Pubkey, amount: u64, accounts: &[AccountInfo]) -> ProgramResult {
    if amount == 0 {
        msg!("Amount of tokens to dispurse can't be zero");
        return Err(RNDRError::UnspecifiedError.into());
    }

    let account_info_iter = &mut accounts.iter();
    let source_token_info = next_account_info(account_info_iter)?;
    let destination_token_info = next_account_info(account_info_iter)?;
    let escrow_info = next_account_info(account_info_iter)?;
    let escrow_owner_info = next_account_info(account_info_iter)?;
    let token_program_id = next_account_info(account_info_iter)?;

    let mut escrow = Escrow::unpack(&escrow_info.try_borrow_data()?)?;
    if escrow_info.owner != program_id {
        msg!("Escrow provided is not owned by the RNDR program");
        return Err(RNDRError::UnspecifiedError.into());
    }
    if &escrow.token_account != source_token_info.key {
        msg!("Escrow token account does not match the source token account provided");
        return Err(RNDRError::UnspecifiedError.into());
    }
    if &escrow.token_program_id != token_program_id.key {
        msg!("Escrow token program does not match the token program provided");
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

    let authority_signer_seeds: &[&[u8]] = &[b"escrow", &[escrow.bump_seed]];

    escrow.amount = escrow
        .amount
        .checked_sub(amount)
        .ok_or(RNDRError::UnspecifiedError)?;

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

fn assert_rent_exempt(rent: &Rent, account_info: &AccountInfo) -> ProgramResult {
    if !rent.is_exempt(account_info.lamports(), account_info.data_len()) {
        msg!(&rent.minimum_balance(account_info.data_len()).to_string());
        Err(RNDRError::UnspecifiedError.into())
    } else {
        Ok(())
    }
}

fn assert_uninitialized<T: Pack + IsInitialized>(
    account_info: &AccountInfo,
) -> Result<T, ProgramError> {
    let account: T = T::unpack_unchecked(&account_info.try_borrow_data()?)?;
    if account.is_initialized() {
        Err(RNDRError::UnspecifiedError.into())
    } else {
        Ok(account)
    }
}

/// Issue a spl_token `InitializeAccount` instruction.
#[inline(always)]
fn spl_token_init_account(params: TokenInitializeAccountParams<'_>) -> ProgramResult {
    let TokenInitializeAccountParams {
        account,
        mint,
        owner,
        rent,
        token_program,
    } = params;
    let ix = spl_token::instruction::initialize_account(
        token_program.key,
        account.key,
        mint.key,
        owner.key,
    )?;
    let result = invoke(&ix, &[account, mint, owner, rent, token_program]);
    result.map_err(|_| RNDRError::UnspecifiedError.into())
}

/// Invoke signed unless signers seeds are empty
#[inline(always)]
fn invoke_optionally_signed(
    instruction: &Instruction,
    account_infos: &[AccountInfo],
    authority_signer_seeds: &[&[u8]],
) -> ProgramResult {
    if authority_signer_seeds.is_empty() {
        invoke(instruction, account_infos)
    } else {
        invoke_signed(instruction, account_infos, &[authority_signer_seeds])
    }
}

/// Issue a spl_token `Transfer` instruction.
#[inline(always)]
fn spl_token_transfer(params: TokenTransferParams<'_, '_>) -> ProgramResult {
    let TokenTransferParams {
        source,
        destination,
        authority,
        token_program,
        amount,
        authority_signer_seeds,
    } = params;
    let result = invoke_optionally_signed(
        &spl_token::instruction::transfer(
            token_program.key,
            source.key,
            destination.key,
            authority.key,
            &[],
            amount,
        )?,
        &[source, destination, authority, token_program],
        authority_signer_seeds,
    );
    result.map_err(|_| RNDRError::UnspecifiedError.into())
}

struct TokenInitializeAccountParams<'a> {
    account: AccountInfo<'a>,
    mint: AccountInfo<'a>,
    owner: AccountInfo<'a>,
    rent: AccountInfo<'a>,
    token_program: AccountInfo<'a>,
}

struct TokenTransferParams<'a: 'b, 'b> {
    source: AccountInfo<'a>,
    destination: AccountInfo<'a>,
    amount: u64,
    authority: AccountInfo<'a>,
    authority_signer_seeds: &'b [&'b [u8]],
    token_program: AccountInfo<'a>,
}

impl PrintProgramError for RNDRError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        msg!(&self.to_string());
    }
}
