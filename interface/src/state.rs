use solana_pubkey::Pubkey;
#[cfg(feature = "serde")]
use {
    serde_derive::{Deserialize, Serialize},
    solana_short_vec as short_vec,
};

/// A collection of keys to be stored in Config account data.
#[derive(Debug, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct ConfigKeys {
    // Each key tuple comprises a unique `Pubkey` identifier,
    // and `bool` whether that key is a signer of the data
    #[cfg_attr(feature = "serde", serde(with = "short_vec"))]
    pub keys: Vec<(Pubkey, bool)>,
}

/// Utility for extracting the `ConfigKeys` data from the account data.
#[cfg(feature = "bincode")]
pub fn get_config_data(bytes: &[u8]) -> Result<&[u8], bincode::Error> {
    bincode::deserialize::<ConfigKeys>(bytes)
        .and_then(|keys| bincode::serialized_size(&keys))
        .map(|offset| &bytes[offset as usize..])
}
