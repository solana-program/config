#![cfg(feature = "test-sbf")]
#![allow(clippy::arithmetic_side_effects)]

use {
    bincode::serialized_size,
    mollusk_svm::{result::Check, Mollusk},
    serde::{Deserialize, Serialize},
    solana_config_program::{error::ConfigError, state::ConfigKeys},
    solana_config_program_client::instructions_bincode::{self as config_instruction},
    solana_sdk::{
        account::Account,
        instruction::{AccountMeta, Instruction},
        program_error::ProgramError,
        pubkey::Pubkey,
    },
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct MyConfig {
    pub item: u64,
}
impl Default for MyConfig {
    fn default() -> Self {
        Self { item: 123_456_789 }
    }
}
impl MyConfig {
    pub fn new(item: u64) -> Self {
        Self { item }
    }
}

fn setup() -> Mollusk {
    Mollusk::new(&solana_config_program::id(), "solana_config_program")
}

fn get_config_space(key_len: usize) -> usize {
    let entry_size = bincode::serialized_size(&(Pubkey::default(), true)).unwrap() as usize;
    let total_keys_size = (key_len).checked_mul(entry_size).unwrap();
    let serialized_size =
        bincode::serialized_size(&(ConfigKeys::default(), MyConfig::default())).unwrap() as usize;
    serialized_size.checked_add(total_keys_size).unwrap()
}

fn create_config_account(mollusk: &Mollusk, keys: Vec<(Pubkey, bool)>) -> Account {
    let space = get_config_space(keys.len());
    let lamports = mollusk.sysvars.rent.minimum_balance(space);
    Account::new_data(
        lamports,
        &(ConfigKeys { keys }, MyConfig::default()),
        &solana_config_program::id(),
    )
    .unwrap()
}

#[test]
fn test_process_create_ok() {
    let mollusk = setup();

    let config = Pubkey::new_unique();
    let config_account = {
        let space = get_config_space(0);
        let lamports = mollusk.sysvars.rent.minimum_balance(space);
        Account::new(lamports, space, &solana_config_program::id())
    };

    // `instruction::initialize_account` without making it public...
    let instruction = {
        let account_metas = vec![AccountMeta::new(config, true)];
        let account_data = (ConfigKeys { keys: vec![] }, MyConfig::default());
        Instruction::new_with_bincode(solana_config_program::id(), &account_data, account_metas)
    };

    mollusk.process_and_validate_instruction(
        &instruction,
        &[(config, config_account)],
        &[
            Check::success(),
            Check::compute_units(594),
            Check::account(&config)
                .data(
                    &bincode::serialize(&(ConfigKeys { keys: vec![] }, MyConfig::default()))
                        .unwrap(),
                )
                .build(),
        ],
    );
}

#[test]
fn test_process_store_ok() {
    let mollusk = setup();

    let config = Pubkey::new_unique();
    let keys = vec![];
    let my_config = MyConfig::new(42);

    let config_account = create_config_account(&mollusk, keys.clone());

    let instruction = config_instruction::store(&config, true, keys.clone(), &my_config);

    mollusk.process_and_validate_instruction(
        &instruction,
        &[(config, config_account)],
        &[
            Check::success(),
            Check::compute_units(594),
            Check::account(&config)
                .data(&bincode::serialize(&(ConfigKeys { keys }, my_config)).unwrap())
                .build(),
        ],
    );
}

#[test]
fn test_process_store_fail_instruction_data_too_large() {
    // [Core BPF]: To be clear, this is testing instruction data that's too
    // large for the keys list provided, not the max deserialize length.
    let mollusk = setup();

    let config = Pubkey::new_unique();
    let keys = vec![];
    let my_config = MyConfig::new(42);

    let config_account = create_config_account(&mollusk, keys.clone());

    let mut instruction = config_instruction::store(&config, true, keys, &my_config);
    instruction.data = vec![0; 123]; // <-- Replace data with a vector that's too large

    mollusk.process_and_validate_instruction(
        &instruction,
        &[(config, config_account)],
        &[Check::err(ProgramError::InvalidInstructionData)],
    );
}

#[test]
fn test_process_store_fail_account0_not_signer() {
    let mollusk = setup();

    let config = Pubkey::new_unique();
    let keys = vec![];
    let my_config = MyConfig::new(42);

    let config_account = create_config_account(&mollusk, keys.clone());

    let mut instruction = config_instruction::store(&config, true, keys, &my_config);
    instruction.accounts[0].is_signer = false; // <----- not a signer

    mollusk.process_and_validate_instruction(
        &instruction,
        &[(config, config_account)],
        &[Check::err(ProgramError::MissingRequiredSignature)],
    );
}

#[test]
fn test_process_store_with_additional_signers() {
    let mollusk = setup();

    let config = Pubkey::new_unique();

    let pubkey = Pubkey::new_unique();
    let signer0 = Pubkey::new_unique();
    let signer1 = Pubkey::new_unique();
    let keys = vec![(pubkey, false), (signer0, true), (signer1, true)];
    let my_config = MyConfig::new(42);

    let config_account = create_config_account(&mollusk, keys.clone());

    let instruction = config_instruction::store(&config, true, keys.clone(), &my_config);

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (config, config_account),
            (signer0, Account::default()),
            (signer1, Account::default()),
        ],
        &[
            Check::success(),
            Check::compute_units(3_209),
            Check::account(&config)
                .data(&bincode::serialize(&(ConfigKeys { keys }, my_config)).unwrap())
                .build(),
        ],
    );
}

