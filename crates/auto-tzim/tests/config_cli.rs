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
    let folder = s.folder("valid");
    fs::write(
        s.config(),
        format!(
            "schema=1\n[installation]\n[[folder]]\nname='x'\nroot={:?}\njobs=2\n",
            folder.to_str().unwrap()
        ),
    )
    .unwrap();
    let out = s.run(&["config", "validate"]);
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8(out.stderr).unwrap();
    assert_eq!(err.lines().count(), 1, "{err}");
    assert!(err.contains("jobs"));
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

// Compare complete trees (including file bytes), not just their immediate children.
fn tree_snapshot(root: &std::path::Path) -> Vec<(PathBuf, Vec<u8>)> {
    fn walk(root: &std::path::Path, current: &std::path::Path, rows: &mut Vec<(PathBuf, Vec<u8>)>) {
        for entry in fs::read_dir(current).unwrap() {
            let path = entry.unwrap().path();
            let relative = path.strip_prefix(root).unwrap().to_path_buf();
            let kind = fs::symlink_metadata(&path).unwrap().file_type();
            if kind.is_dir() {
                rows.push((relative, b"directory".to_vec()));
                walk(root, &path, rows);
            } else if kind.is_file() {
                rows.push((relative, fs::read(path).unwrap()));
            } else {
                panic!("unexpected entry in isolation test: {}", path.display());
            }
        }
    }
    let mut rows = Vec::new();
    walk(root, root, &mut rows);
    rows.sort();
    rows
}

#[test]
fn override_keeps_home_and_xdg_untouched() {
    let s = Sandbox::new();
    let folder = s.folder("photos");
    let xdg = s.root.join("xdg");
    let state = s.root.join("xdg-state");
    fs::create_dir_all(&xdg).unwrap();
    fs::create_dir_all(&state).unwrap();
    let home = s.root.join("home");
    for root in [&home, &xdg, &state] {
        fs::create_dir_all(root.join("sentinel/nested")).unwrap();
        fs::write(root.join("sentinel/nested/keep"), b"unchanged").unwrap();
    }
    let before_home = tree_snapshot(&home);
    let before_xdg = tree_snapshot(&xdg);
    let before_state = tree_snapshot(&state);
    let out = Command::new(env!("CARGO_BIN_EXE_auto-tzim"))
        .env("HOME", s.root.join("home"))
        .env("XDG_CONFIG_HOME", &xdg)
        .env("XDG_STATE_HOME", &state)
        .env("AUTO_TZIM_HOME", s.root.join("config"))
        .arg("folder")
        .arg("add")
        .arg(folder)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(before_home, tree_snapshot(&home));
    assert_eq!(before_xdg, tree_snapshot(&xdg));
    assert_eq!(before_state, tree_snapshot(&state));
    assert!(s.config().exists());
}

