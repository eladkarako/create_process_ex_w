use crate::Command;

#[test]
fn no_env_args_inherits_parent() {
    let child =
        Command::new(r#"cmd.exe /c "if defined PATH (exit 0) else (exit 1)""#)
            .spawn()
            .unwrap();

    let status = child.wait().unwrap();
    assert_eq!(status.code(), 0);
}
