#![cfg(feature = "test-sbf")]

use {
    bincode::{deserialize, serialized_size},
    serde::{Deserialize, Serialize},
    solana_config_program::{
        instruction as config_instruction,
        state::{get_config_data, ConfigKeys, ConfigState},
    },
    solana_program_test::*,
    solana_sdk::{
        account::{AccountSharedData, ReadableAccount},
        instruction::{AccountMeta, InstructionError},
        pubkey::Pubkey,
        signature::{Keypair, Signer},
        transaction::{Transaction, TransactionError},
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
    pub fn deserialize(input: &[u8]) -> Option<Self> {
        deserialize(input).ok()
    }
}

impl ConfigState for MyConfig {
    fn max_space() -> u64 {
        serialized_size(&Self::default()).unwrap()
    }
}

async fn setup_test_context() -> ProgramTestContext {
    let mut program_test = ProgramTest::default();
    program_test.prefer_bpf(true);
    program_test.add_program(
        "solana_config_program",
        solana_config_program::id(),
        processor!(solana_config_program::processor::process),
    );
    program_test.start_with_context().await
}

fn get_config_space(key_len: usize) -> usize {
    let entry_size = bincode::serialized_size(&(Pubkey::default(), true)).unwrap() as usize;
    bincode::serialized_size(&(ConfigKeys::default(), MyConfig::default())).unwrap() as usize
        + key_len * entry_size
}

async fn create_config_account(
    context: &mut ProgramTestContext,
    config_keypair: &Keypair,
    keys: Vec<(Pubkey, bool)>,
) {
    let payer = &context.payer;

    let space = get_config_space(keys.len());
    let lamports = context
        .banks_client
        .get_rent()
        .await
        .unwrap()
        .minimum_balance(space as usize);
    let instructions = config_instruction::create_account::<MyConfig>(
        &payer.pubkey(),
        &config_keypair.pubkey(),
        lamports,
        keys,
    );

    context
        .banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &instructions,
            Some(&payer.pubkey()),
            &[&payer, &config_keypair],
            context.last_blockhash,
        ))
        .await
        .unwrap();
}

#[tokio::test]
async fn test_process_create_ok() {
    let mut context = setup_test_context().await;
    let config_keypair = Keypair::new();
    create_config_account(&mut context, &config_keypair, vec![]).await;
    let config_account = context
        .banks_client
        .get_account(config_keypair.pubkey())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        Some(MyConfig::default()),
        deserialize(get_config_data(config_account.data()).unwrap()).ok()
    );
}

#[tokio::test]
async fn test_process_store_ok() {
    let mut context = setup_test_context().await;

    let config_keypair = Keypair::new();
    let keys = vec![];
    let my_config = MyConfig::new(42);

    create_config_account(&mut context, &config_keypair, keys.clone()).await;
    let instruction = config_instruction::store(&config_keypair.pubkey(), true, keys, &my_config);
    let payer = &context.payer;

    context
        .banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[&payer, &config_keypair],
            context.last_blockhash,
        ))
        .await
        .unwrap();

    let config_account = context
        .banks_client
        .get_account(config_keypair.pubkey())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        Some(my_config),
        deserialize(get_config_data(config_account.data()).unwrap()).ok()
    );
}

#[tokio::test]
async fn test_process_store_fail_instruction_data_too_large() {
    // [Core BPF]: To be clear, this is testing instruction data that's too
    // large for the keys list provided, not the max deserialize length.
    let mut context = setup_test_context().await;

    let config_keypair = Keypair::new();
    let keys = vec![];
    let my_config = MyConfig::new(42);

    create_config_account(&mut context, &config_keypair, keys.clone()).await;
    let mut instruction =
        config_instruction::store(&config_keypair.pubkey(), true, keys, &my_config);
    instruction.data = vec![0; 123]; // <-- Replace data with a vector that's too large
    let payer = &context.payer;

    let err = context
        .banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[&payer, &config_keypair],
            context.last_blockhash,
        ))
        .await
        .unwrap_err()
        .unwrap();
    assert_eq!(
        err,
        TransactionError::InstructionError(0, InstructionError::InvalidInstructionData)
    );
}