#[test]
fn test_process_store_bad_config_account() {
    let mollusk = setup();

    let config = Pubkey::new_unique();

    let pubkey = Pubkey::new_unique();
    let signer0 = Pubkey::new_unique();
    let keys = vec![(pubkey, false), (signer0, true)];
    let my_config = MyConfig::new(42);

    let mut instruction = config_instruction::store(&config, false, keys, &my_config);
    instruction.accounts.remove(0); // Config popped out of instruction.

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            // Config missing from accounts.
            (
                signer0,
                Account::new(100_000, 0, &solana_config_program::id()),
            ),
        ],
        &[Check::err(ProgramError::InvalidAccountData)],
    );
}

#[test]
fn test_process_store_with_bad_additional_signer() {
    let mollusk = setup();

    let config = Pubkey::new_unique();

    let signer0 = Pubkey::new_unique();
    let keys = vec![(signer0, true)];
    let my_config = MyConfig::new(42);

    let bad_signer = Pubkey::new_unique();

    let config_account = create_config_account(&mollusk, keys.clone());

    // Config-data pubkey doesn't match signer.
    let mut instruction = config_instruction::store(&config, true, keys.clone(), &my_config);
    instruction.accounts[1] = AccountMeta::new(bad_signer, true);

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (config, config_account.clone()),
            (bad_signer, Account::default()),
        ],
        &[Check::err(ProgramError::MissingRequiredSignature)],
    );

    // Config-data pubkey not a signer.
    let mut instruction = config_instruction::store(&config, true, keys, &my_config);
    instruction.accounts[1].is_signer = false;

    mollusk.process_and_validate_instruction(
        &instruction,
        &[(config, config_account), (signer0, Account::default())],
        &[Check::err(ProgramError::MissingRequiredSignature)],
    );
}

#[test]
fn test_store_requiring_config() {
    let mollusk = setup();

    let config = Pubkey::new_unique();

    // New keys contains the config account, as well as another signer.
    let signer = Pubkey::new_unique();
    let new_keys = vec![(config, true), (signer, true)];
    let my_config = MyConfig::new(42);

    let config_account = {
        // Allocate enough space for they `new_keys`, but leave the account
        // uninitalized.
        let space = get_config_space(new_keys.len());
        let lamports = mollusk.sysvars.rent.minimum_balance(space);
        Account::new(lamports, space, &solana_config_program::id())
    };

    let mut instruction = config_instruction::store(&config, true, new_keys, &my_config);
    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (config, config_account.clone()),
            (signer, Account::default()),
        ],
        &[Check::err(ProgramError::MissingRequiredSignature)],
    );

    // This is kind of strange, since the `store` helper was taken directly
    // from the builtin crate, and it's designed to only add the config account
    // once, even if it's a signer.
    // However, if you include it in the instruction twice, the loop-counter
    // mechanism of the processor actually works...
    instruction.accounts = vec![
        AccountMeta::new(config, true),
        AccountMeta::new(config, true),
        AccountMeta::new(signer, true),
    ];
    mollusk.process_and_validate_instruction(
        &instruction,
        &[(config, config_account), (signer, Account::default())],
        &[Check::success()],
    );
}

