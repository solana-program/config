#![allow(clippy::arithmetic_side_effects)]

mod generated;
mod hooked;

#[cfg(feature = "serde")]
pub mod instructions_bincode;

pub use {
    generated::{programs::SOLANA_CONFIG_ID as ID, *},
    hooked::*,
};
