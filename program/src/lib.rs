//! Config Program.

#[cfg(all(target_os = "solana", feature = "bpf-entrypoint"))]
mod entrypoint;
pub mod error;
pub mod processor;

solana_program::declare_id!("Config1111111111111111111111111111111111111");
