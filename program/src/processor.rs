//! Config program processor.

use {
    crate::state::ConfigKeys,
    solana_program::{
        account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
        pubkey::Pubkey,
    },
    std::collections::BTreeSet,
};

// Maximum input buffer length that can be deserialized.
// See `solana_sdk::packet::PACKET_DATA_SIZE`.
const MAX_INPUT_LEN: usize = 1232;
// Maximum vector length for a `ConfigKeys` struct's `keys` list.
// See comments below for `safe_deserialize_config_keys`.
//
// Take the maximum input length and subtract (up to) 3 bytes for the
// `ShortU16`, then divide that by the size of a `(Pubkey, bool)` entry.
const MAX_VECTOR_LEN: usize = (MAX_INPUT_LEN - 3) / (32 + 1);

// [Core BPF]: The original Config builtin leverages the
// `solana_sdk::program_utils::limited_deserialize` method to cap the length of
// the input buffer at `MAX_INPUT_LEN` (1232). As a result, any input buffer
// larger than `MAX_INPUT_LEN` will abort deserialization and return
// `InstructionError::InvalidInstructionData`.
//
// Howevever, since `ConfigKeys` contains a vector of `(Pubkey, bool)`, the
// `limited_deserialize` method will still read the vector's length and attempt
// to allocate a vector of the designated size. For extremely large length
// values, this can cause the initial allocation of a large vector to exhuast
// the BPF program's heap before deserialization can proceed.
//
// To mitigate this memory issue, the BPF version of the Config program has
// been designed to "peek" the length value, and ensure it cannot allocate a
// vector that would otherwise violate the input buffer length restriction.
// Since a `ShortU16` value for `MAX_VECTOR_LEN` fits in just one byte, we can
// simply peek the first byte before attempting deserialization.
fn safe_deserialize_config_keys(input: &[u8]) -> Result<ConfigKeys, ProgramError> {
    match input.first() {
        Some(first_byte) if *first_byte as usize <= MAX_VECTOR_LEN => {
            solana_program::program_utils::limited_deserialize::<ConfigKeys>(
                input,
                MAX_INPUT_LEN as u64,
            )
            .map_err(|_| ProgramError::InvalidInstructionData)
        }
        _ => Err(ProgramError::InvalidInstructionData),
    }
}

// [Core BPF]: Similar to `safe_deserialize_config_keys`, this helper serves to
// avoid over-allocations of memory, especially when the trailing data is
// invalid.
//
// Consider a case where an account is malformed, and the `ShortU16` vector
// length actually stores a value larger than the buffer itself. The original
// Config program builtin can handle the initial memory allocation, eventually
// returning `ProgramError::InvalidAccountData` when bincode throws an EOF
// error. However, the BPF version will panic on OOM before it can successfully
// return `ProgramError::InvalidAccountData`.
//
// This helper's purpose is solely to catch malformed `ConfigKeys` data before
// a memory allocation panic can occur, to ensure maximum backwards
// compatibility with the original builtin.
fn safe_deserialize_config_keys_from_state(input: &[u8]) -> Result<ConfigKeys, ProgramError> {
    let (vector_len, offset) = solana_program::short_vec::decode_shortu16_len(input)
        .map_err(|_| ProgramError::InvalidAccountData)?;
    if input[offset..].len() / (32 + 1) < vector_len {
        Err(ProgramError::InvalidAccountData)
    } else {
        bincode::deserialize(input).map_err(|err| {
            msg!("Unable to deserialize config account: {}", err);
            ProgramError::InvalidAccountData
        })
    }
}

/// Config program processor.
pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
    let key_list = safe_deserialize_config_keys(input)?;

    let config_account = accounts.first().ok_or(ProgramError::NotEnoughAccountKeys)?;

    if config_account.owner != program_id {
        msg!("Config account is not owned by the config program");
        return Err(ProgramError::InvalidAccountOwner);
    }

    let current_data = safe_deserialize_config_keys_from_state(&config_account.try_borrow_data()?)?;

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
            let signer_account = accounts.get(counter).ok_or_else(|| {
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