#[tokio::test]
async fn test_process_store_fail_account0_not_signer() {
    let mut context = setup_test_context().await;

    let config_keypair = Keypair::new();
    let keys = vec![];
    let my_config = MyConfig::new(42);

    create_config_account(&mut context, &config_keypair, keys.clone()).await;
    let mut instruction =
        config_instruction::store(&config_keypair.pubkey(), true, keys, &my_config);
    let payer = &context.payer;

    instruction.accounts[0].is_signer = false; // <----- not a signer

    let err = context
        .banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[&payer],
            context.last_blockhash,
        ))
        .await
        .unwrap_err()
        .unwrap();
    assert_eq!(
        err,
        TransactionError::InstructionError(0, InstructionError::MissingRequiredSignature)
    );
}

#[tokio::test]
async fn test_process_store_with_additional_signers() {
    let mut context = setup_test_context().await;

    let config_keypair = Keypair::new();

    let pubkey = Pubkey::new_unique();
    let signer0 = Keypair::new();
    let signer1 = Keypair::new();
    let keys = vec![
        (pubkey, false),
        (signer0.pubkey(), true),
        (signer1.pubkey(), true),
    ];
    let my_config = MyConfig::new(42);

    create_config_account(&mut context, &config_keypair, keys.clone()).await;
    let instruction =
        config_instruction::store(&config_keypair.pubkey(), true, keys.clone(), &my_config);
    let payer = &context.payer;

    context
        .banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[&payer, &config_keypair, &signer0, &signer1],
            context.last_blockhash,
        ))
        .await
        .unwrap();

    let config_account = context
        .banks_client
        .get_account(config_keypair.pubkey())
        .await
        .unwrap()
        .unwrap();
    let config_state: ConfigKeys = deserialize(config_account.data()).unwrap();
    assert_eq!(config_state.keys, keys);
    assert_eq!(
        Some(my_config),
        deserialize(get_config_data(config_account.data()).unwrap()).ok()
    );
}

#[tokio::test]
async fn test_process_store_bad_config_account() {
    let mut context = setup_test_context().await;

    let config_keypair = Keypair::new();

    let pubkey = Pubkey::new_unique();
    let signer0 = Keypair::new();
    let keys = vec![(pubkey, false), (signer0.pubkey(), true)];
    let my_config = MyConfig::new(42);

    context.set_account(
        &signer0.pubkey(),
        &AccountSharedData::new(100_000, 0, &solana_config_program::id()),
    );

    create_config_account(&mut context, &config_keypair, keys.clone()).await;
    let payer = &context.payer;

    let mut instruction =
        config_instruction::store(&config_keypair.pubkey(), false, keys, &my_config);
    instruction.accounts.remove(0); // Config popped out of instruction.

    let err = context
        .banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[&payer, &signer0],
            context.last_blockhash,
        ))
        .await
        .unwrap_err()
        .unwrap();
    assert_eq!(
        err,
        TransactionError::InstructionError(0, InstructionError::InvalidAccountData)
    );
}

#[tokio::test]
async fn test_process_store_with_bad_additional_signer() {
    let mut context = setup_test_context().await;

    let config_keypair = Keypair::new();
    let bad_signer = Keypair::new();

    let signer0 = Keypair::new();
    let keys = vec![(signer0.pubkey(), true)];
    let my_config = MyConfig::new(42);

    create_config_account(&mut context, &config_keypair, keys.clone()).await;
    let payer = &context.payer;

    // Config-data pubkey doesn't match signer.
    let mut instruction =
        config_instruction::store(&config_keypair.pubkey(), true, keys.clone(), &my_config);
    instruction.accounts[1] = AccountMeta::new(bad_signer.pubkey(), true);
    let err = context
        .banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[&payer, &config_keypair, &bad_signer],
            context.last_blockhash,
        ))
        .await
        .unwrap_err()
        .unwrap();
    assert_eq!(
        err,
        TransactionError::InstructionError(0, InstructionError::MissingRequiredSignature)
    );

    // Config-data pubkey not a signer.
    let mut instruction =
        config_instruction::store(&config_keypair.pubkey(), true, keys, &my_config);
    instruction.accounts[1].is_signer = false;
    let err = context
        .banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[&payer, &config_keypair],
            context.last_blockhash,
        ))
        .await
        .unwrap_err()
        .unwrap();
    assert_eq!(
        err,
        TransactionError::InstructionError(0, InstructionError::MissingRequiredSignature)
    );
}

