//! Program instruction helpers.

pub use solana_config_interface::instruction::{create_account_with_max_config_space, store};
use solana_program::{instruction::Instruction, pubkey::Pubkey};

/// Trait defining config state to be stored at the end of the account data.
#[deprecated(since = "1.0.0", note = "This trait is no longer supported")]
pub trait ConfigState: serde::Serialize + Default {
    /// Maximum space that the serialized representation will require
    fn max_space() -> u64;
}

/// Create a new, empty configuration account
#[deprecated(
    since = "1.0.0",
    note = "The `ConfigState` trait is no longer supported"
)]
#[allow(deprecated)]
pub fn create_account<T: ConfigState>(
    from_account_pubkey: &Pubkey,
    config_account_pubkey: &Pubkey,
    lamports: u64,
    keys: Vec<(Pubkey, bool)>,
) -> Vec<Instruction> {
    create_account_with_max_config_space::<T>(
        from_account_pubkey,
        config_account_pubkey,
        lamports,
        T::max_space(),
        keys,
    )
}
