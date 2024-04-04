//! Program state types.

use {
    bincode::{deserialize, serialized_size},
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

impl ConfigKeys {
    /// Get the serialized size of the `ConfigKeys` struct,
    /// given a list of keys.
    pub fn serialized_size(keys: Vec<(Pubkey, bool)>) -> u64 {
        serialized_size(&ConfigKeys { keys }).unwrap()
    }
}

/// Utility for extracting the `ConfigKeys` data from the account data.
pub fn get_config_data(bytes: &[u8]) -> Result<&[u8], bincode::Error> {
    deserialize::<ConfigKeys>(bytes)
        .and_then(|keys| serialized_size(&keys))
        .map(|offset| &bytes[offset as usize..])
}
