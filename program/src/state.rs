//! Program state types.

use {
    bincode::{deserialize, serialized_size},
    bytemuck::{Pod, Zeroable},
    serde::{Deserialize, Serialize},
    solana_program::{program_error::ProgramError, pubkey::Pubkey, short_vec},
    spl_pod::primitives::PodBool,
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

/// Pod-type for zero-copy config key reads.
#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct PodConfigKey(pub Pubkey, pub PodBool);

/// Utility for obtaining stored config keys as a slice.
pub fn get_config_keys_slice(bytes: &[u8]) -> Result<&[PodConfigKey], ProgramError> {
    let (vector_len, num_bytes) = solana_program::short_vec::decode_shortu16_len(bytes)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    let offset = num_bytes;
    let length = vector_len
        .checked_mul(std::mem::size_of::<PodConfigKey>())
        .ok_or(ProgramError::InvalidAccountData)?;
    let end = offset.saturating_add(length);

    bytemuck::try_cast_slice::<u8, PodConfigKey>(&bytes[offset..end])
        .map_err(|_| ProgramError::InvalidAccountData)
}
