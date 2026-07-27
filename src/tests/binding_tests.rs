// src/tests/binding_tests.rs

use std::ffi::c_void;

use crate::binding::{GetHandleInformation, HANDLE_FLAG_INHERIT};

use crate::Command;

/// Helper: queries Windows handle information and checks whether the handle is inheritable.
///
/// # Safety
/// - `handle` must be a valid Windows handle pointer passed to `GetHandleInformation`.
/// - The handle must remain valid for the duration of this call.
unsafe fn is_inheritable(handle: *mut c_void) -> bool {
    let mut flags: u32 = 0;
    let ok = GetHandleInformation(handle, &mut flags) != 0;
    assert!(ok, "GetHandleInformation failed");
    (flags & HANDLE_FLAG_INHERIT) != 0
}

/// Verifies that `Command` defaults to not making handles inheritable in the child.
///
/// This test:
/// - spawns `cmd.exe /c exit 0` with default settings (no `.inherit_handles(true)`),
/// - queries handle information for both the process handle and the primary thread handle,
/// - waits for the process to exit,
/// - asserts that neither handle is marked inheritable.
///
/// This behavior is expected because the crate passes `lpProcessAttributes` and
/// `lpThreadAttributes` as `NULL` when `inherit_handles` is `false`.
#[test]
fn default_spawn_does_not_give_inheritable_handles() {
    let child = Command::new("cmd.exe /c exit 0").spawn().unwrap();
    let hproc = child.process_information.hProcess;
    let hthread = child.process_information.hThread;

    let proc_inheritable = unsafe { is_inheritable(hproc) };
    let thread_inheritable = unsafe { is_inheritable(hthread) };

    // Ensure the process has finished before the test ends.
    child.wait().unwrap();

    assert!(!proc_inheritable, "process handle should NOT be inheritable by default");
    assert!(
        !thread_inheritable,
        "thread handle should NOT be inheritable by default"
    );
}

/// Verifies that enabling handle inheritance makes both handles inheritable.
///
/// This test:
/// - spawns `cmd.exe /c exit 0` with `.inherit_handles(true)`,
/// - checks the inheritable flag on both the process and thread handles,
/// - waits for the process to exit,
/// - asserts both are inheritable.
///
/// The expected behavior is driven by passing a `SECURITY_ATTRIBUTES` with
/// `bInheritHandle = TRUE` into both the process- and thread-attributes parameters.
#[test]
fn inherit_handles_true_gives_inheritable_handles() {
    let child = Command::new("cmd.exe /c exit 0")
        .inherit_handles(true)
        .spawn()
        .unwrap();

    let hproc = child.process_information.hProcess;
    let hthread = child.process_information.hThread;

    let proc_inheritable = unsafe { is_inheritable(hproc) };
    let thread_inheritable = unsafe { is_inheritable(hthread) };

    // Ensure the process has finished before assertions/exit.
    child.wait().unwrap();

    assert!(
        proc_inheritable,
        "process handle should be inheritable when inherit_handles(true)"
    );
    assert!(
        thread_inheritable,
        "thread handle should be inheritable when inherit_handles(true)"
    );
}
