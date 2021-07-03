use {
    crate::error::RNDRError,
    solana_program::{
        account_info::AccountInfo,
        entrypoint::ProgramResult,
        instruction::Instruction,
        msg,
        program::{invoke, invoke_signed},
        program_error::ProgramError,
        program_pack::{IsInitialized, Pack},
        sysvar::rent::Rent,
    },
    spl_token::instruction::{initialize_account, transfer},
};

#[inline(always)]
pub fn assert_rent_exempt(rent: &Rent, account_info: &AccountInfo) -> ProgramResult {
    if !rent.is_exempt(account_info.lamports(), account_info.data_len()) {
        msg!(&rent.minimum_balance(account_info.data_len()).to_string());
        Err(RNDRError::UnspecifiedError.into())
    } else {
        Ok(())
    }
}

#[inline(always)]
pub fn assert_uninitialized<T: Pack + IsInitialized>(
    account_info: &AccountInfo,
) -> Result<T, ProgramError> {
    let account: T = T::unpack_unchecked(&account_info.try_borrow_data()?)?;
    if account.is_initialized() {
        Err(RNDRError::UnspecifiedError.into())
    } else {
        Ok(account)
    }
}

pub struct TokenInitAccountParams<'a> {
    pub account: AccountInfo<'a>,
    pub mint: AccountInfo<'a>,
    pub owner: AccountInfo<'a>,
    pub rent: AccountInfo<'a>,
    pub token_program: AccountInfo<'a>,
}

/// Issue a spl_token `InitializeAccount` instruction.
#[inline(always)]
pub fn spl_token_init_account(params: TokenInitAccountParams<'_>) -> ProgramResult {
    let TokenInitAccountParams {
        account,
        mint,
        owner,
        rent,
        token_program,
    } = params;
    let ix = initialize_account(token_program.key, account.key, mint.key, owner.key)?;
    let result = invoke(&ix, &[account, mint, owner, rent, token_program]);
    result.map_err(|_| RNDRError::UnspecifiedError.into())
}

pub struct TokenTransferParams<'a: 'b, 'b> {
    pub source: AccountInfo<'a>,
    pub destination: AccountInfo<'a>,
    pub amount: u64,
    pub authority: AccountInfo<'a>,
    pub authority_signer_seeds: &'b [&'b [u8]],
    pub token_program: AccountInfo<'a>,
}

/// Issue a spl_token `Transfer` instruction.
#[inline(always)]
pub fn spl_token_transfer(params: TokenTransferParams<'_, '_>) -> ProgramResult {
    let TokenTransferParams {
        source,
        destination,
        authority,
        token_program,
        amount,
        authority_signer_seeds,
    } = params;
    let result = invoke_optionally_signed(
        &transfer(
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
