use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

struct Sandbox {
    root: PathBuf,
}
impl Sandbox {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "auto-tzim-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(root.join("home")).unwrap();
        Self { root }
    }
    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_auto-tzim"))
            .env("HOME", self.root.join("home"))
            .env("AUTO_TZIM_HOME", self.root.join("config"))
            .args(args)
            .output()
            .unwrap()
    }
    fn config(&self) -> PathBuf {
        self.root.join("config/config.toml")
    }
    fn folder(&self, name: &str) -> PathBuf {
        let path = self.root.join(name);
        fs::create_dir_all(&path).unwrap();
        path
    }
}
impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn three_errors_are_reported_and_suggested() {
    let s = Sandbox::new();
    fs::create_dir_all(s.root.join("config")).unwrap();
    fs::write(s.config(), "schema=1\n[installation]\n[[folder]]\nname='x'\nroot='/x'\nrecurse=true\nenabled=1\njobs=2\n").unwrap();
    let out = s.run(&["config", "validate"]);
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8(out.stderr).unwrap();
    for text in ["folder[0].recurse", "recursive", "enabled", "jobs"] {
        assert!(err.contains(text), "{err}");
    }
}

#[test]
fn rejected_edit_does_not_write_and_corrupt_is_preserved() {
    let s = Sandbox::new();
    let folder = s.folder("photos");
    assert!(s
        .run(&["folder", "add", folder.to_str().unwrap()])
        .status
        .success());
    let before = fs::read(s.config()).unwrap();
    let out = s.run(&["folder", "edit", "photos", "--name", ""]);
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(fs::read(s.config()).unwrap(), before);
    fs::write(s.config(), "invalid [[[\n").unwrap();
    let before = fs::read(s.config()).unwrap();
    let out = s.run(&["folder", "remove", "photos"]);
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(fs::read(s.config()).unwrap(), before);
}

#[test]
fn containment_and_atomic_batch() {
    let s = Sandbox::new();
    let folder = s.folder("photos");
    let inside = s.folder("config/inside");
    let result = s.run(&[
        "folder",
        "add",
        folder.to_str().unwrap(),
        inside.to_str().unwrap(),
    ]);
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("state_inside_root"));
    assert!(!s.config().exists());
}

#[test]
fn remove_never_touches_user_files() {
    let s = Sandbox::new();
    let folder = s.folder("photos");
    fs::write(folder.join("image"), b"user content").unwrap();
    assert!(s
        .run(&["folder", "add", folder.to_str().unwrap()])
        .status
        .success());
    assert!(s.run(&["folder", "remove", "photos"]).status.success());
    assert_eq!(fs::read(folder.join("image")).unwrap(), b"user content");
}
