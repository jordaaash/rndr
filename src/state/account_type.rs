use num_enum::{FromPrimitive, IntoPrimitive};

/// Enum representing the account types managed by the program
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive, FromPrimitive)]
#[repr(u8)]
pub enum AccountType {
    /// If the account has not been initialized, the value will be 0
    #[num_enum(default)]
    Uninitialized,
    /// Escrow
    EscrowV1,
    /// Job
    JobV1,
}

impl Default for AccountType {
    fn default() -> Self {
        AccountType::Uninitialized
    }
}