#[test]
fn test_config_updates() {
    let mollusk = setup();

    let config = Pubkey::new_unique();

    let pubkey = Pubkey::new_unique();
    let signer0 = Pubkey::new_unique();
    let signer1 = Pubkey::new_unique();
    let signer2 = Pubkey::new_unique();
    let keys = vec![(pubkey, false), (signer0, true), (signer1, true)];
    let my_config = MyConfig::new(42);

    let config_account = create_config_account(&mollusk, keys.clone());

    let instruction = config_instruction::store(&config, true, keys.clone(), &my_config);
    let result = mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (config, config_account),
            (signer0, Account::default()),
            (signer1, Account::default()),
        ],
        &[Check::success(), Check::compute_units(3_209)],
    );

    // Use this for next invoke.
    let updated_config_account = result.get_account(&config).unwrap().to_owned();

    // Update with expected signatures.
    let new_config = MyConfig::new(84);
    let instruction = config_instruction::store(&config, false, keys.clone(), &new_config);
    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (config, updated_config_account),
            (signer0, Account::default()),
            (signer1, Account::default()),
        ],
        &[
            Check::success(),
            Check::compute_units(3_210),
            Check::account(&config)
                .data(&bincode::serialize(&(ConfigKeys { keys }, new_config)).unwrap())
                .build(),
        ],
    );

    // Use this for next invoke.
    let updated_config_account = result.get_account(&config).unwrap();

    // Attempt update with incomplete signatures.
    let keys = vec![
        (pubkey, false),
        (signer1, true), // Missing signer0.
    ];
    let instruction = config_instruction::store(&config, false, keys, &my_config);
    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (config, updated_config_account.clone()),
            // Missing signer0.
            (signer1, Account::default()),
        ],
        &[Check::err(ProgramError::MissingRequiredSignature)],
    );

    // Do it again, this time missing signer1.
    let keys = vec![
        (pubkey, false),
        (signer0, true), // Missing signer1.
    ];
    let instruction = config_instruction::store(&config, false, keys, &my_config);
    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (config, updated_config_account.clone()),
            (signer0, Account::default()),
            // Missing signer1.
        ],
        &[Check::err(ProgramError::MissingRequiredSignature)],
    );

    // Attempt update with incorrect signatures.
    let keys = vec![
        (pubkey, false),
        (signer0, true),
        (signer2, true), // Incorrect signer1.
    ];
    let instruction = config_instruction::store(&config, false, keys, &my_config);
    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (config, updated_config_account.clone()),
            (signer0, Account::default()),
            (signer2, Account::default()), // Incorrect signer1.
        ],
        &[Check::err(ProgramError::MissingRequiredSignature)],
    );
}

#[test]
fn test_config_initialize_contains_duplicates_fails() {
    let mollusk = setup();

    let config = Pubkey::new_unique();

    let pubkey = Pubkey::new_unique();
    let signer0 = Pubkey::new_unique();
    let keys = vec![
        (pubkey, false),
        (signer0, true),
        (signer0, true), // Duplicate signer0.
    ];
    let my_config = MyConfig::new(42);

    let config_account = create_config_account(&mollusk, keys.clone());

    // Attempt initialization with duplicate signer inputs.
    let instruction = config_instruction::store(&config, true, keys, &my_config);

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (config, config_account),
            (signer0, Account::default()),
            (signer0, Account::default()), // Duplicate signer0.
        ],
        &[Check::err(ProgramError::InvalidArgument)],
    );
}

#[test]
fn test_config_update_contains_duplicates_fails() {
    let mollusk = setup();

    let config = Pubkey::new_unique();

    let pubkey = Pubkey::new_unique();
    let signer0 = Pubkey::new_unique();
    let signer1 = Pubkey::new_unique();
    let keys = vec![(pubkey, false), (signer0, true), (signer1, true)];
    let my_config = MyConfig::new(42);

    let config_account = create_config_account(&mollusk, keys.clone());

    let instruction = config_instruction::store(&config, true, keys, &my_config);
    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (config, config_account.clone()),
            (signer0, Account::default()),
            (signer1, Account::default()),
        ],
        &[Check::success(), Check::compute_units(3_209)],
    );

    // Attempt update with duplicate signer inputs.
    let new_config = MyConfig::new(84);
    let dupe_keys = vec![
        (pubkey, false),
        (signer0, true),
        (signer0, true), // Duplicate signer0.
    ];
    let instruction = config_instruction::store(&config, true, dupe_keys, &new_config);
    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (config, config_account),
            (signer0, Account::default()),
            (signer0, Account::default()), // Duplicate signer0.
        ],
        &[Check::err(ProgramError::InvalidArgument)],
    );
}

