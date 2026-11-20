//! Validation of the result stream for a single staged-file apply.

use crate::{adapter, config, json};
use crate::{
    discovery::Candidate,
    hash,
    stability::Identity,
    staging::{StagedFile, SOURCE_FLAGS},
};
use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io::{BufReader, Read};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

#[derive(Debug, PartialEq, Eq)]
pub struct FileResult {
    pub status: &'static str,
    pub tzim_status: String,
    pub error: Option<TzimError>,
    pub stderr: Option<String>,
    pub reason: Option<String>,
    pub note: Option<String>,
    pub before: u64,
    pub after: u64,
    replacement: Option<VerifiedReplacement>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TzimError {
    pub kind: String,
    pub message: String,
}

impl FileResult {
    pub fn into_replacement(self) -> Option<VerifiedReplacement> {
        self.replacement
    }
}

/// A successful staged optimization bound to the planned source and output bytes.
#[derive(Debug, PartialEq, Eq)]
pub struct VerifiedReplacement {
    pub(crate) root: PathBuf,
    pub(crate) source: Candidate,
    pub(crate) path: PathBuf,
    pub(crate) identity: Identity,
    pub(crate) sha256: [u8; 32],
}

fn optional_string<'a>(value: &'a json::Value, key: &str) -> Result<Option<&'a str>, json::Error> {
    match value.member(key)? {
        json::Value::Null => Ok(None),
        json::Value::String(value) => Ok(Some(value)),
        _ => Err(json::Error(format!("key {key}: expected string or null"))),
    }
}

/// Decode the two records emitted by an apply invocation for exactly one file.
pub fn parse_apply_output(stdout: &[u8]) -> Result<FileResult, json::Error> {
    if stdout.len() > json::MAX_BYTES {
        return Err(json::Error("tzim output exceeds 1 MiB".into()));
    }
    let mut lines = stdout
        .strip_suffix(b"\n")
        .unwrap_or(stdout)
        .split(|&byte| byte == b'\n');
    let file = json::parse(lines.next().unwrap_or_default())?;
    let summary = json::parse(lines.next().unwrap_or_default())?;
    let string = |value: &str| json::Value::String(value.into());
    if lines.next().is_some()
        || *file.member("type")? != string("file")
        || *summary.member("type")? != string("summary")
        || summary.integer("schema")? != 2
        || summary.integer("files")? != 1
        || *summary.member("dry_run")? != json::Value::Bool(false)
        || *summary.member("strip_metadata")? != json::Value::Bool(false)
    {
        return Err(json::Error("invalid single-file apply stream".into()));
    }

    let status = optional_string(&file, "status")?
        .ok_or_else(|| json::Error("missing apply status".into()))?;
    let mapped = adapter::map_result(
        status,
        optional_string(&file, "reason_code")?,
        optional_string(&file, "note")?,
    )
    .map_err(json::Error)?;
    for counter in [
        "optimized",
        "would_optimize",
        "already_optimal",
        "unsupported",
        "skipped",
        "failed",
    ] {
        if summary.integer(counter)? != u64::from(counter == status) {
            return Err(json::Error(format!(
                "inconsistent summary counter: {counter}"
            )));
        }
    }

    let before = file.integer("before")?;
    let after = file.integer("after")?;
    let saved = file.integer("saved")?;
    if before.checked_sub(after) != Some(saved)
        || (status == "optimized") != (saved > 0)
        || summary.integer("before")? != before
        || summary.integer("after")? != after
        || summary.integer("saved")? != saved
    {
        return Err(json::Error("inconsistent apply sizes".into()));
    }

    let error = match file.member("error") {
        Ok(json::Value::Null) | Err(_) => None,
        Ok(value) if status == "failed" => {
            let required = |key| {
                optional_string(value, key)?
                    .map(str::to_owned)
                    .ok_or_else(|| json::Error(format!("missing error {key}")))
            };
            Some(TzimError {
                kind: required("kind")?,
                message: required("message")?,
            })
        }
        Ok(_) => return Err(json::Error("failure payload on a successful result".into())),
    };
    Ok(FileResult {
        status: mapped.status,
        tzim_status: status.to_owned(),
        error,
        stderr: None,
        reason: mapped.reason.map(str::to_owned),
        note: mapped.note.map(str::to_owned),
        before,
        after,
        replacement: None,
    })
}

