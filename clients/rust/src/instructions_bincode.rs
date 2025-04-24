//! Program instruction helpers.

use {
    crate::{ConfigKeys, ID},
    bincode::serialized_size,
    solana_program::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        system_instruction,
    },
};

/// Trait defining config state to be stored at the end of the account data.
#[deprecated(since = "1.0.0", note = "This trait is no longer supported")]
pub trait ConfigState: serde::Serialize + Default {
    /// Maximum space that the serialized representation will require
    fn max_space() -> u64;
}

fn initialize_account<T: Default + serde::Serialize>(config_pubkey: &Pubkey) -> Instruction {
    let account_metas = vec![AccountMeta::new(*config_pubkey, true)];
    let account_data = (ConfigKeys { keys: vec![] }, T::default());
    Instruction::new_with_bincode(ID, &account_data, account_metas)
}

/// Create a new, empty configuration account
pub fn create_account<T: Default + serde::Serialize>(
    from_account_pubkey: &Pubkey,
    config_account_pubkey: &Pubkey,
    lamports: u64,
    max_space: u64,
    keys: Vec<(Pubkey, bool)>,
) -> Vec<Instruction> {
    let space = max_space.saturating_add(serialized_size(&ConfigKeys { keys }).unwrap());
    vec![
        system_instruction::create_account(
            from_account_pubkey,
            config_account_pubkey,
            lamports,
            space,
            &ID,
        ),
        initialize_account::<T>(config_account_pubkey),
    ]
}

/// Store new data in a configuration account
pub fn store<T: serde::Serialize>(
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
    Instruction::new_with_bincode(ID, &account_data, account_metas)
}
