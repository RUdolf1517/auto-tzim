//! Durable installation of a verified staged replacement.

use crate::planner::{base64, json_string};
use crate::runner::{hash_checked, RunError, VerifiedReplacement};
use crate::staging::{check_source, StageError, SOURCE_FLAGS};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub enum InstallError {
    Source(StageError),
    Verification(RunError),
    ChangedReplacement,
    DifferentFilesystem,
    Io(io::Error),
}

impl From<io::Error> for InstallError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub struct InstallJournal {
    file: File,
    run_id: String,
    sequence: u64,
}

impl InstallJournal {
    /// Start a journal in the private run directory while holding the run lock.
    pub fn create(run_dir: &Path, run_id: &str) -> io::Result<Self> {
        if run_id.is_empty()
            || run_id.len() > 64
            || !run_id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid run id",
            ));
        }
        let file = OpenOptions::new()
            .append(true)
            .create_new(true)
            .mode(0o600)
            .open(run_dir.join("journal.jsonl"))?;
        File::open(run_dir)?.sync_all()?;
        Ok(Self {
            file,
            run_id: run_id.into(),
            sequence: 0,
        })
    }

    fn append(&mut self, step: &str, fields: &str) -> io::Result<()> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_secs();
        writeln!(
            self.file,
            "{{\"v\":1,\"ts\":\"{timestamp}\",\"step\":\"{step}\",{fields}}}"
        )?;
        self.file.sync_all()
    }
}

fn raw_path(path: &Path) -> String {
    json_string(&base64(path.as_os_str().as_bytes()))
}

