use crate::Command;

#[test]
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
fn status_wait_returns_exit_code() {
    let child = Command::new("cmd.exe /c exit 0").spawn().unwrap();
    let status = child.wait().unwrap();
    assert!(status.success());
}