#[test]
fn test_config_updates_requiring_config() {
    let mollusk = setup();

    let config = Pubkey::new_unique();

    let pubkey = Pubkey::new_unique();
    let signer0 = Pubkey::new_unique();
    let keys = vec![(pubkey, false), (signer0, true), (config, true)];
    let my_config = MyConfig::new(42);

    let config_account = create_config_account(&mollusk, keys.clone());

    let instruction = config_instruction::store(&config, true, keys.clone(), &my_config);
    let result = mollusk.process_and_validate_instruction(
        &instruction,
        &[(config, config_account), (signer0, Account::default())],
        &[
            Check::success(),
            Check::compute_units(3_303),
            Check::account(&config)
                .data(&bincode::serialize(&(ConfigKeys { keys: keys.clone() }, my_config)).unwrap())
                .build(),
        ],
    );

    // Use this for next invoke.
    let updated_config_account = result.get_account(&config).unwrap().to_owned();

    // Update with expected signatures.
    let new_config = MyConfig::new(84);
    let instruction = config_instruction::store(&config, true, keys.clone(), &new_config);
    let result = mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (config, updated_config_account),
            (signer0, Account::default()),
        ],
        &[
            Check::success(),
            Check::compute_units(3_303),
            Check::account(&config)
                .data(&bincode::serialize(&(ConfigKeys { keys }, new_config)).unwrap())
                .build(),
        ],
    );

    // Use this for next invoke.
    let updated_config_account = result.get_account(&config).unwrap().to_owned();

    // Attempt update with incomplete signatures.
    let keys = vec![(pubkey, false), (config, true)]; // Missing signer0.
    let new_config = MyConfig::new(128);
    let instruction = config_instruction::store(&config, true, keys, &new_config);
    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (config, updated_config_account),
            (signer0, Account::default()),
        ],
        &[Check::err(ProgramError::MissingRequiredSignature)],
    );
}

#[test]
fn test_config_initialize_no_panic() {
    let mollusk = setup();

    let config = Pubkey::new_unique();
    let max_space = serialized_size(&MyConfig::default()).unwrap();

    let instructions = config_instruction::create_account_with_max_config_space::<MyConfig>(
        &Pubkey::new_unique(),
        &config,
        1,
        max_space,
        vec![],
    );
    let mut instruction = instructions[1].clone();
    instruction.accounts = vec![];

    mollusk.process_and_validate_instruction(
        &instruction,
        &[],
        &[Check::err(ProgramError::NotEnoughAccountKeys)],
    );
}

#[test]
fn test_config_bad_owner() {
    let mollusk = setup();

    let config = Pubkey::new_unique();

    let pubkey = Pubkey::new_unique();
    let signer0 = Pubkey::new_unique();
    let keys = vec![(pubkey, false), (signer0, true), (config, true)];
    let my_config = MyConfig::new(42);

    // Store a config account with the wrong owner.
    let config_account = {
        let space = get_config_space(keys.len());
        let lamports = mollusk.sysvars.rent.minimum_balance(space);
        Account::new(lamports, 0, &Pubkey::new_unique())
    };

    let instruction = config_instruction::store(&config, true, keys, &my_config);

    mollusk.process_and_validate_instruction(
        &instruction,
        &[(config, config_account), (signer0, Account::default())],
        &[Check::err(ProgramError::InvalidAccountOwner)],
    );
}

#[test]
fn test_maximum_keys_input() {
    // `limited_deserialize` allows up to 1232 bytes of input.
    // One config key is `Pubkey` + `bool` = 32 + 1 = 33 bytes.
    // 1232 / 33 = 37 keys max.
    let mollusk = setup();

    let config = Pubkey::new_unique();

    // First store with 37 keys.
    let mut keys = vec![];
    for _ in 0..37 {
        keys.push((Pubkey::new_unique(), false));
    }
    let my_config = MyConfig::new(42);

    let config_account = create_config_account(&mollusk, keys.clone());

    let instruction = config_instruction::store(&config, true, keys.clone(), &my_config);
    let result = mollusk.process_and_validate_instruction(
        &instruction,
        &[(config, config_account)],
        &[Check::success(), Check::compute_units(25_756)],
    );

    // Use this for next invoke.
    let updated_config_account = result.get_account(&config).unwrap().to_owned();

    // Do an update with 37 keys, forcing the program to deserialize the
    // config account data.
    let new_config = MyConfig::new(84);
    let instruction = config_instruction::store(&config, true, keys.clone(), &new_config);
    let result = mollusk.process_and_validate_instruction(
        &instruction,
        &[(config, updated_config_account)],
        &[Check::success(), Check::compute_units(25_756)],
    );

    // Use this for next invoke.
    let updated_config_account = result.get_account(&config).unwrap().to_owned();

    // Now try to store with 38 keys.
    keys.push((Pubkey::new_unique(), false));
    let my_config = MyConfig::new(42);
    let instruction = config_instruction::store(&config, true, keys, &my_config);
    mollusk.process_and_validate_instruction(
        &instruction,
        &[(config, updated_config_account)],
        &[Check::err(ProgramError::InvalidInstructionData)],
    );
}

