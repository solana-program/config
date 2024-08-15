mod generated;
mod hooked;

pub use {
    generated::{programs::CONFIG_ID as ID, *},
    hooked::*,
};