#[tokio::test]
async fn test_config_updates() {
    let mut context = setup_test_context().await;

    let config_keypair = Keypair::new();

    let pubkey = Pubkey::new_unique();
    let signer0 = Keypair::new();
    let signer1 = Keypair::new();
    let signer2 = Keypair::new();
    let keys = vec![
        (pubkey, false),
        (signer0.pubkey(), true),
        (signer1.pubkey(), true),
    ];
    let my_config = MyConfig::new(42);

    create_config_account(&mut context, &config_keypair, keys.clone()).await;
    let payer = &context.payer;

    let instruction =
        config_instruction::store(&config_keypair.pubkey(), true, keys.clone(), &my_config);
    context
        .banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[&payer, &config_keypair, &signer0, &signer1],
            context.last_blockhash,
        ))
        .await
        .unwrap();

    // Update with expected signatures.
    let new_config = MyConfig::new(84);
    let instruction =
        config_instruction::store(&config_keypair.pubkey(), false, keys.clone(), &new_config);
    context
        .banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[&payer, &signer0, &signer1],
            context.last_blockhash,
        ))
        .await
        .unwrap();

    let config_account = context
        .banks_client
        .get_account(config_keypair.pubkey())
        .await
        .unwrap()
        .unwrap();
    let config_state: ConfigKeys = deserialize(config_account.data()).unwrap();
    assert_eq!(config_state.keys, keys);
    assert_eq!(
        new_config,
        MyConfig::deserialize(get_config_data(config_account.data()).unwrap()).unwrap()
    );

    // Attempt update with incomplete signatures.
    let keys = vec![
        (pubkey, false),
        (signer0.pubkey(), true), // Missing signer1.
    ];
    let instruction = config_instruction::store(&config_keypair.pubkey(), false, keys, &my_config);
    let err = context
        .banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[&payer, &signer0], // Missing signer1.
            context.last_blockhash,
        ))
        .await
        .unwrap_err()
        .unwrap();
    assert_eq!(
        err,
        TransactionError::InstructionError(0, InstructionError::MissingRequiredSignature)
    );

    // Attempt update with incorrect signatures.
    let keys = vec![
        (pubkey, false),
        (signer0.pubkey(), true),
        (signer2.pubkey(), true), // Incorrect signer1.
    ];
    let instruction = config_instruction::store(&config_keypair.pubkey(), false, keys, &my_config);
    let err = context
        .banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[&payer, &signer0, &signer2], // Incorrect signer1.
            context.last_blockhash,
        ))
        .await
        .unwrap_err()
        .unwrap();
    assert_eq!(
        err,
        TransactionError::InstructionError(0, InstructionError::MissingRequiredSignature)
    );
}

#[tokio::test]
async fn test_config_initialize_contains_duplicates_fails() {
    let mut context = setup_test_context().await;

    let config_keypair = Keypair::new();

    let pubkey = Pubkey::new_unique();
    let signer0 = Keypair::new();
    let keys = vec![
        (pubkey, false),
        (signer0.pubkey(), true),
        (signer0.pubkey(), true), // Duplicate signer0.
    ];
    let my_config = MyConfig::new(42);

    create_config_account(&mut context, &config_keypair, keys.clone()).await;
    let payer = &context.payer;

    // Attempt initialization with duplicate signer inputs.
    let instruction = config_instruction::store(&config_keypair.pubkey(), true, keys, &my_config);
    let err = context
        .banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[&payer, &config_keypair, &signer0],
            context.last_blockhash,
        ))
        .await
        .unwrap_err()
        .unwrap();
    assert_eq!(
        err,
        TransactionError::InstructionError(0, InstructionError::InvalidArgument)
    );
}

