//! Program state types.

use {
    serde::{Deserialize, Serialize},
    solana_program::{pubkey::Pubkey, short_vec},
};

/// Trait defining config state to be stored at the end of the account data.
pub trait ConfigState: serde::Serialize + Default {
    /// Maximum space that the serialized representation will require
    fn max_space() -> u64;
}

/// A collection of keys to be stored in Config account data.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ConfigKeys {
    /// Each key tuple comprises a unique `Pubkey` identifier,
    /// and `bool` whether that key is a signer of the data.
    #[serde(with = "short_vec")]
    pub keys: Vec<(Pubkey, bool)>,
}
