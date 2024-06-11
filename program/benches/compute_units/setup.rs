use {
    mollusk_bencher::Bench,
    serde::{Deserialize, Serialize},
    solana_config_program::{
        instruction::store,
        state::{ConfigKeys, ConfigState},
    },
    solana_sdk::{
        account::AccountSharedData,
        hash::Hash,
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        rent::Rent,
    },
};

pub trait BenchSetup: ConfigState + Default {
    const BENCH_ID: &'static str;

    fn default_account_state(keys: Vec<(Pubkey, bool)>) -> (ConfigKeys, Self) {
        (ConfigKeys { keys }, Self::default())
    }

    fn default_space(keys: Vec<(Pubkey, bool)>) -> usize {
        (Self::max_space() + ConfigKeys::serialized_size(keys)) as usize
    }

    fn keys(keys_len: usize) -> Vec<(Pubkey, bool)> {
        (0..keys_len)
            .map(|_| (Pubkey::new_unique(), false))
            .collect()
    }

    fn test_store_value() -> Self;

    fn init(keys_len: usize) -> Bench {
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

        (
            format!("{}_init_{}_keys", Self::BENCH_ID, keys_len),
            instruction,
            accounts,
        )
    }

    fn store(keys_len: usize) -> Bench {
        let config_pubkey = Pubkey::new_unique();
        let keys = Self::keys(keys_len);
        let space = Self::default_space(keys.clone());
        let lamports = Rent::default().minimum_balance(space);

        let instruction = store(
            &config_pubkey,
            true,
            keys.clone(),
            &Self::test_store_value(),
        );

        let accounts = vec![(
            config_pubkey,
            AccountSharedData::new_data(
                lamports,
                &Self::default_account_state(keys),
                &solana_config_program::id(),
            )
            .unwrap(),
        )];

        (
            format!("small_store_{}_keys", keys_len),
            instruction,
            accounts,
        )
    }
}

#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
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

    fn test_store_value() -> Self {
        Self { item: 42 }
    }
}

#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct ConfigMedium {
    pub hashes: [Hash; 8], // 8 x 32 = 256 bytes
    pub rent: Rent,
}

impl ConfigState for ConfigMedium {
    fn max_space() -> u64 {
        bincode::serialized_size(&Self::default()).unwrap()
    }
}

impl BenchSetup for ConfigMedium {
    const BENCH_ID: &'static str = "config_medium";

    fn test_store_value() -> Self {
        Self {
            hashes: [[1; 32].into(); 8],
            rent: Rent::default(),
        }
    }
}

#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct ConfigLarge {
    pub hashes: [Hash; 32], // 32 x 32 = 1024 bytes
    pub rent: Rent,
}

impl ConfigState for ConfigLarge {
    fn max_space() -> u64 {
        bincode::serialized_size(&Self::default()).unwrap()
    }
}

impl BenchSetup for ConfigLarge {
    const BENCH_ID: &'static str = "config_large";

    fn test_store_value() -> Self {
        Self {
            hashes: [[1; 32].into(); 32],
            rent: Rent::default(),
        }
    }
}
