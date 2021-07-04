use {
    super::*,
    arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs},
    solana_program::{
        msg,
        program_error::ProgramError,
        program_pack::{IsInitialized, Pack, Sealed},
        pubkey::{Pubkey, PUBKEY_BYTES},
    },
    std::convert::TryFrom,
};

/// Escrow state
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Escrow {
    /// Account type, must be EscrowV1 currently
    pub account_type: AccountType,
    /// Amount of tokens in escrow
    pub amount: u64,
    /// Owner authority that can disburse funds
    pub owner: Pubkey,
}

impl Escrow {
    /// Create an escrow
    pub fn new(params: InitEscrowParams) -> Self {
        let mut escrow = Self::default();
        Self::init(&mut escrow, params);
        escrow
    }

    /// Initialize an escrow
    pub fn init(&mut self, params: InitEscrowParams) {
        self.account_type = AccountType::EscrowV1;
        self.amount = 0;
        self.owner = params.owner;
    }
}

/// Initialize a escrow
pub struct InitEscrowParams {
    /// Owner authority that can disburse funds
    pub owner: Pubkey,
}

impl Sealed for Escrow {}

impl IsInitialized for Escrow {
    fn is_initialized(&self) -> bool {
        self.account_type != AccountType::Uninitialized
    }
}

const ESCROW_LEN: usize = 41; // 1 + 8 + 32
impl Pack for Escrow {
    const LEN: usize = ESCROW_LEN;

    fn pack_into_slice(&self, output: &mut [u8]) {
        let output = array_mut_ref![output, 0, ESCROW_LEN];
        #[allow(clippy::ptr_offset_with_cast)]
        let (account_type, amount, owner) = mut_array_refs![output, 1, 8, PUBKEY_BYTES];

        *account_type = u8::from(self.account_type).to_le_bytes();
        *amount = self.amount.to_le_bytes();
        owner.copy_from_slice(&self.owner.to_bytes());
    }

    fn unpack_from_slice(input: &[u8]) -> Result<Self, ProgramError> {
        let input = array_ref![input, 0, ESCROW_LEN];
        #[allow(clippy::ptr_offset_with_cast)]
        let (account_type, amount, owner) = array_refs![input, 1, 8, PUBKEY_BYTES];

        let account_type = AccountType::try_from(u8::from_le_bytes(*account_type))
            .map_err(|_| ProgramError::InvalidAccountData)?;
        if account_type != AccountType::EscrowV1 {
            msg!("Escrow account type is invalid");
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            account_type,
            amount: u64::from_le_bytes(*amount),
            owner: Pubkey::new_from_array(*owner),
        })
    }
}
