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

/// Job state
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Job {
    /// Account type, must be JobV1 currently
    pub account_type: AccountType,
    /// Amount of tokens in escrow for the job
    pub amount: u64,
    /// User authority that initialized the job
    pub authority: Pubkey,
}

impl Job {
    /// Create a job
    pub fn new(params: InitJobParams) -> Self {
        let mut job = Self::default();
        Self::init(&mut job, params);
        job
    }

    /// Initialize a job
    pub fn init(&mut self, params: InitJobParams) {
        self.account_type = AccountType::JobV1;
        self.amount = 0;
        self.authority = params.authority;
    }
}

/// Initialize a job
pub struct InitJobParams {
    /// User authority that initialized the job
    pub authority: Pubkey,
}

impl Sealed for Job {}

impl IsInitialized for Job {
    fn is_initialized(&self) -> bool {
        self.account_type != AccountType::Uninitialized
    }
}

const JOB_LEN: usize = 41; // 1 + 8 + 32
impl Pack for Job {
    const LEN: usize = JOB_LEN;

    fn pack_into_slice(&self, output: &mut [u8]) {
        let output = array_mut_ref![output, 0, JOB_LEN];
        #[allow(clippy::ptr_offset_with_cast)]
        let (account_type, amount, authority) = mut_array_refs![output, 1, 8, PUBKEY_BYTES];

        *account_type = u8::from(self.account_type).to_le_bytes();
        *amount = self.amount.to_le_bytes();
        authority.copy_from_slice(&self.authority.to_bytes());
    }

    fn unpack_from_slice(input: &[u8]) -> Result<Self, ProgramError> {
        let input = array_ref![input, 0, JOB_LEN];
        #[allow(clippy::ptr_offset_with_cast)]
        let (account_type, amount, authority) = array_refs![input, 1, 8, PUBKEY_BYTES];

        let account_type = AccountType::try_from(u8::from_le_bytes(*account_type))
            .map_err(|_| ProgramError::InvalidAccountData)?;
        if account_type != AccountType::JobV1 {
            msg!("Job account type is invalid");
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            account_type,
            amount: u64::from_le_bytes(*amount),
            authority: Pubkey::new_from_array(*authority),
        })
    }
}
