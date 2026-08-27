//! Program instruction helpers.
//!
//! The helpers are implemented once for `bincode` and once for `wincode`,
//! since each constrains the config state type differently. `wincode` takes
//! precedence when both features are enabled.

#[cfg(all(not(feature = "wincode"), feature = "bincode"))]
mod bincode;
#[cfg(feature = "wincode")]
mod wincode;

#[cfg(all(not(feature = "wincode"), feature = "bincode"))]
pub use bincode::*;
#[cfg(feature = "wincode")]
pub use wincode::*;
