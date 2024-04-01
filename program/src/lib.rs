//! Config Program.

#[cfg(all(target_os = "solana", feature = "bpf-entrypoint"))]
mod entrypoint;
pub mod instruction;
pub mod processor;
pub mod state;

// [Core BPF]: TODO: Program-test will not overwrite existing built-ins.
// See https://github.com/solana-labs/solana/pull/35233.
// solana_program::declare_id!("Config1111111111111111111111111111111111111");
solana_program::declare_id!("J333MuXPTwcHvibaTNMW32FZj6EHuowCnAHvvP99vnKv");
