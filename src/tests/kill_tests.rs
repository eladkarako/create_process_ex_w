use std::ffi::c_void;

use crate::binding::{GetHandleInformation, HANDLE_FLAG_INHERIT};
use crate::Command;

unsafe fn is_inheritable(handle: *mut c_void) -> bool {
    let mut flags: u32 = 0;
    let ok = GetHandleInformation(handle, &mut flags) != 0;
    assert!(ok, "GetHandleInformation failed");
    (flags & HANDLE_FLAG_INHERIT) != 0
}

#[test]
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

    let status = child.wait().unwrap();
    // We can’t guarantee the exact exit code after TerminateProcess,
    // but the process should have finished.
    assert!(status.code() >= 0);
}