/// Install a verified replacement and retain a recoverable original until the
/// new directory entry has been synced. Hold the run lock across this call.
pub fn install(
    journal: &mut InstallJournal,
    replacement: VerifiedReplacement,
) -> Result<(), InstallError> {
    let check =
        || check_source(&replacement.root, &replacement.source).map_err(InstallError::Source);
    let source = check()?;
    let parent = source.parent().expect("checked source has a parent");
    let directory = File::open(parent)?;
    if directory.metadata()?.dev() != replacement.identity.dev {
        return Err(InstallError::DifferentFilesystem);
    }

    let staged = OpenOptions::new()
        .read(true)
        .custom_flags(SOURCE_FLAGS)
        .open(&replacement.path)?;
    if crate::stability::Identity::from_metadata(&staged.metadata()?) != replacement.identity
        || hash_checked(&replacement.path, &replacement.identity)
            .map_err(InstallError::Verification)?
            != replacement.sha256
    {
        return Err(InstallError::ChangedReplacement);
    }
    let source_mode = fs::symlink_metadata(&source)?.permissions().mode() & 0o777;
    staged.set_permissions(fs::Permissions::from_mode(source_mode))?;
    staged.sync_all()?;

    journal.sequence += 1;
    let backup_name = format!(".auto-tzim-old-{}-{}", journal.run_id, journal.sequence);
    let backup = parent.join(&backup_name);
    let identity = &replacement.source.identity;
    let key = format!(
        "\"file\":{},\"file_raw_b64\":{}",
        json_string(&replacement.source.path.to_string_lossy()),
        raw_path(&replacement.source.path)
    );
    let digest = crate::hash::hex(&replacement.sha256);
    journal.append(
        "stat",
        &format!(
            "{key},\"identity\":{{\"dev\":{},\"ino\":{},\"size\":{},\"mtime_ns\":{}}}",
            identity.dev, identity.ino, identity.size, identity.mtime_ns
        ),
    )?;
    journal.append(
        "staged",
        &format!(
            "{key},\"staged_raw_b64\":{},\"sha256\":\"{digest}\"",
            raw_path(&replacement.path)
        ),
    )?;
    check()?;
    let staging_dir = replacement
        .path
        .parent()
        .expect("staging path has a parent");
    let reservation_path = staging_dir.join(format!(".auto-tzim-reserve-{}", journal.sequence));
    let reservation = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&reservation_path)?;
    let reservation = reservation.metadata()?;
    journal.append(
        "replaced",
        &format!(
            "{key},\"old\":{},\"sha256\":\"{digest}\",\"reserved_dev\":{},\"reserved_ino\":{},\"replacement_dev\":{},\"replacement_ino\":{}",
            json_string(&backup_name),
            reservation.dev(),
            reservation.ino(),
            replacement.identity.dev,
            replacement.identity.ino
        ),
    )?;
    #[cfg(test)]
    tests::checkpoint("journaled");
    fs::hard_link(&reservation_path, &backup)?;
    #[cfg(test)]
    tests::checkpoint("reserved");
    check()?;
    let slot = fs::symlink_metadata(&backup)?;
    if !slot.is_file() || slot.dev() != reservation.dev() || slot.ino() != reservation.ino() {
        return Err(InstallError::Source(StageError::SourceChanged));
    }
    fs::rename(&source, &backup)?;
    directory.sync_all()?;
    #[cfg(test)]
    tests::checkpoint("moved");
    let moved = fs::symlink_metadata(&backup)?;
    if !moved.is_file() || crate::stability::Identity::from_metadata(&moved) != *identity {
        return Err(InstallError::Source(StageError::SourceChanged));
    }
    // Publish without overwriting a path recreated by another process.
    fs::hard_link(&replacement.path, &source)?;
    let mut installed = regular_identity(&source)?.ok_or(InstallError::ChangedReplacement)?;
    if installed.links != 2 {
        return Err(InstallError::ChangedReplacement);
    }
    installed.links = 1;
    if installed != replacement.identity {
        return Err(InstallError::ChangedReplacement);
    }
    directory.sync_all()?;
    #[cfg(test)]
    tests::checkpoint("published");
    fs::remove_file(&replacement.path)?;
    fs::remove_file(&reservation_path)?;
    File::open(staging_dir)?.sync_all()?;
    fs::remove_file(&backup)?;
    directory.sync_all()?;
    #[cfg(test)]
    tests::checkpoint("old_removed");
    journal.append("committed", &key)?;
    #[cfg(test)]
    tests::checkpoint("committed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{discovery::Candidate, hash::sha256, stability::Identity};
    use std::path::PathBuf;

    pub(super) fn checkpoint(name: &str) {
        if std::env::var("AT05_CRASH_POINT").as_deref() == Ok(name) {
            let _ = std::process::Command::new("/bin/kill")
                .args(["-KILL", &std::process::id().to_string()])
                .status();
            panic!("failed to SIGKILL install fixture");
        }
    }

    #[test]
    fn install_crash_child() {
        let Some(base) = std::env::var_os("AT05_CRASH_FIXTURE") else {
            return;
        };
        let base = PathBuf::from(base);
        let root = base.join("root");
        let staged = base.join("staged.png");
        let replacement = VerifiedReplacement {
            source: Candidate {
                path: "photo.png".into(),
                identity: Identity::from_metadata(&fs::metadata(root.join("photo.png")).unwrap()),
            },
            identity: Identity::from_metadata(&fs::metadata(&staged).unwrap()),
            sha256: sha256(b"small"),
            path: staged,
            root,
        };
        let mut journal = InstallJournal::create(&base, "crash").unwrap();
        install(&mut journal, replacement).unwrap();
        panic!("install did not reach the requested crash point");
    }

    #[test]
    fn recovers_after_sigkill_at_install_boundaries() {
        use std::os::unix::process::ExitStatusExt;

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let base =
            std::env::temp_dir().join(format!("auto-tzim-crash-{}-{nonce}", std::process::id()));
        fs::create_dir(&base).unwrap();
        let fixture = Fixture(fs::canonicalize(base).unwrap());
        for (point, expected, outcome) in [
            (
                "journaled",
                b"original".as_slice(),
                Some(Recovery::OriginalPresent),
            ),
            ("reserved", b"original", Some(Recovery::OriginalPresent)),
            ("moved", b"original", Some(Recovery::OriginalRestored)),
            ("published", b"small", Some(Recovery::ReplacementPresent)),
            ("old_removed", b"small", Some(Recovery::ReplacementPresent)),
            ("committed", b"small", None),
        ] {
            let base = fixture.0.join(point);
            let root = base.join("root");
            fs::create_dir_all(&root).unwrap();
            fs::write(root.join("photo.png"), b"original").unwrap();
            fs::write(base.join("staged.png"), b"small").unwrap();
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "install::tests::install_crash_child"])
                .env("AT05_CRASH_FIXTURE", &base)
                .env("AT05_CRASH_POINT", point)
                .output()
                .unwrap();
            assert_eq!(
                output.status.signal(),
                Some(9),
                "{point}: {}",
                String::from_utf8_lossy(&output.stdout)
            );
            let recovered = recover(&base.join("journal.jsonl"), &root, "crash").unwrap();
            assert_eq!(
                recovered,
                outcome
                    .into_iter()
                    .map(|value| (PathBuf::from("photo.png"), value))
                    .collect::<Vec<_>>(),
                "{point}"
            );
            assert_eq!(
                fs::read(root.join("photo.png")).unwrap(),
                expected,
                "{point}"
            );
            assert_eq!(fs::read_dir(&root).unwrap().count(), 1, "{point}");
        }
    }

    struct Fixture(PathBuf);
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn installs_and_records_the_durable_swap() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let base =
            std::env::temp_dir().join(format!("auto-tzim-install-{}-{nonce}", std::process::id()));
        fs::create_dir(&base).unwrap();
        let fixture = Fixture(fs::canonicalize(base).unwrap());
        let root = fixture.0.join("root");
        fs::create_dir(&root).unwrap();
        let source_path = root.join("photo.png");
        let staged = fixture.0.join("photo.png");
        fs::write(&source_path, b"original").unwrap();
        fs::write(&staged, b"small").unwrap();
        fs::set_permissions(&source_path, fs::Permissions::from_mode(0o640)).unwrap();
        let source = Candidate {
            path: "photo.png".into(),
            identity: Identity::from_metadata(&fs::symlink_metadata(&source_path).unwrap()),
        };
        let identity = Identity::from_metadata(&fs::symlink_metadata(&staged).unwrap());

        let mut journal = InstallJournal::create(&fixture.0, "test").unwrap();
        install(
            &mut journal,
            VerifiedReplacement {
                root: root.clone(),
                source,
                path: staged.clone(),
                identity,
                sha256: sha256(b"small"),
            },
        )
        .unwrap();

        assert_eq!(fs::read(&source_path).unwrap(), b"small");
        assert_eq!(fs::metadata(source_path).unwrap().mode() & 0o777, 0o640);
        assert_eq!(fs::read_dir(root).unwrap().count(), 1);
        assert!(!staged.exists());
        let log = fs::read_to_string(fixture.0.join("journal.jsonl")).unwrap();
        let steps: Vec<_> = log
            .lines()
            .map(|line| {
                crate::json::parse(line.as_bytes())
                    .unwrap()
                    .member("step")
                    .unwrap()
                    .clone()
            })
            .collect();
        assert_eq!(
            steps,
            ["stat", "staged", "replaced", "committed"]
                .map(|s| crate::json::Value::String(s.into()))
        );

        let journal_path = fixture.0.join("journal.jsonl");
        assert!(recover(&journal_path, &fixture.0.join("root"), "test")
            .unwrap()
            .is_empty());

        // Lose the commit record after publishing, as if the process died
        // after unlinking the original and before appending that record.
        let incomplete = log.lines().take(3).collect::<Vec<_>>().join("\n");
        fs::write(&journal_path, format!("{incomplete}\n{{")).unwrap();
        let root = fixture.0.join("root");
        assert_eq!(
            recover(&journal_path, &root, "test").unwrap(),
            vec![(PathBuf::from("photo.png"), Recovery::ReplacementPresent)]
        );

        // Recreate a journal for the state just after moving the original aside.
        let backup = root.join(".auto-tzim-old-test-1");
        fs::rename(root.join("photo.png"), &backup).unwrap();
        let old = Identity::from_metadata(&fs::metadata(&backup).unwrap());
        let entries = format!(
            "{{\"v\":1,\"file_raw_b64\":\"cGhvdG8ucG5n\",\"step\":\"stat\",\"identity\":{{\"dev\":{},\"ino\":{},\"size\":{},\"mtime_ns\":{}}}}}\n{}\n",
            old.dev, old.ino, old.size, old.mtime_ns, log.lines().nth(2).unwrap()
        );
        fs::write(&journal_path, entries).unwrap();
        let foreign = root.join("photo.png");
        fs::write(&foreign, b"small").unwrap();
        assert_eq!(
            recover(&journal_path, &root, "test").unwrap(),
            vec![(PathBuf::from("photo.png"), Recovery::Unresolved)]
        );
        assert_eq!(fs::read(&backup).unwrap(), b"small");
        assert_eq!(fs::read(&foreign).unwrap(), b"small");
        fs::remove_file(foreign).unwrap();
        assert_eq!(
            recover(&journal_path, &root, "test").unwrap(),
            vec![(PathBuf::from("photo.png"), Recovery::OriginalRestored)]
        );
        assert_eq!(fs::read(root.join("photo.png")).unwrap(), b"small");
        assert!(!backup.exists());
    }
}

