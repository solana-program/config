//! Program error types.

use {
    num_derive::FromPrimitive,
    solana_program::{
        decode_error::DecodeError,
        msg,
        program_error::{PrintProgramError, ProgramError},
    },
    thiserror::Error,
};

/// Errors that can be returned by the Config program.
#[derive(Error, Clone, Debug, Eq, PartialEq, FromPrimitive)]
pub enum ConfigError {
    /// Instruction modified data of a read-only account.
    #[error("Instruction modified data of a read-only account")]
    ReadonlyDataModified,
}

impl PrintProgramError for ConfigError {
    fn print<E>(&self) {
        msg!(&self.to_string());
    }
}

impl From<ConfigError> for ProgramError {
    fn from(e: ConfigError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for ConfigError {
    fn type_of() -> &'static str {
        "ConfigError"
    }
}
