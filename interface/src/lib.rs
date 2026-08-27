#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(clippy::arithmetic_side_effects)]
#[cfg(any(feature = "bincode", feature = "wincode"))]
pub mod instruction;
pub mod state;
pub use solana_sdk_ids::config::id;
