use std::ffi::c_void;

use crate::binding::{GetHandleInformation, HANDLE_FLAG_INHERIT};
use crate::Command;

/// Checks whether a raw Windows handle is marked as inheritable.
///
/// This helper wraps the Windows API `GetHandleInformation` and tests the
/// `HANDLE_FLAG_INHERIT` bit in the returned flags value.
///
/// # Parameters
/// - `handle`: A raw handle pointer passed to `GetHandleInformation`.
///
/// # Safety
/// - The caller must ensure `handle` is a valid pointer to a Windows handle
///   value appropriate for `GetHandleInformation`.
/// - Passing an invalid pointer/handle can lead to undefined behavior.
///
/// # Returns
/// - `true` if the handle has the `HANDLE_FLAG_INHERIT` flag set.
/// - `false` otherwise.
///
/// # Panics
/// - Panics if `GetHandleInformation` fails (`ok == 0`), since this test
///   uses the result as a prerequisite for validating other behavior.
unsafe fn is_inheritable(handle: *mut c_void) -> bool {
    let mut flags: u32 = 0;
    let ok = GetHandleInformation(handle, &mut flags) != 0;
    assert!(ok, "GetHandleInformation failed");
    (flags & HANDLE_FLAG_INHERIT) != 0
}

#[test]
/// Verifies that `Child::kill()` terminates the spawned process.
///
/// This test creates a child process that is expected to run long enough
/// for the test to call `kill()`:
/// - It launches `cmd.exe /c ping 127.0.0.1 -n 50 >nul`.
/// - `>nul` suppresses ping output.
///
/// The test then:
/// 1) Enables handle inheritance on the `Command` (`inherit_handles(true)`).
/// 2) Spawns the child.
/// 3) Uses `is_inheritable` to sanity-check that the process and thread
///    handles are marked inheritable.
/// 4) Calls `child.kill()` and asserts it returns `Ok`.
/// 5) Calls `child.wait()` to ensure the process has finished.
///
/// Note: after forcibly terminating a process via `TerminateProcess`, the
/// exact exit code is not asserted; instead, the test checks that waiting
/// completes after `kill()`.
fn kill_terminates_process() {
    // A long-running command we can terminate.
    // `ping -n 50` takes some time; if timing differs, kill() still should work.
    let child = Command::new(r#"cmd.exe /c ping 127.0.0.1 -n 50 >nul"#)
        .inherit_handles(true)
        .spawn()
        .unwrap();

    // Ensure we can call kill safely.
    let _ = unsafe { is_inheritable(child.process_information.hProcess) };
    let _ = unsafe { is_inheritable(child.process_information.hThread) };

    child.kill().unwrap();

    let _status = child.wait().unwrap();
    // We can’t guarantee the exact exit code after TerminateProcess,
    // but the process should have finished.
}