/// Disk state observed for an installation without a durable commit record.
#[derive(Debug, PartialEq, Eq)]
pub enum Recovery {
    OriginalPresent,
    OriginalRestored,
    ReplacementPresent,
    Unresolved,
}

/// Replay one run's journal under the run lock, in the run's canonical root.
pub fn recover(
    journal: &Path,
    root: &Path,
    run_id: &str,
) -> io::Result<Vec<(std::path::PathBuf, Recovery)>> {
    use crate::json;
    use std::collections::BTreeMap;
    use std::io::{BufRead, BufReader};
    use std::path::Component;

    let mut states = BTreeMap::new();
    let mut pending = BTreeMap::new();
    let mut reader = BufReader::new(File::open(journal)?);
    let mut line = Vec::new();
    loop {
        line.clear();
        if reader.read_until(b'\n', &mut line)? == 0 || !line.ends_with(b"\n") {
            break;
        }
        let row = json::parse(&line).map_err(io::Error::other)?;
        if row.integer("v").map_err(io::Error::other)? != 1 {
            return Err(invalid("unknown journal version"));
        }
        let path = decode_path(field(&row, "file_raw_b64")?)?;
        if path.as_os_str().is_empty()
            || !path
                .components()
                .all(|part| matches!(part, Component::Normal(_)))
        {
            return Err(invalid("invalid journal path"));
        }
        match field(&row, "step")? {
            "stat" => {
                let identity = row.member("identity").map_err(io::Error::other)?;
                let crate::json::Value::Number(mtime) =
                    identity.member("mtime_ns").map_err(io::Error::other)?
                else {
                    return Err(invalid("invalid modification time"));
                };
                states.insert(
                    path,
                    crate::stability::Identity {
                        dev: number(identity, "dev")?,
                        ino: number(identity, "ino")?,
                        size: number(identity, "size")?,
                        mtime_ns: mtime.parse().map_err(io::Error::other)?,
                        links: 1,
                    },
                );
            }
            "staged" => {}
            "replaced" => {
                let name = field(&row, "old")?.to_owned();
                let prefix = format!(".auto-tzim-old-{run_id}-");
                if !name
                    .strip_prefix(&prefix)
                    .is_some_and(|suffix| suffix.parse::<u64>().is_ok())
                    || !name
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-'))
                {
                    return Err(invalid("invalid backup name"));
                }
                let original = states
                    .remove(&path)
                    .ok_or_else(|| invalid("missing original identity"))?;
                pending.insert(path, (original, name, row));
            }
            "committed" => {
                pending.remove(&path);
            }
            _ => return Err(invalid("unknown journal step")),
        }
    }

    if !pending.is_empty() && fs::canonicalize(root)? != root {
        return Err(invalid("run root changed"));
    }
    let mut results = Vec::new();
    for (path, (original, name, row)) in pending {
        let source = root.join(&path);
        let parent = source.parent().ok_or_else(|| invalid("missing parent"))?;
        let result = if fs::canonicalize(parent).ok().as_deref() == Some(parent) {
            recover_file(&source, &parent.join(name), &original, &row)?
        } else {
            Recovery::Unresolved
        };
        results.push((path, result));
    }
    Ok(results)
}

