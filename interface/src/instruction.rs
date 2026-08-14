//! Program instruction helpers.
//!
//! The helpers are implemented once per serialization backend, since each
//! backend constrains the config state type differently. `wincode` takes
//! precedence when both features are enabled.

#[cfg(all(not(feature = "wincode"), feature = "bincode"))]
mod bincode;
#[cfg(feature = "wincode")]
mod wincode;

#[cfg(all(not(feature = "wincode"), feature = "bincode"))]
pub use bincode::*;
#[cfg(feature = "wincode")]
pub use wincode::*;
