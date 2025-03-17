mod generated;
mod hooked;

#[cfg(feature = "serde")]
pub mod instruction_helpers;

pub use {
    generated::{programs::SOLANA_CONFIG_ID as ID, *},
    hooked::*,
};
