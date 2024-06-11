//! Config program compute unit benchmark testing.

mod setup;

use {
    crate::setup::{BenchSetup, ConfigLarge, ConfigMedium, ConfigSmall},
    mollusk::Mollusk,
    mollusk_bencher::MolluskComputeUnitBencher,
};

// Taken from `https://github.com/anza-xyz/agave/blob/20549d93420bff03b74c38dd1194ae479ba0824c/programs/config/src/config_processor.rs#L14`.
const DEFAULT_COMPUTE_UNITS: u64 = 450;

fn main() {
    std::env::set_var("SBF_OUT_DIR", "../target/deploy");
    let mollusk = Mollusk::new(&solana_config_program::id(), "solana_config_program");

    MolluskComputeUnitBencher::new(mollusk)
        .benchmark(DEFAULT_COMPUTE_UNITS)
        .bench(ConfigSmall::init(0))
        .bench(ConfigSmall::init(1))
        .bench(ConfigSmall::init(5))
        .bench(ConfigSmall::init(10))
        .bench(ConfigSmall::init(25))
        .bench(ConfigSmall::init(37))
        .bench(ConfigSmall::store(0))
        .bench(ConfigSmall::store(1))
        .bench(ConfigSmall::store(5))
        .bench(ConfigSmall::store(10))
        .bench(ConfigSmall::store(25))
        .bench(ConfigSmall::store(37))
        .bench(ConfigMedium::init(0))
        .bench(ConfigMedium::init(1))
        .bench(ConfigMedium::init(5))
        .bench(ConfigMedium::init(10))
        .bench(ConfigMedium::init(25))
        .bench(ConfigMedium::init(37))
        .bench(ConfigMedium::store(0))
        .bench(ConfigMedium::store(1))
        .bench(ConfigMedium::store(5))
        .bench(ConfigMedium::store(10))
        .bench(ConfigMedium::store(25))
        .bench(ConfigMedium::store(37))
        .bench(ConfigLarge::init(0))
        .bench(ConfigLarge::init(1))
        .bench(ConfigLarge::init(5))
        .bench(ConfigLarge::init(10))
        .bench(ConfigLarge::init(25))
        .bench(ConfigLarge::init(37))
        .bench(ConfigLarge::store(0))
        .bench(ConfigLarge::store(1))
        .bench(ConfigLarge::store(5))
        .bench(ConfigLarge::store(10))
        .bench(ConfigLarge::store(25))
        .bench(ConfigLarge::store(37))
        .iterations(100)
        .must_pass(true)
        .out_dir("../target/benches")
        .execute();
}