fn recover_file(
    source: &Path,
    backup: &Path,
    original: &crate::stability::Identity,
    row: &crate::json::Value,
) -> io::Result<Recovery> {
    let current = regular_identity(source)?;
    let old = regular_identity(backup)?;
    if current.as_ref() == Some(original) {
        if let Some(old) = old {
            if old.dev != number(row, "reserved_dev")?
                || old.ino != number(row, "reserved_ino")?
                || old.size != 0
                || !matches!(old.links, 1 | 2)
            {
                return Ok(Recovery::Unresolved);
            }
            fs::remove_file(backup)?;
            sync_parent(source)?;
        }
        return Ok(Recovery::OriginalPresent);
    }

    if let Some(identity) = current {
        if let Some(mut old) = old.clone() {
            if identity == old && identity.links == 2 {
                old.links = 1;
                if &old == original {
                    sync_parent(source)?;
                    fs::remove_file(backup)?;
                    sync_parent(source)?;
                    return Ok(Recovery::OriginalRestored);
                }
            }
        }
        let digest = field(row, "sha256")?;
        if !matches!(identity.links, 1 | 2)
            || identity.dev != number(row, "replacement_dev")?
            || identity.ino != number(row, "replacement_ino")?
            || hash_checked(source, &identity)
                .ok()
                .map(|sum| crate::hash::hex(&sum))
                .as_deref()
                != Some(digest)
            || old.as_ref().is_some_and(|old| old != original)
        {
            return Ok(Recovery::Unresolved);
        }
        if old.is_some() {
            sync_parent(source)?;
            fs::remove_file(backup)?;
            sync_parent(source)?;
        }
        return Ok(Recovery::ReplacementPresent);
    }

    if old.as_ref() != Some(original) {
        return Ok(Recovery::Unresolved);
    }
    fs::hard_link(backup, source)?;
    sync_parent(source)?;
    fs::remove_file(backup)?;
    sync_parent(source)?;
    Ok(Recovery::OriginalRestored)
}