#[test]
fn test_safe_deserialize() {
    let mollusk = setup();

    // Accounts don't matter for this test.

    // First try to spoof the program with just `ShortU16` length values.
    let build_instruction =
        |data: &[u8]| Instruction::new_with_bytes(solana_config_program::id(), data, vec![]);

    mollusk.process_and_validate_instruction(
        // Empty buffer. Not a valid `ShortU16`.
        &build_instruction(&[]),
        &[],
        &[Check::err(ProgramError::InvalidInstructionData)],
    );

    mollusk.process_and_validate_instruction(
        // `ShortU16` value of 38. One byte too large.
        &build_instruction(&[0x26]),
        &[],
        &[Check::err(ProgramError::InvalidInstructionData)],
    );

    mollusk.process_and_validate_instruction(
        // `ShortU16` value of 37. OK for vector size, but no keys following.
        &build_instruction(&[0x25]),
        &[],
        &[Check::err(ProgramError::InvalidInstructionData)],
    );

    // Now try with some actual `ConfigKeys` inputs.
    let mut keys = Vec::new();
    let serialized_config_keys = |keys: &[(Pubkey, bool)]| {
        let config_keys = ConfigKeys {
            keys: keys.to_vec(),
        };
        bincode::serialize(&config_keys).unwrap()
    };

    // First build out to an acceptable size of 37.
    (0..37).for_each(|i| keys.push((Pubkey::new_unique(), i % 2 == 0)));

    mollusk.process_and_validate_instruction(
        // `ShortU16` value of 37. OK.
        &build_instruction(&serialized_config_keys(&keys)),
        &[],
        // Falls through to account keys failure.
        &[Check::err(ProgramError::NotEnoughAccountKeys)],
    );

    // Add one more key, pushing the size to 38.
    keys.push((Pubkey::new_unique(), true));

    mollusk.process_and_validate_instruction(
        // `ShortU16` value of 38. Err.
        &build_instruction(&serialized_config_keys(&keys)),
        &[],
        &[Check::err(ProgramError::InvalidInstructionData)],
    );
}

#[test]
fn test_safe_deserialize_from_state() {
    let mollusk = setup();

    let config = Pubkey::new_unique();

    let keys = vec![(config, false), (config, true)];
    let my_config = MyConfig::new(42);

    // Input doesn't matter for this test.
    let instruction = config_instruction::store(&config, false, keys.clone(), &my_config);

    // Store the keys in the config account, but give it the wrong length.
    let config_account = {
        let space = bincode::serialized_size(&ConfigKeys { keys: keys.clone() }).unwrap() as usize;
        let lamports = mollusk.sysvars.rent.minimum_balance(space);

        let mut data = vec![0; space];
        bincode::serialize_into(&mut data, &ConfigKeys { keys }).unwrap();
        data[0] = 255; // length of 255.

        let mut account = Account::new(lamports, space, &solana_config_program::id());
        account.data = data;
        account
    };

    mollusk.process_and_validate_instruction(
        &instruction,
        &[(config, config_account)],
        &[Check::err(ProgramError::InvalidAccountData)],
    );
}

// Backwards compatibility test case.
#[test]
fn test_write_same_data_to_readonly() {
    let mollusk = setup();

    let config = Pubkey::new_unique();
    let keys = vec![];

    // Creates a config account with `MyConfig::default()`.
    let config_account = create_config_account(&mollusk, keys.clone());

    // Pass the exact same data (`MyConfig::default()`) to the instruction,
    // which we'll attempt to write into the account.
    let mut instruction =
        config_instruction::store(&config, true, keys.clone(), &MyConfig::default());

    // Make the config account read-only.
    instruction.accounts[0].is_writable = false;

    mollusk.process_and_validate_instruction(
        &instruction,
        &[(config, config_account)],
        &[Check::err(ProgramError::Custom(
            ConfigError::ReadonlyDataModified as u32,
        ))],
    );
}
