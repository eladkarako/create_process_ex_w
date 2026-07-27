use crate::Command;

#[test]
/// Verifies that `Child::try_wait()` eventually returns `Some(ExitStatus)` for a
/// process that terminates quickly.
///
/// This test uses a child process that exits immediately:
/// - `cmd.exe /c exit 0` terminates almost right away.
///
/// Test behavior:
/// 1) Spawn the child.
/// 2) Repeatedly call `try_wait()` up to 200 times.
/// 3) As soon as `try_wait()` returns `Some(status)`, stop polling.
/// 4) Assert that:
///    - we observed `Some(ExitStatus)`, and
///    - the exit code is `0`.
fn status_try_wait_eventually_returns_some() {
    // Quick exit so try_wait should become Some(exit_status) quickly.
    let child = Command::new("cmd.exe /c exit 0").spawn().unwrap();

    let mut got = None;
    for _ in 0..200 {
        got = child.try_wait().unwrap();
        if got.is_some() {
            break;
        }
    }

    let status = got.expect("expected try_wait to eventually return Some(exit_status)");
    assert_eq!(status.code(), 0);
}

#[test]
/// Verifies that `Child::wait()` blocks until the process exits and returns
/// an `ExitStatus` that reports success.
///
/// This test spawns a process that deterministically exits with code `0`:
/// - `cmd.exe /c exit 0`
///
/// Test behavior:
/// 1) Spawn the child.
/// 2) Call `wait()` once, capturing the returned `ExitStatus`.
/// 3) Assert that `status.success()` is true (i.e., exit code indicates success).
fn status_wait_returns_exit_code() {
    let child = Command::new("cmd.exe /c exit 0").spawn().unwrap();
    let status = child.wait().unwrap();
    assert!(status.success());
}
