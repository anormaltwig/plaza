#[allow(clippy::module_inception)] // Shut up
mod bureau;
pub use bureau::*;

pub mod math;
pub mod protocol;
pub mod user;
pub mod user_list;