fn regular_identity(path: &Path) -> io::Result<Option<crate::stability::Identity>> {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.is_file() => Ok(Some(crate::stability::Identity::from_metadata(&meta))),
        Ok(_) => Err(invalid("journal path is no longer a regular file")),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

fn sync_parent(path: &Path) -> io::Result<()> {
    File::open(path.parent().ok_or_else(|| invalid("missing parent"))?)?.sync_all()
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn field<'a>(row: &'a crate::json::Value, key: &str) -> io::Result<&'a str> {
    match row.member(key).map_err(io::Error::other)? {
        crate::json::Value::String(value) => Ok(value),
        _ => Err(invalid("expected journal string")),
    }
}

fn number(row: &crate::json::Value, key: &str) -> io::Result<u64> {
    row.integer(key).map_err(io::Error::other)
}

pub(crate) fn decode_path(value: &str) -> io::Result<std::path::PathBuf> {
    use std::os::unix::ffi::OsStringExt;
    let mut bytes = Vec::new();
    let mut bits = 0u8;
    let mut buffer = 0u16;
    for byte in value.bytes().take_while(|&byte| byte != b'=') {
        let digit = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return Err(invalid("invalid base64 path")),
        };
        buffer = (buffer << 6) | u16::from(digit);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            bytes.push((buffer >> bits) as u8);
        }
    }
    if base64(&bytes) != value || bytes.contains(&0) {
        return Err(invalid("invalid base64 path"));
    }
    Ok(std::ffi::OsString::from_vec(bytes).into())
}
