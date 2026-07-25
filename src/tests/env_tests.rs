use crate::Command;

#[test]
fn env_var_is_passed_to_child() {
    let child = Command::new(r#"cmd.exe /c "if "%MY_VAR%"=="hello_test" (exit 0) else (exit 1)""#)
        .env("MY_VAR", "hello_test")
        .spawn()
        .unwrap();

    let status = child.wait().unwrap();
    assert_eq!(status.code(), 0, "MY_VAR should be 'hello_test'");
}

#[test]
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
fn env_remove_removes_var() {
    let child = Command::new("cmd.exe /c \"if defined PATH (exit 1) else (exit 0)\"")
        .env_remove("PATH")
        .spawn()
        .unwrap();

    let status = child.wait().unwrap();
    assert_eq!(status.code(), 0);
}

#[test]
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
fn remove_overrides_earlier_env() {
    let child = Command::new(r#"cmd.exe /c "if defined MY_VAR (exit 1) else (exit 0)""#)
        .env("MY_VAR", "some_value")
        .env_remove("MY_VAR")
        .spawn()
        .unwrap();

    let status = child.wait().unwrap();
    assert_eq!(status.code(), 0, "env_remove should override earlier env");
}