/// Hold the same kernel lock across recovery and processing.
pub fn acquire_run_lock() -> Result<config::ConfigLock, config::ConfigError> {
    config::acquire_lock_file(&config::lock_dir()?.join("run.lock"))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Interrupt {
    Int = 130,
    Term = 143,
}

#[derive(Default)]
pub struct Cancellation(std::sync::atomic::AtomicU8);

impl Cancellation {
    pub fn request(&self, signal: Interrupt) {
        use std::sync::atomic::Ordering::Relaxed;
        let _ = self.0.compare_exchange(0, signal as u8, Relaxed, Relaxed);
    }

    pub fn requested(&self) -> Option<Interrupt> {
        match self.0.load(std::sync::atomic::Ordering::Relaxed) {
            130 => Some(Interrupt::Int),
            143 => Some(Interrupt::Term),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum RunError {
    Io(std::io::Error),
    Output(String),
    Timeout,
    Interrupted(Interrupt),
    Exit {
        status: std::process::ExitStatus,
        stderr: Vec<u8>,
        report: Option<Box<FileResult>>,
    },
}

/// Run tzim on a file already copied into the private staging directory.
/// The caller owns staging, checks the original identity, and handles installation.
pub fn process_staged_file(
    binary: &adapter::Identity,
    staged: &StagedFile,
    formats: Option<&OsStr>,
    timeout: Duration,
    cancellation: &Cancellation,
) -> Result<FileResult, RunError> {
    if let Some(signal) = cancellation.requested() {
        return Err(RunError::Interrupted(signal));
    }
    let before = std::fs::symlink_metadata(staged.path()).map_err(RunError::Io)?;
    if !before.is_file() || Identity::from_metadata(&before) != staged.identity {
        return Err(RunError::Output(
            "staged file changed before execution".into(),
        ));
    }
    let mut command = Command::new(&binary.realpath);
    command.arg("--json");
    if let Some(formats) = formats {
        command.arg("--formats").arg(formats);
    }
    let mut child = command
        .arg("--")
        .arg(staged.path())
        .env_clear()
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(RunError::Io)?;

    let (status, [stdout, stderr]) =
        capture(&mut child, timeout, Duration::from_secs(10), cancellation)?;
    if !status.success() {
        let report = if matches!(status.code(), Some(1 | 4)) {
            parse_apply_output(&stdout)
                .ok()
                .filter(|result| result.status == "failed")
                .map(|mut result| {
                    result.stderr = stderr_text(&stderr);
                    Box::new(result)
                })
        } else {
            None
        };
        return Err(RunError::Exit {
            status,
            stderr,
            report,
        });
    }
    let mut result =
        parse_apply_output(&stdout).map_err(|error| RunError::Output(error.to_string()))?;
    result.stderr = stderr_text(&stderr);
    let after = std::fs::symlink_metadata(staged.path()).map_err(RunError::Io)?;
    let identity = Identity::from_metadata(&after);
    if !after.is_file()
        || identity.links != 1
        || result.before != staged.identity.size
        || result.after != identity.size
        || (result.status != "optimized" && identity != staged.identity)
    {
        return Err(RunError::Output(
            "staged file does not match tzim result".into(),
        ));
    }
    if result.status == "optimized" {
        let sha256 = hash_checked(staged.path(), &identity)?;
        result.replacement = Some(VerifiedReplacement {
            root: staged.root.clone(),
            source: staged.candidate.clone(),
            path: staged.path().to_owned(),
            identity,
            sha256,
        });
    }
    Ok(result)
}

pub(crate) fn hash_checked(path: &Path, identity: &Identity) -> Result<[u8; 32], RunError> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(SOURCE_FLAGS)
        .open(path)
        .map_err(RunError::Io)?;
    let check = |metadata: fs::Metadata| {
        if !metadata.is_file() || Identity::from_metadata(&metadata) != *identity {
            return Err(RunError::Output(
                "file changed while checking its digest".into(),
            ));
        }
        Ok(())
    };
    check(file.metadata().map_err(RunError::Io)?)?;
    let mut reader = BufReader::new(&file).take(identity.size.saturating_add(1));
    let digest = hash::sha256_reader(&mut reader).map_err(RunError::Io)?;
    if reader.limit() != 1 {
        return Err(RunError::Output("file size changed while hashing".into()));
    }
    check(file.metadata().map_err(RunError::Io)?)?;
    check(fs::symlink_metadata(path).map_err(RunError::Io)?)?;
    Ok(digest)
}

fn capture(
    child: &mut Child,
    timeout: Duration,
    grace: Duration,
    cancellation: &Cancellation,
) -> Result<(std::process::ExitStatus, [Vec<u8>; 2]), RunError> {
    let result = capture_streams(child, timeout, grace, cancellation);
    if result.is_err() {
        let _ = child.kill();
        let _ = child.wait();
    }
    result
}

fn capture_streams(
    child: &mut Child,
    timeout: Duration,
    grace: Duration,
    cancellation: &Cancellation,
) -> Result<(std::process::ExitStatus, [Vec<u8>; 2]), RunError> {
    let started = Instant::now();
    let stdout = child.stdout.take().expect("stdout configured as a pipe");
    let stderr = child.stderr.take().expect("stderr configured as a pipe");
    let (tx, rx) = mpsc::channel();
    let stdout_tx = tx.clone();
    std::thread::spawn(move || {
        let _ = stdout_tx.send((0, adapter::read_pipe(stdout, json::MAX_BYTES as u64)));
    });
    std::thread::spawn(move || {
        let _ = tx.send((1, read_stderr(stderr).map_err(|error| error.to_string())));
    });

    let mut status = None;
    let mut streams = [None, None];
    loop {
        while let Ok((index, result)) = rx.try_recv() {
            streams[index] = Some(result.map_err(RunError::Output)?);
        }
        if status.is_none() {
            status = child.try_wait().map_err(RunError::Io)?;
        }
        if let Some(signal) = cancellation.requested() {
            if status.is_none() {
                terminate(child, grace);
            }
            return Err(RunError::Interrupted(signal));
        }
        if let (Some(status), [Some(_), Some(_)]) = (status, &streams) {
            return Ok((
                status,
                streams.map(|stream| stream.expect("stream received")),
            ));
        }
        if started.elapsed() >= timeout {
            if status.is_none() {
                terminate(child, grace);
            }
            return Err(RunError::Timeout);
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

const STDERR_LIMIT: u64 = 64 * 1024;
const STDERR_TRUNCATED: &[u8] = b"\n[stderr truncated]";

fn read_stderr(mut pipe: impl Read) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    pipe.by_ref()
        .take(STDERR_LIMIT + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > STDERR_LIMIT {
        bytes.truncate(STDERR_LIMIT as usize - STDERR_TRUNCATED.len());
        bytes.extend_from_slice(STDERR_TRUNCATED);
        // Continue draining: closing the pipe would interrupt an otherwise valid
        // tzim run. The capture deadline still bounds how long the parent waits.
        std::io::copy(&mut pipe, &mut std::io::sink())?;
    }
    Ok(bytes)
}

fn stderr_text(bytes: &[u8]) -> Option<String> {
    (!bytes.is_empty()).then(|| String::from_utf8_lossy(bytes).into_owned())
}

fn terminate(child: &mut Child, grace: Duration) {
    // Use the platform utility with a fixed executable and a numeric child PID.
    let sent = Command::new("/bin/kill")
        .arg("-TERM")
        .arg(child.id().to_string())
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success());
    if sent {
        let started = Instant::now();
        while started.elapsed() < grace {
            if !matches!(child.try_wait(), Ok(None)) {
                return;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
    // The caller force-kills and reaps the child on error.
}

/// Inspect the kernel lock without creating a lock file or changing its metadata.
pub fn active_run() -> Result<Option<json::Value>, config::ConfigError> {
    use std::os::unix::fs::MetadataExt;

    let path = config::lock_dir()?.join("run.lock");
    let metadata = match fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let invalid = || config::ConfigError::Internal("invalid run lock file".into());
    if !metadata.is_file() {
        return Err(invalid());
    }
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(SOURCE_FLAGS)
        .open(path)?;
    let opened = file.metadata()?;
    if !opened.is_file() || (opened.dev(), opened.ino()) != (metadata.dev(), metadata.ino()) {
        return Err(invalid());
    }
    match file.try_lock() {
        Ok(()) => Ok(None),
        Err(std::fs::TryLockError::WouldBlock) => {
            let mut bytes = Vec::new();
            file.take(4096).read_to_end(&mut bytes)?;
            // The lock is authoritative even when the holder hasn't finished
            // publishing its diagnostic fields.
            Ok(Some(json::parse(&bytes).unwrap_or(json::Value::Null)))
        }
        Err(std::fs::TryLockError::Error(error)) => Err(error.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE: &str = r#"{"type":"file","status":"skipped","reason_code":"refused","note":"keep","before":41,"after":41,"saved":0}"#;
    const SUMMARY: &str = r#"{"type":"summary","schema":2,"files":1,"optimized":0,"would_optimize":0,"already_optimal":0,"unsupported":0,"skipped":1,"failed":0,"before":41,"after":41,"saved":0,"dry_run":false,"strip_metadata":false}"#;

    #[test]
    fn output_limits_reject_large_stdout_and_drain_large_stderr() {
        let spawn = |script| {
            Command::new("/bin/sh")
                .args(["-c", script])
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap()
        };
        let mut child = spawn("exec /bin/dd if=/dev/zero bs=1048576 count=10");
        let outcome = capture(
            &mut child,
            Duration::from_secs(5),
            Duration::from_millis(50),
            &Cancellation::default(),
        );
        assert!(matches!(outcome, Err(RunError::Output(_))));

        let mut child = spawn("(/bin/dd if=/dev/zero bs=1048576 count=10) >&2; printf result");
        let outcome = capture(
            &mut child,
            Duration::from_secs(5),
            Duration::from_millis(50),
            &Cancellation::default(),
        );
        let (status, [stdout, stderr]) = outcome.unwrap();
        assert!(status.success());
        assert_eq!(stdout, b"result");
        assert_eq!(stderr.len(), STDERR_LIMIT as usize);
        assert!(stderr.ends_with(STDERR_TRUNCATED));
    }

    #[test]
    fn timeout_force_kills_and_reaps_a_child_ignoring_term() {
        use std::os::unix::process::ExitStatusExt;

        let mut child = Command::new("/bin/sh")
            .args(["-c", "trap '' TERM; printf ready; exec /bin/sleep 60"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut ready = [0; 5];
        child
            .stdout
            .as_mut()
            .unwrap()
            .read_exact(&mut ready)
            .unwrap();
        assert_eq!(&ready, b"ready");
        let result = capture(
            &mut child,
            Duration::from_millis(30),
            Duration::from_millis(75),
            &Cancellation::default(),
        );
        assert!(matches!(result, Err(RunError::Timeout)));
        assert_eq!(child.wait().unwrap().signal(), Some(9));
    }

    #[test]
    fn timeout_terminates_and_reaps_a_child() {
        let mut child = Command::new("/bin/sleep")
            .arg("60")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let result = capture(
            &mut child,
            Duration::from_millis(30),
            Duration::from_millis(100),
            &Cancellation::default(),
        );
        assert!(matches!(result, Err(RunError::Timeout)));
        use std::os::unix::process::ExitStatusExt;
        assert_eq!(child.wait().unwrap().signal(), Some(15));
    }

    #[test]
    fn maps_a_valid_apply_stream() {
        let output = format!("{FILE}\n{SUMMARY}\n");
        let result = parse_apply_output(output.as_bytes()).unwrap();
        assert_eq!(result.status, "skipped_by_tzim");
        assert_eq!(result.reason.as_deref(), Some("refused"));
        assert_eq!(result.note.as_deref(), Some("keep"));
    }

    #[test]
    fn rejects_ambiguous_or_inconsistent_streams() {
        for output in [
            FILE.into(),
            format!("{FILE}\n{SUMMARY}\n{SUMMARY}"),
            format!("{FILE}\n{}", &SUMMARY[..SUMMARY.len() - 1]),
            format!(
                "{FILE}\n{}",
                SUMMARY.replace("\"schema\":2", "\"schema\":3")
            ),
            format!(
                "{FILE}\n{}",
                SUMMARY.replace("\"skipped\":1", "\"skipped\":0")
            ),
            format!(
                "{FILE}\n{}",
                SUMMARY.replace("\"after\":41", "\"after\":40")
            ),
            format!(
                "{FILE}\n{}",
                SUMMARY.replace("\"dry_run\":false", "\"dry_run\":true")
            ),
        ] {
            assert!(parse_apply_output(output.as_bytes()).is_err(), "{output}");
        }
    }
}
