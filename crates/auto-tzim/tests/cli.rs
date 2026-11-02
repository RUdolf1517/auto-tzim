use std::process::Command;

#[test]
fn version_and_unknown_argument() {
    let executable = env!("CARGO_BIN_EXE_auto-tzim");
    let version = Command::new(executable).arg("--version").output().unwrap();
    assert!(version.status.success());
    assert_eq!(
        String::from_utf8(version.stdout).unwrap(),
        format!("auto-tzim {}\n", env!("CARGO_PKG_VERSION"))
    );
    assert!(version.stderr.is_empty());

    let unknown = Command::new(executable).arg("unknown").output().unwrap();
    assert_eq!(unknown.status.code(), Some(2));
    assert!(unknown.stdout.is_empty());
    assert!(!unknown.stderr.is_empty());
}
