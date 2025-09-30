//! Config Program.

#[cfg(target_os = "solana")]
mod entrypoint;
pub mod error;
pub mod processor;

solana_program::declare_id!("Config1111111111111111111111111111111111111");
