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
fn default_spawn_does_not_give_inheritable_handles() {
    let child = Command::new("cmd.exe /c exit 0").spawn().unwrap();
    let hproc = child.process_information.hProcess;
    let hthread = child.process_information.hThread;

    let proc_inheritable = unsafe { is_inheritable(hproc) };
    let thread_inheritable = unsafe { is_inheritable(hthread) };
    child.wait().unwrap();
    assert!(!proc_inheritable, "process handle should NOT be inheritable by default");
    assert!(
        !thread_inheritable,
        "thread handle should NOT be inheritable by default"
    );
}

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
