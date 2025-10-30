//! Program error types.

use {
    num_derive::FromPrimitive,
    solana_program_error::{ProgramError, ToStr},
    thiserror::Error,
};

/// Errors that can be returned by the Config program.
#[repr(u32)]
#[derive(Error, Clone, Debug, Eq, PartialEq, FromPrimitive)]
pub enum ConfigError {
    /// Instruction modified data of a read-only account.
    #[error("Instruction modified data of a read-only account")]
    ReadonlyDataModified,
}

impl From<ConfigError> for ProgramError {
    fn from(e: ConfigError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl ToStr for ConfigError {
    fn to_str(&self) -> &'static str {
        match self {
            Self::ReadonlyDataModified => "Instruction modified data of a read-only account",
        }
    }
}

impl TryFrom<u32> for ConfigError {
    type Error = ProgramError;
    fn try_from(code: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(code).ok_or(ProgramError::InvalidArgument)
    }
}
