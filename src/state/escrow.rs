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

/// Escrow state
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Escrow {
    /// Version of escrow
    pub version: u8,
    /// Bump seed for derived authority address
    pub bump_seed: u8,
    /// Amount of tokens in escrow
    pub amount: u64,
    /// Owner authority which can disburse funds
    pub owner: Pubkey,
    /// RNDR SPL token account to hold tokens in escrow
    pub token_account: Pubkey,
    /// RNDR SPL token mint
    pub token_mint: Pubkey,
    /// Token program id
    pub token_program_id: Pubkey,
}

impl Escrow {
    /// Create a new escrow
    pub fn new(params: InitEscrowParams) -> Self {
        let mut escrow = Self::default();
        Self::init(&mut escrow, params);
        escrow
    }

    /// Initialize an escrow
    pub fn init(&mut self, params: InitEscrowParams) {
        self.version = PROGRAM_VERSION;
        self.bump_seed = params.bump_seed;
        self.amount = 0;
        self.owner = params.owner;
        self.token_account = params.token_account;
        self.token_mint = params.token_mint;
        self.token_program_id = params.token_program_id;
    }
}

/// Initialize a escrow
pub struct InitEscrowParams {
    /// Bump seed for derived authority address
    pub bump_seed: u8,
    /// Owner authority which can disburse funds
    pub owner: Pubkey,
    /// RNDR SPL token account to hold tokens in escrow
    pub token_account: Pubkey,
    /// RNDR SPL token mint
    pub token_mint: Pubkey,
    /// Token program id
    pub token_program_id: Pubkey,
}

impl Sealed for Escrow {}

impl IsInitialized for Escrow {
    fn is_initialized(&self) -> bool {
        self.version != UNINITIALIZED_VERSION
    }
}

const ESCROW_LEN: usize = 138;

// 1 + 1 + 8 + 32 + 32 + 32 + 32
impl Pack for Escrow {
    const LEN: usize = ESCROW_LEN;

    fn pack_into_slice(&self, output: &mut [u8]) {
        let output = array_mut_ref![output, 0, ESCROW_LEN];
        #[allow(clippy::ptr_offset_with_cast)]
        let (version, bump_seed, amount, owner, token_account, token_mint, token_program_id) = mut_array_refs![
            output,
            1,
            1,
            8,
            PUBKEY_BYTES,
            PUBKEY_BYTES,
            PUBKEY_BYTES,
            PUBKEY_BYTES
        ];

        *version = self.version.to_le_bytes();
        *bump_seed = self.bump_seed.to_le_bytes();
        *amount = self.amount.to_le_bytes();
        owner.copy_from_slice(self.owner.as_ref());
        token_account.copy_from_slice(self.token_account.as_ref());
        token_mint.copy_from_slice(self.token_mint.as_ref());
        token_program_id.copy_from_slice(self.token_program_id.as_ref());
    }

    fn unpack_from_slice(input: &[u8]) -> Result<Self, ProgramError> {
        let input = array_ref![input, 0, ESCROW_LEN];
        #[allow(clippy::ptr_offset_with_cast)]
        let (version, bump_seed, amount, owner, token_account, token_mint, token_program_id) = array_refs![
            input,
            1,
            1,
            8,
            PUBKEY_BYTES,
            PUBKEY_BYTES,
            PUBKEY_BYTES,
            PUBKEY_BYTES
        ];

        let version = u8::from_le_bytes(*version);
        if version > PROGRAM_VERSION {
            msg!("Escrow version does not match RNDR program version");
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            version,
            bump_seed: u8::from_le_bytes(*bump_seed),
            amount: u64::from_le_bytes(*amount),
            owner: Pubkey::new_from_array(*owner),
            token_account: Pubkey::new_from_array(*token_account),
            token_mint: Pubkey::new_from_array(*token_mint),
            token_program_id: Pubkey::new_from_array(*token_program_id),
        })
    }
}
