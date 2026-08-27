//! `wincode` implementation of the instruction helpers.

use {
    crate::{id, state::ConfigKeys},
    ::wincode::{serialized_size, Serialize},
    solana_instruction::{AccountMeta, Instruction},
    solana_pubkey::Pubkey,
};

fn initialize_account<T: Default + Serialize<Src = T>>(config_pubkey: &Pubkey) -> Instruction {
    let account_metas = vec![AccountMeta::new(*config_pubkey, true)];
    let account_data = (ConfigKeys { keys: vec![] }, T::default());
    Instruction::new_with_wincode(id(), &account_data, account_metas)
}

/// Create a new, empty configuration account
pub fn create_account_with_max_config_space<T: Default + Serialize<Src = T>>(
    from_account_pubkey: &Pubkey,
    config_account_pubkey: &Pubkey,
    lamports: u64,
    max_config_space: u64,
    keys: Vec<(Pubkey, bool)>,
) -> Vec<Instruction> {
    let space = max_config_space.saturating_add(serialized_size(&ConfigKeys { keys }).unwrap());
    vec![
        solana_system_interface::instruction::create_account(
            from_account_pubkey,
            config_account_pubkey,
            lamports,
            space,
            &id(),
        ),
        initialize_account::<T>(config_account_pubkey),
    ]
}

/// Store new data in a configuration account
pub fn store<T: Serialize<Src = T>>(
    config_account_pubkey: &Pubkey,
    is_config_signer: bool,
    keys: Vec<(Pubkey, bool)>,
    data: &T,
) -> Instruction {
    let mut account_metas = vec![AccountMeta::new(*config_account_pubkey, is_config_signer)];
    for (signer_pubkey, _) in keys.iter().filter(|(_, is_signer)| *is_signer) {
        if signer_pubkey != config_account_pubkey {
            account_metas.push(AccountMeta::new(*signer_pubkey, true));
        }
    }
    let account_data = (ConfigKeys { keys }, data);
    Instruction::new_with_wincode(id(), &account_data, account_metas)
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use {
        super::*,
        ::wincode::{SchemaRead, SchemaWrite},
        serde_derive::Serialize as SerdeSerialize,
    };

    #[derive(Default, SerdeSerialize, SchemaRead, SchemaWrite)]
    struct MyConfig {
        value: u64,
    }

    #[test]
    fn test_store_data_matches_bincode() {
        let config_pubkey = Pubkey::new_unique();
        let keys = vec![(Pubkey::new_unique(), true), (Pubkey::new_unique(), false)];
        let my_config = MyConfig { value: 42 };

        let instruction = store(&config_pubkey, true, keys.clone(), &my_config);

        let expected = bincode::serialize(&(ConfigKeys { keys }, &my_config)).unwrap();
        assert_eq!(instruction.data, expected);
    }

    #[test]
    fn test_initialize_account_data_matches_bincode() {
        let instruction = initialize_account::<MyConfig>(&Pubkey::new_unique());

        let expected =
            bincode::serialize(&(ConfigKeys { keys: vec![] }, MyConfig::default())).unwrap();
        assert_eq!(instruction.data, expected);
    }

    #[test]
    fn test_create_account_space_matches_bincode() {
        let keys = vec![(Pubkey::new_unique(), true); 3];
        let instructions = create_account_with_max_config_space::<MyConfig>(
            &Pubkey::new_unique(),
            &Pubkey::new_unique(),
            1,
            100,
            keys.clone(),
        );

        let expected_space = 100 + bincode::serialized_size(&ConfigKeys { keys }).unwrap();
        // `create_account` encodes the space as a `u64` right after the 4-byte
        // instruction discriminant and the 8-byte lamports value.
        let space = u64::from_le_bytes(instructions[0].data[12..20].try_into().unwrap());
        assert_eq!(space, expected_space);
    }
}
