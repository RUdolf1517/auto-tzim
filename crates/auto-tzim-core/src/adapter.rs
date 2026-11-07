//! Read-only, fail-closed inspection of the supported tzim binary.
use crate::{hash, json};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

const BUDGET: Duration = Duration::from_secs(10);
const STDERR_LIMIT: u64 = 64 * 1024;

#[derive(Debug, Clone)]
pub struct Identity {
    pub path: PathBuf,
    pub realpath: PathBuf,
    pub size: u64,
    pub sha256: String,
    pub version: String,
    pub capabilities_schema: u64,
    pub result_schema: u64,
    pub adapter: &'static str,
    pub capabilities: json::Value,
}

#[derive(Debug)]
pub struct Incompatible(pub String);
impl std::fmt::Display for Incompatible {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "incompatible_tzim: {}", self.0)
    }
}
impl std::error::Error for Incompatible {}

fn failure(message: impl Into<String>) -> Incompatible {
    Incompatible(message.into())
}

fn read_pipe(mut pipe: impl Read, limit: u64) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    pipe.by_ref()
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > limit {
        return Err(format!("output exceeds {limit} bytes"));
    }
    Ok(bytes)
}

fn probe(path: &Path, flag: &str, start: Instant) -> Result<Vec<u8>, Incompatible> {
    if start.elapsed() >= BUDGET {
        return Err(failure(format!("timeout after {:?}", start.elapsed())));
    }
    let mut child = Command::new(path)
        .arg(flag)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| failure(format!("{flag}: spawn {}: {e}", path.display())))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| failure("stdout pipe missing"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| failure("stderr pipe missing"))?;
    let (tx, rx) = mpsc::channel();
    let tx_stdout = tx.clone();
    std::thread::spawn(move || {
        let _ = tx_stdout.send((0, read_pipe(stdout, json::MAX_BYTES as u64)));
    });
    std::thread::spawn(move || {
        let _ = tx.send((1, read_pipe(stderr, STDERR_LIMIT)));
    });
    let mut streams: [Option<Vec<u8>>; 2] = [None, None];
    let mut status = None;
    loop {
        while let Ok((index, result)) = rx.try_recv() {
            match result {
                Ok(bytes) => streams[index] = Some(bytes),
                Err(e) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(failure(format!("{flag}: {e}")));
                }
            }
        }
        if status.is_none() {
            match child.try_wait() {
                Ok(value) => status = value,
                Err(e) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(failure(format!("{flag}: wait: {e}")));
                }
            }
        }
        if start.elapsed() >= BUDGET {
            let _ = child.kill();
            let _ = child.wait();
            return Err(failure(format!(
                "{flag}: timeout after {:?}",
                start.elapsed()
            )));
        }
        if status.is_some() && streams.iter().all(Option::is_some) {
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let stdout = streams[0].take().unwrap_or_default();
    let stderr = streams[1].take().unwrap_or_default();
    if !status.expect("checked above").success() {
        return Err(failure(format!(
            "{flag}: exit {:?}; stderr={}",
            status,
            String::from_utf8_lossy(&stderr)
        )));
    }
    if stdout.is_empty() {
        return Err(failure(format!("{flag}: empty stdout")));
    }
    Ok(stdout)
}

fn version(bytes: &[u8]) -> Result<String, Incompatible> {
    let text = std::str::from_utf8(bytes).map_err(|_| failure("--version: invalid UTF-8"))?;
    let line = text.strip_suffix('\n').unwrap_or(text);
    let semver = line
        .strip_prefix("tzim ")
        .ok_or_else(|| failure(format!("--version: observed {text:?}")))?;
    let parts: Vec<_> = semver.split('.').collect();
    if parts.len() != 3
        || parts.iter().any(|s| {
            s.is_empty()
                || !s.bytes().all(|b| b.is_ascii_digit())
                || (s.len() > 1 && s.starts_with('0'))
        })
    {
        return Err(failure(format!("--version: observed {text:?}")));
    }
    let numbers = parts
        .iter()
        .map(|s| s.parse::<u64>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| failure(format!("--version: out of range {text:?}")))?;
    if numbers[0] != 0 || numbers[1] != 16 {
        return Err(failure(format!("--version: unsupported {semver}")));
    }
    Ok(semver.into())
}

