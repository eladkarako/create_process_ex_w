#![allow(non_snake_case)]

// Internal modules that implement the crate.
//
// `binding` contains low-level Windows FFI declarations and shared constants/types.
// The other modules build a safe-ish Rust API on top of those bindings.
mod binding;

mod command;
mod child;
mod env_block;
mod exit_status;

// Re-export the public types so users can access them directly from the crate root.
pub use child::Child;
pub use command::Command;
pub use exit_status::ExitStatus;

#[cfg(test)]
mod tests;
