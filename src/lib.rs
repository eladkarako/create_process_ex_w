#![allow(non_snake_case)]

mod binding;

mod command;
mod child;
mod env_block;
mod exit_status;

pub use child::Child;
pub use command::Command;
pub use exit_status::ExitStatus;

#[cfg(test)]
mod tests;
