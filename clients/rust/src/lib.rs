#![allow(clippy::arithmetic_side_effects)]

mod generated;
mod hooked;
#[deprecated(since = "1.0.0", note = "use `solana_config_interface` crate instead")]
#[cfg(feature = "serde")]
pub mod instructions_bincode;

pub use {
    generated::{programs::SOLANA_CONFIG_ID as ID, *},
    hooked::*,
};