#[tokio::test]
async fn test_config_update_contains_duplicates_fails() {
    let mut context = setup_test_context().await;

    let config_keypair = Keypair::new();

    let pubkey = Pubkey::new_unique();
    let signer0 = Keypair::new();
    let signer1 = Keypair::new();
    let keys = vec![
        (pubkey, false),
        (signer0.pubkey(), true),
        (signer1.pubkey(), true),
    ];
    let my_config = MyConfig::new(42);

    create_config_account(&mut context, &config_keypair, keys.clone()).await;
    let payer = &context.payer;

    let instruction = config_instruction::store(&config_keypair.pubkey(), true, keys, &my_config);
    context
        .banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[&payer, &config_keypair, &signer0, &signer1],
            context.last_blockhash,
        ))
        .await
        .unwrap();

    // Attempt update with duplicate signer inputs.
    let new_config = MyConfig::new(84);
    let dupe_keys = vec![
        (pubkey, false),
        (signer0.pubkey(), true),
        (signer0.pubkey(), true), // Duplicate signer0.
    ];
    let instruction =
        config_instruction::store(&config_keypair.pubkey(), true, dupe_keys, &new_config);
    let err = context
        .banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[&payer, &config_keypair, &signer0],
            context.last_blockhash,
        ))
        .await
        .unwrap_err()
        .unwrap();
    assert_eq!(
        err,
        TransactionError::InstructionError(0, InstructionError::InvalidArgument)
    );
}

#[tokio::test]
async fn test_config_updates_requiring_config() {
    let mut context = setup_test_context().await;

    let config_keypair = Keypair::new();

    let pubkey = Pubkey::new_unique();
    let signer0 = Keypair::new();
    let keys = vec![
        (pubkey, false),
        (signer0.pubkey(), true),
        (config_keypair.pubkey(), true),
    ];
    let my_config = MyConfig::new(42);

    create_config_account(
        &mut context,
        &config_keypair,
        vec![(pubkey, false), (pubkey, false), (pubkey, false)], // Dummy keys for account sizing.
    )
    .await;
    let payer = &context.payer;

    let instruction =
        config_instruction::store(&config_keypair.pubkey(), true, keys.clone(), &my_config);
    context
        .banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[&payer, &config_keypair, &signer0],
            context.last_blockhash,
        ))
        .await
        .unwrap();

    // Update with expected signatures.
    let new_config = MyConfig::new(84);
    let instruction =
        config_instruction::store(&config_keypair.pubkey(), true, keys.clone(), &new_config);
    context
        .banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[&payer, &config_keypair, &signer0],
            context.last_blockhash,
        ))
        .await
        .unwrap();

    let config_account = context
        .banks_client
        .get_account(config_keypair.pubkey())
        .await
        .unwrap()
        .unwrap();
    let config_state: ConfigKeys = deserialize(config_account.data()).unwrap();
    assert_eq!(config_state.keys, keys);
    assert_eq!(
        Some(new_config),
        deserialize(get_config_data(config_account.data()).unwrap()).ok()
    );

    // Attempt update with incomplete signatures.
    let keys = vec![(pubkey, false), (config_keypair.pubkey(), true)]; // Missing signer0.
    let instruction = config_instruction::store(&config_keypair.pubkey(), true, keys, &my_config);
    let err = context
        .banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[&payer, &config_keypair], // Missing signer0.
            context.last_blockhash,
        ))
        .await
        .unwrap_err()
        .unwrap();
    assert_eq!(
        err,
        TransactionError::InstructionError(0, InstructionError::MissingRequiredSignature)
    );
}

