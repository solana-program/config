#![allow(clippy::arithmetic_side_effects)]

mod generated;
mod hooked;

pub use {
    generated::{programs::SOLANA_CONFIG_ID as ID, *},
    hooked::*,
};
