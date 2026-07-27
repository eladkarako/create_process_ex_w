use crate::Command;

#[test]
/// Ensures that when no environment modifications are provided, the spawned
/// child inherits the parent process environment.
///
/// This is tested using `cmd.exe` with a simple existence check:
/// - The child runs `if defined PATH ...`.
/// - If `PATH` is defined, the script exits with `0`.
/// - If `PATH` is not defined, the script exits with `1`.
///
/// Test flow:
/// 1) Create a `Command` for `cmd.exe` without calling `.env(...)`,
///    `.env_clear()`, or `.env_remove(...)`.
/// 2) Spawn the child.
/// 3) Wait for the child to exit.
/// 4) Assert the exit code is `0`, proving `PATH` is inherited.
fn no_env_args_inherits_parent() {
    let child =
        Command::new(r#"cmd.exe /c "if defined PATH (exit 0) else (exit 1)""#)
            .spawn()
            .unwrap();

    let status = child.wait().unwrap();
    assert_eq!(status.code(), 0);
}
