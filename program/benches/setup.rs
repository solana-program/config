use {
    mollusk_svm_bencher::Bench,
    serde::Serialize,
    solana_config_program::state::ConfigKeys,
    solana_config_program_client::instructions_bincode::{store, ConfigState},
    solana_sdk::{
        account::AccountSharedData,
        hash::Hash,
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        rent::Rent,
    },
};

/// Helper struct to convert to a `Bench`.
pub struct BenchContext {
    label: String,
    instruction: Instruction,
    accounts: Vec<(Pubkey, AccountSharedData)>,
}

impl BenchContext {
    /// Convert to a `Bench`.
    pub fn bench(&self) -> Bench {
        (self.label.as_str(), &self.instruction, &self.accounts)
    }
}

/// Trait to avoid re-defining the same instruction and account constructors
/// for each `ConfigState`.
pub trait BenchSetup: ConfigState + Default {
    const BENCH_ID: &'static str;

    fn default_account_state(keys: Vec<(Pubkey, bool)>) -> (ConfigKeys, Self) {
        (ConfigKeys { keys }, Self::default())
    }

    fn default_space(keys: Vec<(Pubkey, bool)>) -> usize {
        (Self::max_space()
            .checked_add(bincode::serialized_size(&ConfigKeys { keys }).unwrap())
            .unwrap()) as usize
    }

    fn keys(keys_len: usize) -> Vec<(Pubkey, bool)> {
        (0..keys_len)
            .map(|_| (Pubkey::new_unique(), false))
            .collect()
    }

    fn init(keys_len: usize) -> BenchContext {
        let config_pubkey = Pubkey::new_unique();
        let keys = Self::keys(keys_len);
        let space = Self::default_space(keys.clone());
        let lamports = Rent::default().minimum_balance(space);

        let instruction = {
            let account_metas = vec![AccountMeta::new(config_pubkey, true)];
            let account_data = Self::default_account_state(keys);
            Instruction::new_with_bincode(solana_config_program::id(), &account_data, account_metas)
        };

        let accounts = vec![(
            config_pubkey,
            AccountSharedData::new(lamports, space, &solana_config_program::id()),
        )];

        BenchContext {
            label: format!("{}_init_{}_keys", Self::BENCH_ID, keys_len),
            instruction,
            accounts,
        }
    }

    fn store(keys_len: usize) -> BenchContext {
        let config_pubkey = Pubkey::new_unique();
        let keys = Self::keys(keys_len);
        let space = Self::default_space(keys.clone());
        let lamports = Rent::default().minimum_balance(space);

        let instruction = store(&config_pubkey, true, keys.clone(), &Self::default());

        let accounts = vec![(
            config_pubkey,
            AccountSharedData::new_data(
                lamports,
                &Self::default_account_state(keys),
                &solana_config_program::id(),
            )
            .unwrap(),
        )];

        BenchContext {
            label: format!("{}_store_{}_keys", Self::BENCH_ID, keys_len),
            instruction,
            accounts,
        }
    }
}

/// A small config, which just stores 8 bytes.
#[derive(Debug, Default, PartialEq, Serialize)]
pub struct ConfigSmall {
    pub item: u64,
}

impl ConfigState for ConfigSmall {
    fn max_space() -> u64 {
        bincode::serialized_size(&Self::default()).unwrap()
    }
}

impl BenchSetup for ConfigSmall {
    const BENCH_ID: &'static str = "config_small";
}

/// A medium config, which stores 1024 bytes.
#[derive(Debug, Default, PartialEq, Serialize)]
pub struct ConfigMedium {
    pub hashes: [Hash; 32], // 32 x 32 = 1024 bytes
}

impl ConfigState for ConfigMedium {
    fn max_space() -> u64 {
        bincode::serialized_size(&Self::default()).unwrap()
    }
}

impl BenchSetup for ConfigMedium {
    const BENCH_ID: &'static str = "config_medium";
}

/// A large config, which stores 32_768 bytes.
#[derive(Debug, Default, PartialEq, Serialize)]
pub struct ConfigLarge {
    pub hashes: [[Hash; 32]; 32], // 32 x 32 x 32 = 32_768 bytes
}

impl ConfigState for ConfigLarge {
    fn max_space() -> u64 {
        bincode::serialized_size(&Self::default()).unwrap()
    }
}

impl BenchSetup for ConfigLarge {
    const BENCH_ID: &'static str = "config_large";
}
