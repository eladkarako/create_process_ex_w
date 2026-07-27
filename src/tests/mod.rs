// src/tests/mod.rs

// Test suite for the crate.
//
// This module groups together integration-style unit tests that exercise
// the Windows process lifecycle: spawning, status/wait behavior, killing,
// and environment handling.
///
/// These tests are kept in a `tests/` submodule tree (not as separate
/// crate integration tests) and rely on internal implementation details
/// exposed to tests via `crate::binding` and `cfg(test)` items.
mod binding_tests;
mod env_tests;
mod spawn_tests;
mod kill_tests;
mod status_tests;
