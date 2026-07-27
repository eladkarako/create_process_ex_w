use crate::Command;

#[test]
/// Ensures that environment variables configured on the `Command` builder are
/// actually visible inside the spawned child process.
///
/// In this test, we:
/// 1) Build a `cmd.exe /c ...` one-liner that checks whether `%MY_VAR%` equals
///    the string `"hello_test"`.
/// 2) Set `MY_VAR` via `.env("MY_VAR", "hello_test")`.
/// 3) Spawn the child and wait for completion.
/// 4) Assert the child exits successfully (exit code `0`).
///
/// This validates the end-to-end contract that `Command::env(...)` results in a
/// working environment variable in the child created via Windows process creation.
fn env_var_is_passed_to_child() {
    let child = Command::new(r#"cmd.exe /c "if "%MY_VAR%"=="hello_test" (exit 0) else (exit 1)""#)
        .env("MY_VAR", "hello_test")
        .spawn()
        .unwrap();

    let status = child.wait().unwrap();
    assert_eq!(status.code(), 0, "MY_VAR should be 'hello_test'");
}

#[test]
/// Verifies that `Command::env_clear()` clears the base environment presented to
/// the child process, *before* applying subsequent overrides.
///
/// Specifically, this test executes a `cmd.exe` script that:
/// - Fails with exit code `1` if `PATH` is defined (`if defined PATH (exit 1)`),
/// - Otherwise checks whether `%CUSTOM%` equals `"value"`,
///   returning `0` if correct, and `2` if not.
///
/// The test flow is:
/// 1) Start with `Command::new("cmd.exe /c ...")`.
/// 2) Call `.env_clear()`
///    - This requests that the child's environment be cleared prior to applying
///      modifications.
/// 3) Add a single variable using `.env("CUSTOM", "value")`.
/// 4) Spawn and wait.
/// 5) Assert exit code `0`, demonstrating:
///    - `PATH` is not present in the child environment, and
///    - `CUSTOM` is present with the expected value.
fn env_clear_with_single_var() {
    let child = Command::new(
        r#"cmd.exe /c "if defined PATH (exit 1) else (if "%CUSTOM%"=="value" (exit 0) else (exit 2))""#,
    )
        .env_clear()
        .env("CUSTOM", "value")
        .spawn()
        .unwrap();

    let status = child.wait().unwrap();
    assert_eq!(
        status.code(),
        0,
        "PATH should be unset and CUSTOM should be 'value'"
    );
}

#[test]
/// Checks that `Command::env_remove(key)` unsets/removes the specified environment
/// variable from the child process environment.
///
/// The child command line for this test is:
/// - `if defined PATH (exit 1) else (exit 0)`
///
/// Meaning:
/// - If `PATH` is present in the child environment, the child exits with `1`.
/// - If `PATH` is absent, the child exits with `0`.
///
/// The test itself:
/// 1) Creates a `Command` for `cmd.exe` with the above conditional.
/// 2) Calls `.env_remove("PATH")` which records an environment modification of
///    `(PATH, None)` to be translated into an unset environment entry during spawn.
/// 3) Spawns and waits.
/// 4) Asserts exit code `0` to confirm the variable is not defined in the child.
fn env_remove_removes_var() {
    let child = Command::new("cmd.exe /c \"if defined PATH (exit 1) else (exit 0)\"")
        .env_remove("PATH")
        .spawn()
        .unwrap();

    let status = child.wait().unwrap();
    assert_eq!(status.code(), 0);
}

#[test]
/// Validates how duplicate keys are handled when the same environment variable
/// is set multiple times via `.env(key, value)`.
///
/// Windows environment variables are effectively a single mapping from a variable
/// name to a value. This crate represents requested modifications as an ordered
/// list (`env_vars`), and applying duplicates must result in a deterministic choice.
///
/// This test checks that:
/// - When `.env("MY_VAR", ...)` is called twice with the same key,
///   the *last* call wins.
///
/// Child behavior:
/// - If `%MY_VAR%` is `"second"`, exit with `2`.
/// - Else if `%MY_VAR%` is `"first"`, exit with `0`.
/// - Else exit with `3`.
///
/// Test flow:
/// 1) Call `.env("MY_VAR", "first")`.
/// 2) Then call `.env("MY_VAR", "second")`.
/// 3) Spawn and wait.
/// 4) Assert the exit code is `2`, proving the `"second"` value is what the child sees.
fn last_duplicate_key_wins() {
    let child = Command::new(
        r#"cmd.exe /c "if "%MY_VAR%"=="second" (exit 2) else (if "%MY_VAR%"=="first" (exit 0) else (exit 3))""#,
    )
        .env("MY_VAR", "first")
        .env("MY_VAR", "second")
        .spawn()
        .unwrap();

    let status = child.wait().unwrap();
    assert_eq!(status.code(), 2, "duplicate key should keep the last value 'second'");
}

#[test]
/// Ensures duplicate environment variable keys are treated case-insensitively on
/// Windows when deciding which duplicate “wins”.
///
/// This test is the case-variation counterpart to `last_duplicate_key_wins`.
/// It checks that:
/// - `.env("MyVar", "first")` and `.env("MYVAR", "second")` refer to the same
///   logical environment variable on Windows.
/// - The last inserted value ("second") is the one visible to the child.
///
/// Child behavior:
/// - If `%MYVAR%` is `"second"`, exit with `2`.
/// - Else if `%MYVAR%` is `"first"`, exit with `0`.
/// - Else exit with `3`.
///
/// The test flow:
/// 1) Set `MyVar` to `"first"`.
/// 2) Set `MYVAR` to `"second"`.
/// 3) Spawn and wait.
/// 4) Assert exit code `2` to confirm the last value wins despite key casing differences.
fn last_duplicate_key_wins_case_insensitive() {
    let child = Command::new(
        r#"cmd.exe /c "if "%MYVAR%"=="second" (exit 2) else (if "%MYVAR%"=="first" (exit 0) else (exit 3))""#,
    )
        .env("MyVar", "first")
        .env("MYVAR", "second")
        .spawn()
        .unwrap();

    let status = child.wait().unwrap();
    assert_eq!(
        status.code(),
        2,
        "case-insensitive duplicate key should keep the last value 'second'"
    );
}

#[test]
/// Verifies ordering between `.env_remove(key)` and a later `.env(key, value)`
/// for the same key.
///
/// This test demonstrates that later configuration calls take precedence.
/// In particular, it checks this ordering rule:
/// - If you remove a key and then later set it, the final value should exist
///   in the child environment.
///
/// Child behavior:
/// - If `%MY_VAR%` equals `"hello_override"`, exit with `0`.
/// - Otherwise exit with `1`.
///
/// Test flow:
/// 1) Record `.env_remove("MY_VAR")` (unset request).
/// 2) Then record `.env("MY_VAR", "hello_override")` (set request).
/// 3) Spawn and wait.
/// 4) Assert exit code `0`, proving the later `.env(...)` overrides the earlier removal.
fn env_overrides_earlier_remove() {
    let child = Command::new(
        r#"cmd.exe /c "if "%MY_VAR%"=="hello_override" (exit 0) else (exit 1)""#,
    )
        .env_remove("MY_VAR")
        .env("MY_VAR", "hello_override")
        .spawn()
        .unwrap();

    let status = child.wait().unwrap();
    assert_eq!(status.code(), 0, "env should override earlier env_remove");
}

#[test]
/// Verifies ordering between an earlier `.env(key, value)` and a later
/// `.env_remove(key)` for the same key.
///
/// This test checks the complementary precedence rule to `env_overrides_earlier_remove`:
/// - If you set a key and then later remove it, the key should be absent in the
///   child environment.
///
/// Child behavior:
/// - If `MY_VAR` is defined, exit with `1`.
/// - Else exit with `0`.
///
/// Test flow:
/// 1) Record `.env("MY_VAR", "some_value")` (set request).
/// 2) Then record `.env_remove("MY_VAR")` (unset request).
/// 3) Spawn and wait.
/// 4) Assert exit code `0`, proving the later removal wins.
fn remove_overrides_earlier_env() {
    let child = Command::new(r#"cmd.exe /c "if defined MY_VAR (exit 1) else (exit 0)""#)
        .env("MY_VAR", "some_value")
        .env_remove("MY_VAR")
        .spawn()
        .unwrap();

    let status = child.wait().unwrap();
    assert_eq!(status.code(), 0, "env_remove should override earlier env");
}
