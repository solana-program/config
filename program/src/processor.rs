//! Config program processor.

use {
    crate::state::ConfigKeys,
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        program_error::ProgramError,
        pubkey::Pubkey,
    },
    std::collections::BTreeSet,
};

// [Core BPF]: Locally-implemented
// `solana_sdk::program_utils::limited_deserialize`.
fn limited_deserialize<T>(input: &[u8]) -> Result<T, ProgramError>
where
    T: serde::de::DeserializeOwned,
{
    solana_program::program_utils::limited_deserialize(
        input, 1232, // [Core BPF]: See `solana_sdk::packet::PACKET_DATA_SIZE`
    )
    .map_err(|_| ProgramError::InvalidInstructionData)
}

/// Config program processor.
pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
    let key_list: ConfigKeys = limited_deserialize(input)?;

    let mut accounts_iter = accounts.iter();
    let config_account = next_account_info(&mut accounts_iter)?;

    if config_account.owner != program_id {
        msg!("Config account is not owned by the config program");
        return Err(ProgramError::InvalidAccountOwner);
    }

    let current_data: ConfigKeys = bincode::deserialize(&config_account.try_borrow_data()?)
        .map_err(|err| {
            msg!("Unable to deserialize config account: {}", err);
            ProgramError::InvalidAccountData
        })?;

    let current_signer_keys: Vec<Pubkey> = current_data
        .keys
        .iter()
        .filter(|(_, is_signer)| *is_signer)
        .map(|(pubkey, _)| *pubkey)
        .collect();

    if current_signer_keys.is_empty() {
        // Config account keypair must be a signer on account initialization,
        // or when no signers specified in Config data.
        if !config_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
    }

    let mut counter: usize = 0;
    for (signer, _) in key_list.keys.iter().filter(|(_, is_signer)| *is_signer) {
        counter = counter.saturating_add(1);
        if signer != config_account.key {
            let signer_account = next_account_info(&mut accounts_iter).map_err(|_| {
                msg!("account {:?} is not in account list", signer);
                ProgramError::MissingRequiredSignature
            })?;
            if !signer_account.is_signer {
                msg!("account {:?} signer_key().is_none()", signer);
                return Err(ProgramError::MissingRequiredSignature);
            }
            if signer_account.key != signer {
                msg!(
                    "account[{:?}].signer_key() does not match Config data)",
                    counter.saturating_add(1)
                );
                return Err(ProgramError::MissingRequiredSignature);
            }
            // If Config account is already initialized, update signatures must match Config
            // data.
            if !current_data.keys.is_empty()
                && !current_signer_keys.iter().any(|pubkey| pubkey == signer)
            {
                msg!("account {:?} is not in stored signer list", signer);
                return Err(ProgramError::MissingRequiredSignature);
            }
        } else if !config_account.is_signer {
            msg!("account[0].signer_key().is_none()");
            return Err(ProgramError::MissingRequiredSignature);
        }
    }

    // Dedupe signers.
    let total_new_keys = key_list.keys.len();
    let unique_new_keys = key_list.keys.into_iter().collect::<BTreeSet<_>>();
    if unique_new_keys.len() != total_new_keys {
        msg!("new config contains duplicate keys");
        return Err(ProgramError::InvalidArgument);
    }

    // Check for Config data signers not present in incoming account update.
    if current_signer_keys.len() > counter {
        msg!(
            "too few signers: {:?}; expected: {:?}",
            counter,
            current_signer_keys.len()
        );
        return Err(ProgramError::MissingRequiredSignature);
    }

    if config_account.data_len() < input.len() {
        msg!("Instruction data too large");
        return Err(ProgramError::InvalidInstructionData);
    }

    config_account.try_borrow_mut_data()?[..input.len()].copy_from_slice(input);

    Ok(())
}
