//! Program state types.

use {
    serde::{Deserialize, Serialize},
    solana_program::{pubkey::Pubkey, short_vec},
};

/// A collection of keys to be stored in Config account data.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ConfigKeys {
    /// Each key tuple comprises a unique `Pubkey` identifier,
    /// and `bool` whether that key is a signer of the data.
    #[serde(with = "short_vec")]
    pub keys: Vec<(Pubkey, bool)>,
}
