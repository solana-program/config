//! Compute unit benchmark testing.

mod setup;

use {
    crate::setup::{BenchSetup, ConfigLarge, ConfigMedium, ConfigSmall},
    mollusk_svm::Mollusk,
    mollusk_svm_bencher::MolluskComputeUnitBencher,
};

fn main() {
    std::env::set_var("SBF_OUT_DIR", "../target/deploy");
    let mollusk = Mollusk::new(&solana_config_program::id(), "solana_config_program");

    MolluskComputeUnitBencher::new(mollusk)
        .bench(ConfigSmall::init(0).bench())
        .bench(ConfigSmall::init(1).bench())
        .bench(ConfigSmall::init(5).bench())
        .bench(ConfigSmall::init(10).bench())
        .bench(ConfigSmall::init(25).bench())
        .bench(ConfigSmall::init(37).bench())
        .bench(ConfigSmall::store(0).bench())
        .bench(ConfigSmall::store(1).bench())
        .bench(ConfigSmall::store(5).bench())
        .bench(ConfigSmall::store(10).bench())
        .bench(ConfigSmall::store(25).bench())
        .bench(ConfigSmall::store(37).bench())
        .bench(ConfigMedium::init(0).bench())
        .bench(ConfigMedium::init(1).bench())
        .bench(ConfigMedium::init(5).bench())
        .bench(ConfigMedium::init(10).bench())
        .bench(ConfigMedium::init(25).bench())
        .bench(ConfigMedium::init(37).bench())
        .bench(ConfigMedium::store(0).bench())
        .bench(ConfigMedium::store(1).bench())
        .bench(ConfigMedium::store(5).bench())
        .bench(ConfigMedium::store(10).bench())
        .bench(ConfigMedium::store(25).bench())
        .bench(ConfigMedium::store(37).bench())
        .bench(ConfigLarge::init(0).bench())
        .bench(ConfigLarge::init(1).bench())
        .bench(ConfigLarge::init(5).bench())
        .bench(ConfigLarge::init(10).bench())
        .bench(ConfigLarge::init(25).bench())
        .bench(ConfigLarge::init(37).bench())
        .bench(ConfigLarge::store(0).bench())
        .bench(ConfigLarge::store(1).bench())
        .bench(ConfigLarge::store(5).bench())
        .bench(ConfigLarge::store(10).bench())
        .bench(ConfigLarge::store(25).bench())
        .bench(ConfigLarge::store(37).bench())
        .must_pass(true)
        .out_dir("./benches")
        .execute();
}
