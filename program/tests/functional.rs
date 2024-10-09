#![cfg(feature = "test-sbf")]

use {
    bincode::serialized_size,
    mollusk_svm::{result::Check, Mollusk},
    serde::{Deserialize, Serialize},
    solana_config_program::{
        instruction as config_instruction,
        state::{ConfigKeys, ConfigState},
    },
    solana_sdk::{
        account::AccountSharedData,
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

impl ConfigState for MyConfig {
    fn max_space() -> u64 {
        serialized_size(&Self::default()).unwrap()
    }
}

fn setup() -> Mollusk {
    Mollusk::new(&solana_config_program::id(), "solana_config_program")
}

fn get_config_space(key_len: usize) -> usize {
    let entry_size = bincode::serialized_size(&(Pubkey::default(), true)).unwrap() as usize;
    bincode::serialized_size(&(ConfigKeys::default(), MyConfig::default())).unwrap() as usize
        + key_len * entry_size
}

fn create_config_account(mollusk: &Mollusk, keys: Vec<(Pubkey, bool)>) -> AccountSharedData {
    let space = get_config_space(keys.len());
    let lamports = mollusk.sysvars.rent.minimum_balance(space as usize);
    AccountSharedData::new_data(
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
        AccountSharedData::new(lamports, space as usize, &solana_config_program::id())
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
            (signer0, AccountSharedData::default()),
            (signer1, AccountSharedData::default()),
        ],
        &[
            Check::success(),
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
                AccountSharedData::new(100_000, 0, &solana_config_program::id()),
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
            (bad_signer, AccountSharedData::default()),
        ],
        &[Check::err(ProgramError::MissingRequiredSignature)],
    );

    // Config-data pubkey not a signer.
    let mut instruction = config_instruction::store(&config, true, keys, &my_config);
    instruction.accounts[1].is_signer = false;

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (config, config_account),
            (signer0, AccountSharedData::default()),
        ],
        &[Check::err(ProgramError::MissingRequiredSignature)],
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
            (signer0, AccountSharedData::default()),
            (signer1, AccountSharedData::default()),
        ],
        &[Check::success()],
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
            (signer0, AccountSharedData::default()),
            (signer1, AccountSharedData::default()),
        ],
        &[
            Check::success(),
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
        (signer0, true), // Missing signer1.
    ];
    let instruction = config_instruction::store(&config, false, keys, &my_config);
    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (config, updated_config_account.clone()),
            (signer0, AccountSharedData::default()), // Missing signer1.
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
            (signer0, AccountSharedData::default()),
            (signer2, AccountSharedData::default()), // Incorrect signer1.
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
            (signer0, AccountSharedData::default()),
            (signer0, AccountSharedData::default()), // Duplicate signer0.
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
            (signer0, AccountSharedData::default()),
            (signer1, AccountSharedData::default()),
        ],
        &[Check::success()],
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
            (signer0, AccountSharedData::default()),
            (signer0, AccountSharedData::default()), // Duplicate signer0.
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
        &[
            (config, config_account),
            (signer0, AccountSharedData::default()),
        ],
        &[Check::success()],
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
            (signer0, AccountSharedData::default()),
        ],
        &[
            Check::success(),
            Check::account(&config)
                .data(&bincode::serialize(&(ConfigKeys { keys }, new_config)).unwrap())
                .build(),
        ],
    );

    // Use this for next invoke.
    let updated_config_account = result.get_account(&config).unwrap().to_owned();

    // Attempt update with incomplete signatures.
    let keys = vec![(pubkey, false), (config, true)]; // Missing signer0.
    let instruction = config_instruction::store(&config, true, keys, &my_config);
    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (config, updated_config_account),
            (signer0, AccountSharedData::default()),
        ],
        &[Check::err(ProgramError::MissingRequiredSignature)],
    );
}

#[test]
fn test_config_initialize_no_panic() {
    let mollusk = setup();

    let config = Pubkey::new_unique();

    let instructions =
        config_instruction::create_account::<MyConfig>(&Pubkey::new_unique(), &config, 1, vec![]);
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
        let lamports = mollusk.sysvars.rent.minimum_balance(space as usize);
        AccountSharedData::new(lamports, 0, &Pubkey::new_unique())
    };

    let instruction = config_instruction::store(&config, true, keys, &my_config);

    mollusk.process_and_validate_instruction(
        &instruction,
        &[
            (config, config_account),
            (signer0, AccountSharedData::default()),
        ],
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
        &[Check::success()],
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
        &[Check::success()],
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
