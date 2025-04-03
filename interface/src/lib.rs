#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![allow(clippy::arithmetic_side_effects)]
#[cfg(feature = "bincode")]
pub mod instruction;
pub mod state;
pub use solana_sdk_ids::config::id;
