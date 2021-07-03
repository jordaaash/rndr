use {
    super::*,
    arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs},
    solana_program::{
        msg,
        program_error::ProgramError,
        program_pack::{IsInitialized, Pack, Sealed},
        pubkey::{Pubkey, PUBKEY_BYTES},
    },
};

/// Job state
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Job {
    /// Version of job
    pub version: u8,
    /// Amount of tokens in escrow for the job
    pub amount: u64,
    /// Authority that initialized the job
    pub authority: Pubkey,
}

impl Job {
    /// Create a new job
    pub fn new(params: InitJobParams) -> Self {
        let mut job = Self::default();
        Self::init(&mut job, params);
        job
    }

    /// Initialize an escrow
    pub fn init(&mut self, params: InitJobParams) {
        self.version = PROGRAM_VERSION;
        self.amount = 0;
        self.authority = params.authority;
    }
}

/// Initialize a job
pub struct InitJobParams {
    /// Authority that initialized the job
    pub authority: Pubkey,
}

impl Sealed for Job {}

impl IsInitialized for Job {
    fn is_initialized(&self) -> bool {
        self.version != UNINITIALIZED_VERSION
    }
}

const ESCROW_LEN: usize = 41;

// 1 + 8 + 32
impl Pack for Job {
    const LEN: usize = ESCROW_LEN;

    fn pack_into_slice(&self, output: &mut [u8]) {
        let output = array_mut_ref![output, 0, ESCROW_LEN];
        #[allow(clippy::ptr_offset_with_cast)]
        let (version, amount, authority) = mut_array_refs![output, 1, 8, PUBKEY_BYTES];

        *version = self.version.to_le_bytes();
        *amount = self.amount.to_le_bytes();
        authority.copy_from_slice(self.authority.as_ref());
    }

    fn unpack_from_slice(input: &[u8]) -> Result<Self, ProgramError> {
        let input = array_ref![input, 0, ESCROW_LEN];
        #[allow(clippy::ptr_offset_with_cast)]
        let (version, amount, authority) = array_refs![input, 1, 8, PUBKEY_BYTES];

        let version = u8::from_le_bytes(*version);
        if version > PROGRAM_VERSION {
            msg!("Job version does not match RNDR program version");
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            version,
            amount: u64::from_le_bytes(*amount),
            authority: Pubkey::new_from_array(*authority),
        })
    }
}
