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

#[cfg(unix)]
#[test]
fn permissions_are_private_under_umask_zero() {
    use std::os::unix::fs::PermissionsExt;
    let s = Sandbox::new();
    let folder = s.folder("photos");
    let out = Command::new("sh")
        .args([
            "-c",
            "umask 000; exec \"$1\" folder add \"$2\"",
            "test",
            env!("CARGO_BIN_EXE_auto-tzim"),
        ])
        .arg(&folder)
        .env("HOME", s.root.join("home"))
        .env("AUTO_TZIM_HOME", s.root.join("config"))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        fs::metadata(s.config()).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert_eq!(
        fs::metadata(s.root.join("config"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o700
    );
}

#[test]
fn names_are_case_insensitive_and_cloud_roots_rejected() {
    let s = Sandbox::new();
    let first = s.folder("photos");
    let second = s.folder("other");
    assert!(s
        .run(&["folder", "add", first.to_str().unwrap()])
        .status
        .success());
    assert!(s.run(&["folder", "show", "PHOTOS"]).status.success());
    let out = s.run(&[
        "folder",
        "add",
        second.to_str().unwrap(),
        "--name",
        "PHOTOS",
    ]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("duplicate_name"));
    let cloud = s.folder("home/Dropbox/images");
    let out = s.run(&["folder", "add", cloud.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("cloud_sync_root"));
}

#[test]
fn edit_keeps_other_profile_bytes_and_supports_rename() {
    let s = Sandbox::new();
    let a = s.folder("alpha");
    let b = s.folder("beta");
    assert!(s
        .run(&["folder", "add", a.to_str().unwrap(), b.to_str().unwrap()])
        .status
        .success());
    let before = fs::read_to_string(s.config()).unwrap();
    let beta = before.split("[[folder]]").nth(2).unwrap().to_owned();
    let out = s.run(&["folder", "edit", "alpha", "--name", "changed"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let after = fs::read_to_string(s.config()).unwrap();
    assert_eq!(after.split("[[folder]]").nth(2).unwrap(), beta);
    assert!(s.run(&["folder", "show", "changed"]).status.success());
    assert_eq!(s.run(&["folder", "show", "alpha"]).status.code(), Some(2));
}

#[cfg(unix)]
#[test]
fn root_symlinks_are_not_laundered_by_a_trailing_slash() {
    use std::os::unix::fs::symlink;
    let s = Sandbox::new();
    let real = s.folder("real");
    let link = s.root.join("link");
    symlink(real, &link).unwrap();
    let root = format!("{}/", link.display());
    let out = s.run(&["folder", "add", &root]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("root_is_symlink"));
}

#[test]
fn configuration_refusal_matrix() {
    use std::os::unix::fs::symlink;
    let s = Sandbox::new();
    let file = s.root.join("file");
    fs::write(&file, "data").unwrap();
    let actual = s.folder("real");
    let link = s.root.join("link");
    symlink(&actual, &link).unwrap();
    let state_child = s.folder("config/inside");
    let home = s.root.join("home");
    for (value, reason) in [
        ("".to_string(), "root_not_absolute"),
        (".".to_string(), "root_not_absolute"),
        ("relative".to_string(), "root_not_absolute"),
        ("/".to_string(), "forbidden_root"),
        (home.to_string_lossy().into_owned(), "forbidden_root"),
        (file.to_string_lossy().into_owned(), "root_not_a_directory"),
        (link.to_string_lossy().into_owned(), "root_is_symlink"),
        (
            state_child.to_string_lossy().into_owned(),
            "state_inside_root",
        ),
        (s.root.to_string_lossy().into_owned(), "state_inside_root"),
        (
            s.root.join("missing").to_string_lossy().into_owned(),
            "root_missing",
        ),
    ] {
        let output = s.run(&["folder", "add", &value]);
        assert_eq!(output.status.code(), Some(2), "{value}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(reason),
            "{value}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!s.config().exists());
    }
}

#[test]
fn reserved_values_are_rejected_and_inert_settings_accepted() {
    let s = Sandbox::new();
    let folder = s.folder("photos");
    fs::create_dir_all(s.root.join("config")).unwrap();
    let base = format!("schema = 1\n[installation]\nupdate_policy = 'notify'\nallowed_major_versions = [0]\n[[folder]]\nname = 'photos'\nroot = {:?}\n", folder.to_str().unwrap());
    for (field, reason) in [
        ("follow_symlinks = true\n", "not_in_v01"),
        ("allow_hard_links = true\n", "not_in_v01"),
        ("jobs = 2\n", "not_in_v01"),
        ("[folder.schedule]\nenabled = true\n", "not_in_v01"),
        ("transforms = ['fits-tile']\n", "transforms_forbidden"),
    ] {
        fs::write(s.config(), format!("{base}{field}")).unwrap();
        let out = s.run(&["config", "validate"]);
        assert_eq!(out.status.code(), Some(2), "{field}");
        assert!(String::from_utf8_lossy(&out.stderr).contains(reason));
    }
    fs::write(s.config(), format!("{base}transforms = []\n")).unwrap();
    assert!(s.run(&["config", "validate"]).status.success());
}

#[test]
fn killed_edits_never_leave_a_partial_config() {
    use std::{thread, time::Duration};
    let s = Sandbox::new();
    let folder = s.folder("photos");
    assert!(s
        .run(&["folder", "add", folder.to_str().unwrap()])
        .status
        .success());
    let old = fs::read(s.config()).unwrap();
    assert!(s.run(&["folder", "disable", "photos"]).status.success());
    let new = fs::read(s.config()).unwrap();
    assert_ne!(old, new);
    for i in 0..20 {
        fs::write(s.config(), &old).unwrap();
        let mut child = Command::new(env!("CARGO_BIN_EXE_auto-tzim"))
            .env("HOME", s.root.join("home"))
            .env("AUTO_TZIM_HOME", s.root.join("config"))
            .args(["folder", "disable", "photos"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap();
        thread::sleep(Duration::from_micros(((i * 541) % 3000) as u64));
        let _ = child.kill();
        let _ = child.wait();
        let actual = fs::read(s.config()).unwrap();
        assert!(
            actual == old || actual == new,
            "unexpected truncated config"
        );
        assert!(s.run(&["config", "validate"]).status.success());
    }
}
