//! Copy a planned source into a private run directory before invoking tzim.

use crate::{discovery::Candidate, stability::Identity};
use std::fs::{self, File, Metadata, OpenOptions};
use std::io::{self, Read};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Component, Path, PathBuf};

// The std-only build supports macOS and Linux. O_NONBLOCK prevents an intervening
// FIFO replacement from blocking open, and O_NOFOLLOW rejects a final symlink.
#[cfg(target_os = "macos")]
pub(crate) const SOURCE_FLAGS: i32 = 0x0004 | 0x0100;
#[cfg(target_os = "linux")]
pub(crate) const SOURCE_FLAGS: i32 = 0o4000 | 0o400000;

pub struct StagedFile {
    path: PathBuf,
    pub(crate) candidate: Candidate,
    pub(crate) root: PathBuf,
    pub(crate) identity: Identity,
}

impl StagedFile {
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[derive(Debug)]
pub enum StageError {
    InvalidPath,
    SourceChanged,
    Io(io::Error),
}

impl From<io::Error> for StageError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

fn check_identity(metadata: &Metadata, expected: &Identity) -> Result<(), StageError> {
    if !metadata.is_file() || expected.links != 1 || Identity::from_metadata(metadata) != *expected
    {
        return Err(StageError::SourceChanged);
    }
    Ok(())
}

/// Recheck the planned path, including ancestor containment, before using it.
pub fn check_source(root: &Path, candidate: &Candidate) -> Result<PathBuf, StageError> {
    if candidate.path.as_os_str().is_empty()
        || !candidate
            .path
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
    {
        return Err(StageError::InvalidPath);
    }
    let source = root.join(&candidate.path);
    let parent = source.parent().ok_or(StageError::InvalidPath)?;
    if fs::canonicalize(parent).ok().as_deref() != Some(parent) {
        return Err(StageError::SourceChanged);
    }
    let metadata = fs::symlink_metadata(&source).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            StageError::SourceChanged
        } else {
            StageError::Io(error)
        }
    })?;
    check_identity(&metadata, &candidate.identity)?;
    Ok(source)
}

/// Copy into a dedicated, already-created private staging directory.
pub fn stage_file(
    root: &Path,
    candidate: &Candidate,
    staging_dir: &Path,
) -> Result<StagedFile, StageError> {
    let directory = fs::canonicalize(staging_dir)?;
    if directory.starts_with(root) {
        return Err(StageError::InvalidPath);
    }
    let metadata = fs::symlink_metadata(staging_dir)?;
    if !metadata.is_dir() || metadata.permissions().mode() & 0o077 != 0 {
        return Err(StageError::InvalidPath);
    }

    let source = check_source(root, candidate)?;
    let mut input = OpenOptions::new()
        .read(true)
        .custom_flags(SOURCE_FLAGS)
        .open(&source)?;
    check_identity(&input.metadata()?, &candidate.identity)?;
    check_source(root, candidate)?;

    let path = directory.join(source.file_name().ok_or(StageError::InvalidPath)?);
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&path)?;
    let result = copy(&mut input, &mut output, root, candidate).map(|identity| StagedFile {
        path: path.clone(),
        candidate: candidate.clone(),
        root: root.to_owned(),
        identity,
    });
    if result.is_err() {
        let _ = fs::remove_file(&path);
    }
    result
}

fn copy(
    input: &mut File,
    output: &mut File,
    root: &Path,
    candidate: &Candidate,
) -> Result<Identity, StageError> {
    if io::copy(
        &mut input.take(candidate.identity.size.saturating_add(1)),
        output,
    )? != candidate.identity.size
    {
        return Err(StageError::SourceChanged);
    }
    check_identity(&input.metadata()?, &candidate.identity)?;
    check_source(root, candidate)?;
    output.sync_all()?;
    Ok(Identity::from_metadata(&output.metadata()?))
}

/// Check the filesystem that will hold the sequential staging copies.
pub fn available_bytes(path: &Path) -> io::Result<u64> {
    let directory = path
        .ancestors()
        .find(|ancestor| ancestor.is_dir())
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no staging ancestor"))?;
    let directory = fs::canonicalize(directory)?;
    let output = std::process::Command::new("/bin/df")
        .args(["-Pk"])
        .arg(directory)
        .env_clear()
        .env("LC_ALL", "C")
        .stdin(std::process::Stdio::null())
        .output()?;
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "df exited with {}",
            output.status
        )));
    }
    // POSIX df uses 1024-byte blocks with -k. The capacity column identifies
    // the preceding available-block count even when names contain spaces.
    let blocks = output
        .stdout
        .split(|&byte| byte == b'\n')
        .nth(1)
        .and_then(|line| {
            let columns: Vec<_> = line
                .split(|byte| byte.is_ascii_whitespace())
                .filter(|column| !column.is_empty())
                .collect();
            columns.windows(5).find_map(|cols| {
                if !cols[4].ends_with(b"%") {
                    return None;
                }
                let parse = |value| std::str::from_utf8(value).ok()?.parse::<i128>().ok();
                parse(cols[1])?;
                parse(cols[2])?;
                parse(cols[3])
            })
        })
        .ok_or_else(|| io::Error::other("invalid staging filesystem capacity"))?;
    Ok(
        u64::try_from(blocks.saturating_mul(1024).clamp(0, i128::from(u64::MAX)))
            .expect("clamped to u64"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::{symlink, DirBuilderExt};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn stages_an_independent_copy_and_refuses_a_replaced_source() {
        struct Fixture(PathBuf);
        impl Drop for Fixture {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let base =
            std::env::temp_dir().join(format!("auto-tzim-stage-{}-{nonce}", std::process::id()));
        fs::DirBuilder::new().mode(0o700).create(&base).unwrap();
        let fixture = Fixture(fs::canonicalize(base).unwrap());
        let root = fixture.0.join("root");
        let stage = fixture.0.join("stage");
        for path in [&root, &stage] {
            fs::DirBuilder::new().mode(0o700).create(path).unwrap();
        }

        let source = root.join("image.png");
        fs::write(&source, b"original").unwrap();
        let candidate = Candidate {
            path: PathBuf::from("image.png"),
            identity: Identity::from_metadata(&fs::symlink_metadata(&source).unwrap()),
        };
        assert!(matches!(
            stage_file(&root, &candidate, &root),
            Err(StageError::InvalidPath)
        ));

        let copy = stage_file(&root, &candidate, &stage).unwrap();
        assert_eq!(fs::read(copy.path()).unwrap(), b"original");
        fs::write(copy.path(), b"shorter").unwrap();
        assert_eq!(fs::read(&source).unwrap(), b"original");

        fs::remove_file(&source).unwrap();
        symlink(copy.path(), &source).unwrap();
        assert!(matches!(
            stage_file(&root, &candidate, &stage),
            Err(StageError::SourceChanged)
        ));
    }
}
