#![allow(non_snake_case)]
#![deny(missing_docs)]

mod binding;

mod command;
mod child;
mod env_block;
mod exit_status;

pub use command::Command;
pub use child::Child;
pub use exit_status::ExitStatus;

#[cfg(test)]
mod tests;
