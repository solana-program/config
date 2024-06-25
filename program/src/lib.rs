//! Config Program.

#[cfg(all(target_os = "solana", feature = "bpf-entrypoint"))]
mod entrypoint;
pub mod instruction;
pub mod processor;
pub mod state;

solana_program::declare_id!("Config1111111111111111111111111111111111111");
