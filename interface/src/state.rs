use solana_pubkey::Pubkey;
#[cfg(feature = "serde")]
use {
    serde_derive::{Deserialize, Serialize},
    solana_short_vec as short_vec,
};
#[cfg(feature = "wincode")]
use {
    solana_short_vec::ShortU16,
    wincode::{containers, SchemaRead, SchemaWrite},
};

/// A collection of keys to be stored in Config account data.
#[derive(Debug, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "wincode", derive(SchemaRead, SchemaWrite))]
pub struct ConfigKeys {
    // Each key tuple comprises a unique `Pubkey` identifier,
    // and `bool` whether that key is a signer of the data
    #[cfg_attr(feature = "serde", serde(with = "short_vec"))]
    #[cfg_attr(
        feature = "wincode",
        wincode(with = "containers::Vec<(Pubkey, bool), ShortU16>")
    )]
    pub keys: Vec<(Pubkey, bool)>,
}

/// Utility for extracting the `ConfigKeys` data from the account data.
#[cfg(all(not(feature = "wincode"), feature = "bincode"))]
pub fn get_config_data(bytes: &[u8]) -> Result<&[u8], bincode::Error> {
    bincode::deserialize::<ConfigKeys>(bytes)
        .and_then(|keys| bincode::serialized_size(&keys))
        .map(|offset| &bytes[offset as usize..])
}

/// Utility for extracting the `ConfigKeys` data from the account data.
#[cfg(feature = "wincode")]
pub fn get_config_data(bytes: &[u8]) -> Result<&[u8], wincode::ReadError> {
    // Reading through a cursor yields the offset of the trailing config data
    // directly, so unlike the `bincode` implementation there is no need to
    // re-serialize the keys to work out where they end.
    let mut reader = wincode::io::Cursor::new(bytes);
    wincode::deserialize_from::<ConfigKeys>(&mut reader)?;
    Ok(&bytes[reader.position()..])
}

#[cfg(all(test, feature = "serde", feature = "wincode"))]
mod tests {
    use {super::*, wincode::Serialize as _};

    fn config_keys(num_keys: usize) -> ConfigKeys {
        ConfigKeys {
            keys: (0..num_keys)
                .map(|i| (Pubkey::new_unique(), i % 2 == 0))
                .collect(),
        }
    }

    #[test]
    fn test_wincode_matches_bincode() {
        // Cover one, two, and three byte `ShortU16` key list lengths.
        for num_keys in [0, 1, 2, 127, 128, 129, 16_383, 16_384] {
            let keys = config_keys(num_keys);

            let bincode_bytes = bincode::serialize(&keys).unwrap();
            let wincode_bytes = wincode::serialize(&keys).unwrap();
            assert_eq!(bincode_bytes, wincode_bytes);

            assert_eq!(
                bincode::serialized_size(&keys).unwrap(),
                ConfigKeys::serialized_size(&keys).unwrap()
            );

            let decoded: ConfigKeys = wincode::deserialize(&bincode_bytes).unwrap();
            assert_eq!(decoded.keys, keys.keys);
        }
    }

    #[test]
    fn test_get_config_data() {
        let keys = config_keys(3);
        let mut bytes = wincode::serialize(&keys).unwrap();
        let prefix_len = bytes.len();
        bytes.extend_from_slice(&[1, 2, 3, 4]);

        assert_eq!(get_config_data(&bytes).unwrap(), &[1, 2, 3, 4]);
        assert_eq!(get_config_data(&bytes[..prefix_len]).unwrap(), &[]);
    }

    #[test]
    fn test_get_config_data_invalid() {
        // Key list length claims one entry, but no entry follows.
        assert!(get_config_data(&[1]).is_err());
    }
}