pub fn inspect(path: &Path) -> Result<Identity, Incompatible> {
    let realpath = path
        .canonicalize()
        .map_err(|e| failure(format!("{}: {e}", path.display())))?;
    let binary =
        std::fs::read(&realpath).map_err(|e| failure(format!("{}: {e}", realpath.display())))?;
    let size = binary.len() as u64;
    let sha256 = hash::hex(&hash::sha256(&binary));
    let start = Instant::now();
    let version = version(&probe(&realpath, "--version", start)?)?;
    let capabilities = json::parse(&probe(&realpath, "--capabilities", start)?)
        .map_err(|e| failure(format!("--capabilities: {e}")))?;
    let capabilities_schema = capabilities
        .integer("capabilities_schema_version")
        .map_err(|e| failure(e.to_string()))?;
    let result_schema = capabilities
        .integer("result_schema_version")
        .map_err(|e| failure(e.to_string()))?;
    if capabilities_schema != 1 {
        return Err(failure(format!("version={version} capabilities_schema_version={capabilities_schema} result_schema_version={result_schema}")));
    }
    match capabilities
        .member("build_features")
        .map_err(|e| failure(e.to_string()))?
    {
        json::Value::Object(fields)
            if fields.values().all(|v| matches!(v, json::Value::Bool(_))) => {}
        _ => return Err(failure("key build_features: expected object of booleans")),
    }
    for key in ["compiled_formats", "default_enabled_formats"] {
        match capabilities
            .member(key)
            .map_err(|e| failure(e.to_string()))?
        {
            json::Value::Array(items)
                if items.iter().all(|v| matches!(v, json::Value::String(_))) => {}
            _ => return Err(failure(format!("key {key}: expected array of strings"))),
        }
    }
    for key in [
        "selection_flags",
        "handler_opt_ins",
        "transformations",
        "confirmation_policies",
    ] {
        match capabilities
            .member(key)
            .map_err(|e| failure(e.to_string()))?
        {
            json::Value::Array(items)
                if items.iter().all(|v| matches!(v, json::Value::Object(_))) => {}
            _ => return Err(failure(format!("key {key}: expected array of objects"))),
        }
    }
    Ok(Identity {
        path: path.to_owned(),
        realpath,
        size,
        sha256,
        version,
        capabilities_schema,
        result_schema,
        adapter: "tzim-0.16",
        capabilities,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn strict_versions() {
        for good in ["tzim 0.16.0\n", "tzim 0.16.1"] {
            assert!(version(good.as_bytes()).is_ok());
        }
        for bad in [
            "tzim 0.15.0",
            "tzim 0.17.0",
            "tzim 1.0.0",
            "",
            "tzim 0.16.01",
            "tzim 0.16.0\nextra",
            "tzim 0.16.0\r\n",
        ] {
            assert!(version(bad.as_bytes()).is_err(), "{bad:?}");
        }
    }
}

/// D-02's apply result mapping. A dry-run-only status has no apply mapping.
pub const STATUS_MAP: &[(&str, &str)] = &[
    ("optimized", "optimized"),
    ("already_optimal", "already_optimal"),
    ("unsupported", "skipped_unsupported"),
    ("skipped", "skipped_by_tzim"),
    ("failed", "failed"),
];

#[derive(Debug, PartialEq, Eq)]
pub struct MappedResult<'a> {
    pub status: &'static str,
    pub reason: Option<&'a str>,
    pub note: Option<&'a str>,
}

/// Preserve unknown reason codes and the note without interpreting either.
pub fn map_result<'a>(
    status: &str,
    reason_code: Option<&'a str>,
    note: Option<&'a str>,
) -> Result<MappedResult<'a>, Incompatible> {
    let mapped = STATUS_MAP
        .iter()
        .find(|(input, _)| *input == status)
        .map(|(_, output)| *output)
        .ok_or_else(|| failure(format!("unknown result status: {status}")))?;
    Ok(MappedResult {
        status: mapped,
        reason: reason_code,
        note,
    })
}

#[cfg(test)]
mod mapping_tests {
    use super::*;
    #[test]
    fn reason_and_note_are_opaque() {
        for reason in [Some("format_disabled"), Some("something_new"), None] {
            let row = map_result("skipped", reason, Some("verbatim\nnot interpreted")).unwrap();
            assert_eq!(row.status, "skipped_by_tzim");
            assert_eq!(row.reason, reason);
            assert_eq!(row.note, Some("verbatim\nnot interpreted"));
        }
        for (input, output) in STATUS_MAP {
            assert_eq!(map_result(input, None, None).unwrap().status, *output);
        }
        assert!(map_result("unknown", None, None).is_err());
    }
}