#[test]
fn add_to_explicit_empty_profile_array() {
    let s = Sandbox::new();
    let folder = s.folder("photos");
    fs::create_dir_all(s.root.join("config")).unwrap();
    fs::write(
        s.config(),
        "schema = 1\nfolder = [] # keep this comment\n[installation]\n",
    )
    .unwrap();
    let out = s.run(&["folder", "add", folder.to_str().unwrap()]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(s.run(&["config", "validate"]).status.success());
    assert!(fs::read_to_string(s.config())
        .unwrap()
        .contains("# keep this comment"));
}

#[test]
fn edit_preserves_multiline_string_in_another_profile() {
    let sandbox = Sandbox::new();
    let first = sandbox.folder("first");
    let second = sandbox.folder("second");
    fs::create_dir_all(sandbox.root.join("config")).unwrap();
    let original = format!(
        "schema = 1\n[installation]\n\n[[folder]]\nname = 'first'\nroot = {:?}\nexclude_globs = [\"\"\"line one\n[[folder]]\nline three\"\"\"]\n# retain this comment\n\n[[folder]]\nname = 'second'\nroot = {:?}\n",
        first.to_str().unwrap(),
        second.to_str().unwrap()
    );
    fs::write(sandbox.config(), &original).unwrap();
    let result = sandbox.run(&["folder", "edit", "second", "--name", "renamed"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let actual = fs::read_to_string(sandbox.config()).unwrap();
    assert!(
        actual.contains(
            "exclude_globs = [\"\"\"line one\n[[folder]]\nline three\"\"\"]\n# retain this comment"
        ),
        "{actual}"
    );
    assert!(sandbox.run(&["config", "validate"]).status.success());
}

#[test]
fn edit_does_not_destroy_installation_after_folder() {
    let sandbox = Sandbox::new();
    let folder = sandbox.folder("photos");
    fs::create_dir_all(sandbox.root.join("config")).unwrap();
    let original = format!(
        "schema = 1\n[[folder]]\nname = 'photos'\nroot = {:?}\n[installation]\nupdate_policy = 'notify'\n",
        folder.to_str().unwrap()
    );
    fs::write(sandbox.config(), &original).unwrap();
    let result = sandbox.run(&["folder", "edit", "photos", "--name", "renamed"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let text = fs::read_to_string(sandbox.config()).unwrap();
    assert!(text.contains("[installation]\nupdate_policy = 'notify'\n"));
    assert!(sandbox.run(&["config", "validate"]).status.success());
}

#[test]
fn inline_folder_array_supports_edit_remove_and_add() {
    let sandbox = Sandbox::new();
    let first = sandbox.folder("first");
    let second = sandbox.folder("second");
    let third = sandbox.folder("third");
    fs::create_dir_all(sandbox.root.join("config")).unwrap();
    fs::write(sandbox.config(), format!("schema = 1\nfolder = [{{ name = 'first', root = {:?} }}, {{ name = 'second', root = {:?} }}]\n[installation]\n", first.to_str().unwrap(), second.to_str().unwrap())).unwrap();
    for args in [
        vec!["folder", "edit", "first", "--name", "renamed"],
        vec!["folder", "remove", "second"],
        vec!["folder", "add", third.to_str().unwrap()],
    ] {
        let result = sandbox.run(&args);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(sandbox.run(&["config", "validate"]).status.success());
    }
    assert!(sandbox.run(&["folder", "show", "renamed"]).status.success());
    assert!(sandbox.run(&["folder", "show", "third"]).status.success());
    assert_eq!(
        sandbox.run(&["folder", "show", "second"]).status.code(),
        Some(2)
    );
}

#[test]
fn stale_temporary_file_is_never_loaded_as_configuration() {
    let sandbox = Sandbox::new();
    let folder = sandbox.folder("photos");
    fs::create_dir_all(sandbox.root.join("config")).unwrap();
    fs::write(
        sandbox.root.join("config/.config.toml.1.0.tmp"),
        b"corrupt temporary data",
    )
    .unwrap();
    let result = sandbox.run(&["folder", "add", folder.to_str().unwrap()]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(sandbox.run(&["config", "validate"]).status.success());
    assert!(sandbox.run(&["folder", "show", "photos"]).status.success());
}

#[test]
fn quoted_toml_headers_are_preserved_during_edit() {
    let sandbox = Sandbox::new();
    let folder = sandbox.folder("photos");
    fs::create_dir_all(sandbox.root.join("config")).unwrap();
    let original = format!(
        "schema = 1\n[[\"fol\\u0064er\"]] # folder\nname = 'photos'\nroot = {:?}\n[\"installation\"] # installation\nupdate_policy = 'notify'\n",
        folder.to_str().unwrap()
    );
    fs::write(sandbox.config(), &original).unwrap();
    let result = sandbox.run(&["folder", "edit", "photos", "--name", "renamed"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let result = fs::read_to_string(sandbox.config()).unwrap();
    assert!(result.contains("[\"installation\"] # installation\nupdate_policy = 'notify'"));
    assert!(sandbox.run(&["config", "validate"]).status.success());
}

#[test]
fn non_utf8_directory_name_does_not_panic() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    let sandbox = Sandbox::new();
    let path = sandbox
        .root
        .join(OsString::from_vec(b"non-utf8-\xff".to_vec()));
    fs::create_dir(&path).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_auto-tzim"))
        .env("HOME", sandbox.root.join("home"))
        .env("AUTO_TZIM_HOME", sandbox.root.join("config"))
        .args(["folder", "add"])
        .arg(&path)
        .args(["--name", "safe-name"])
        .output()
        .unwrap();
    assert!(!String::from_utf8_lossy(&output.stderr).contains("panicked"));
    assert!(output.status.success() || output.status.code() == Some(2));
    if !output.status.success() {
        assert!(!sandbox.config().exists());
    }
}

#[test]
fn lock_holder_process() {
    let Some(root) = std::env::var_os("AT02_LOCK_HOLDER") else {
        return;
    };
    let root = PathBuf::from(root);
    let _guard =
        auto_tzim_core::config::acquire_lock_file(&root.join("locks/config.lock")).unwrap();
    fs::write(root.join("holder-ready"), std::process::id().to_string()).unwrap();
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn wait_for_file(path: &std::path::Path) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    while !path.exists() {
        assert!(
            std::time::Instant::now() < deadline,
            "timed out waiting for {}",
            path.display()
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

#[test]
fn lock_contention_reports_pid_and_sigkill_releases_kernel_lock() {
    let sandbox = Sandbox::new();
    let base = sandbox.root.join("config");
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "lock_holder_process", "--nocapture"])
        .env("AT02_LOCK_HOLDER", &base)
        .stdout(std::process::Stdio::null())
        .spawn()
        .unwrap();
    wait_for_file(&base.join("holder-ready"));
    let holder_pid = child.id().to_string();
    let blocked = sandbox.run(&["config", "validate"]);
    assert_eq!(blocked.status.code(), Some(8));
    assert!(String::from_utf8_lossy(&blocked.stderr).contains(&holder_pid));
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(base.join("locks/config.lock").exists());
    let after = sandbox.run(&["config", "validate"]);
    assert!(
        after.status.success(),
        "{}",
        String::from_utf8_lossy(&after.stderr)
    );
    assert!(base.join("locks/config.lock").exists());
}

#[test]
fn lock_counter_worker() {
    let Some(root) = std::env::var_os("AT02_COUNTER_WORKER") else {
        return;
    };
    let root = PathBuf::from(root);
    for _ in 0..50 {
        let guard = loop {
            match auto_tzim_core::config::acquire_lock_file(&root.join("locks/config.lock")) {
                Ok(guard) => break guard,
                Err(auto_tzim_core::config::ConfigError::Locked(_)) => std::thread::yield_now(),
                Err(error) => panic!("lock failed: {error}"),
            }
        };
        let counter = root.join("counter");
        let previous: u32 = fs::read_to_string(&counter).unwrap().parse().unwrap();
        std::thread::yield_now();
        fs::write(counter, (previous + 1).to_string()).unwrap();
        drop(guard);
    }
}

#[test]
fn two_processes_serialize_100_counter_updates() {
    let sandbox = Sandbox::new();
    fs::write(sandbox.root.join("counter"), "0").unwrap();
    let start = || {
        Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "lock_counter_worker", "--nocapture"])
            .env("AT02_COUNTER_WORKER", &sandbox.root)
            .stdout(std::process::Stdio::null())
            .spawn()
            .unwrap()
    };
    let mut a = start();
    let mut b = start();
    assert!(a.wait().unwrap().success());
    assert!(b.wait().unwrap().success());
    assert_eq!(
        fs::read_to_string(sandbox.root.join("counter")).unwrap(),
        "100"
    );
}

#[test]
fn successful_write_cleans_stale_temps_but_failed_write_does_not() {
    let sandbox = Sandbox::new();
    let folder = sandbox.folder("photos");
    fs::create_dir_all(sandbox.root.join("config")).unwrap();
    let stale = sandbox.root.join("config/.config.toml.1.0.tmp");
    let unrelated = sandbox.root.join("config/.config.toml.not-ours.tmp");
    fs::write(&stale, b"stale").unwrap();
    fs::write(&unrelated, b"keep").unwrap();
    fs::write(sandbox.config(), b"broken [[[").unwrap();
    assert_eq!(
        sandbox
            .run(&["folder", "add", folder.to_str().unwrap()])
            .status
            .code(),
        Some(2)
    );
    assert!(stale.exists());
    fs::remove_file(sandbox.config()).unwrap();
    let result = sandbox.run(&["folder", "add", folder.to_str().unwrap()]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(!stale.exists());
    assert_eq!(fs::read(unrelated).unwrap(), b"keep");
}

#[test]
fn unlocked_leftover_lock_file_is_reused() {
    use std::os::unix::fs::PermissionsExt;
    let sandbox = Sandbox::new();
    let path = sandbox.root.join("config/locks/config.lock");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, b"{\"pid\": 999999, \"host\": \"dead\"}").unwrap();
    let result = sandbox.run(&["config", "validate"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(path.exists());
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    let metadata = fs::read_to_string(&path).unwrap();
    assert!(metadata.contains("\"pid\":"));
    assert!(metadata.contains("\"start_time\":"));
    assert!(metadata.contains("\"host\":"));
}
