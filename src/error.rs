//! Error types

use {
    num_derive::FromPrimitive,
    solana_program::{decode_error::DecodeError, program_error::ProgramError},
    thiserror::Error,
};

/// Errors that may be returned by the program
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum RNDRError {
    // 0
    /// InstructionUnpackError
    #[error("InstructionUnpackError")]
    InstructionUnpackError,
    /// UnspecifiedError
    #[error("UnspecifiedError")]
    UnspecifiedError,
}

impl From<RNDRError> for ProgramError {
    fn from(e: RNDRError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for RNDRError {
    fn type_of() -> &'static str {
        "RNDR Error"
    }
}