#[tokio::test]
async fn test_config_initialize_no_panic() {
    let mut context = setup_test_context().await;
    let config_keypair = Keypair::new();
    create_config_account(&mut context, &config_keypair, vec![]).await;
    let payer = &context.payer;

    let instructions = config_instruction::create_account::<MyConfig>(
        &payer.pubkey(),
        &config_keypair.pubkey(),
        1,
        vec![],
    );
    let mut instruction = instructions[1].clone();
    instruction.accounts = vec![];

    let err = context
        .banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[&payer],
            context.last_blockhash,
        ))
        .await
        .unwrap_err()
        .unwrap();
    assert_eq!(
        err,
        TransactionError::InstructionError(0, InstructionError::NotEnoughAccountKeys)
    );
}

#[tokio::test]
async fn test_config_bad_owner() {
    let mut context = setup_test_context().await;

    let config_keypair = Keypair::new();

    let pubkey = Pubkey::new_unique();
    let signer0 = Keypair::new();
    let keys = vec![
        (pubkey, false),
        (signer0.pubkey(), true),
        (config_keypair.pubkey(), true),
    ];
    let my_config = MyConfig::new(42);

    // Store a config account with the wrong owner.
    let space = get_config_space(keys.len());
    let lamports = context
        .banks_client
        .get_rent()
        .await
        .unwrap()
        .minimum_balance(space as usize);
    context.set_account(
        &config_keypair.pubkey(),
        &AccountSharedData::new(lamports, 0, &Pubkey::new_unique()),
    );

    let payer = &context.payer;

    let instruction = config_instruction::store(&config_keypair.pubkey(), true, keys, &my_config);
    let err = context
        .banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[&payer, &config_keypair, &signer0],
            context.last_blockhash,
        ))
        .await
        .unwrap_err()
        .unwrap();
    assert_eq!(
        err,
        TransactionError::InstructionError(0, InstructionError::InvalidAccountOwner)
    );
}

#[tokio::test]
async fn test_maximum_keys_input() {
    // `limited_deserialize` allows up to 1232 bytes of input.
    // One config key is `Pubkey` + `bool` = 32 + 1 = 33 bytes.
    // 1232 / 33 = 37 keys max.
    let mut context = setup_test_context().await;

    let config_keypair = Keypair::new();

    // First store with 37 keys.
    let mut keys = vec![];
    for _ in 0..37 {
        keys.push((Pubkey::new_unique(), false));
    }
    let my_config = MyConfig::new(42);

    create_config_account(&mut context, &config_keypair, keys.clone()).await;
    let instruction =
        config_instruction::store(&config_keypair.pubkey(), true, keys.clone(), &my_config);
    let payer = &context.payer;

    context
        .banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[&payer, &config_keypair],
            context.last_blockhash,
        ))
        .await
        .unwrap();

    let config_account = context
        .banks_client
        .get_account(config_keypair.pubkey())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        Some(my_config),
        deserialize(get_config_data(config_account.data()).unwrap()).ok()
    );

    // Do an update with 37 keys, forcing the program to deserialize the
    // config account data.
    let new_config = MyConfig::new(84);
    let instruction =
        config_instruction::store(&config_keypair.pubkey(), true, keys.clone(), &new_config);
    context
        .banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[&payer, &config_keypair],
            context.last_blockhash,
        ))
        .await
        .unwrap();

    // Now try to store with 38 keys.
    keys.push((Pubkey::new_unique(), false));
    let my_config = MyConfig::new(42);
    let instruction = config_instruction::store(&config_keypair.pubkey(), true, keys, &my_config);

    let err = context
        .banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[&payer, &config_keypair],
            context.last_blockhash,
        ))
        .await
        .unwrap_err()
        .unwrap();
    assert_eq!(
        err,
        TransactionError::InstructionError(0, InstructionError::InvalidInstructionData)
    );
}
