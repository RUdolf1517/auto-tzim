//! Reuse decisions only when the file, runner and policy are unchanged.

use crate::{adapter, apply::AppliedFile, history::encode_value, json, planner, runner::TzimError};
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::unix::{ffi::OsStrExt, fs::OpenOptionsExt};
use std::path::{Path, PathBuf};

const COMPACT_AT: u64 = 8 * 1024 * 1024;
const VERDICTS: &[&str] = &[
    "already_optimal",
    "skipped_unsupported",
    "skipped_by_tzim",
    "skipped_policy",
    "failed",
];

pub(crate) struct Cache {
    state: PathBuf,
    entries: BTreeMap<(String, String), json::Value>,
    torn_tail: bool,
}

impl Cache {
    pub fn open(state: &Path) -> io::Result<Self> {
        let mut cache = Self {
            state: state.to_owned(),
            entries: BTreeMap::new(),
            torn_tail: false,
        };
        let file = match File::open(state.join("eligibility.jsonl")) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(cache),
            Err(error) => return Err(error),
        };
        let mut reader = BufReader::new(file);
        let mut line = Vec::new();
        loop {
            line.clear();
            let count = reader
                .by_ref()
                .take(json::MAX_BYTES as u64 + 1)
                .read_until(b'\n', &mut line)?;
            if count == 0 {
                break;
            }
            if count > json::MAX_BYTES {
                return Err(io::Error::other("oversized eligibility row"));
            }
            if !line.ends_with(b"\n") {
                cache.torn_tail = true;
                break;
            }
            let entry = json::parse(&line).map_err(io::Error::other)?;
            let field = |name| {
                string(&entry, name)
                    .map(str::to_owned)
                    .ok_or_else(|| io::Error::other("invalid eligibility key"))
            };
            if entry.integer("v").map_err(io::Error::other)? != 1 {
                return Err(io::Error::other("unsupported eligibility schema"));
            }
            cache
                .entries
                .insert((field("root")?, field("path")?), entry);
        }
        Ok(cache)
    }

    pub fn lookup(&self, key: &Key, size: u64) -> Option<AppliedFile> {
        let entry = self.entries.get(&(key.root.clone(), key.path.clone()))?;
        if string(entry, "row_hash")? != key.row_hash
            || string(entry, "tzim_sha256")? != key.binary
            || string(entry, "adapter")? != key.adapter
            || string(entry, "policy_hash")? != key.policy
        {
            return None;
        }
        let status = string(entry, "verdict")?;
        let status = VERDICTS.iter().copied().find(|&value| value == status)?;
        let field = |name| string(entry, name).map(str::to_owned);
        Some(AppliedFile {
            status,
            tzim_status: field("tzim_status"),
            tzim_exit_code: entry
                .integer("tzim_exit_code")
                .ok()
                .and_then(|code| i32::try_from(code).ok()),
            tzim_error: field("error_kind")
                .zip(field("error_message"))
                .map(|(kind, message)| TzimError { kind, message }),
            tzim_stderr: field("stderr"),
            reason: field("reason"),
            note: field("note"),
            before: size,
            after: size,
        })
    }

    pub fn add(&mut self, key: Key, result: &AppliedFile) -> io::Result<()> {
        if !VERDICTS.contains(&result.status) {
            return Ok(());
        }
        let elapsed = std::time::UNIX_EPOCH.elapsed().map_err(io::Error::other)?;
        let text = |value: &str| json::Value::String(value.to_owned());
        let optional = |value: Option<&str>| value.map(text).unwrap_or(json::Value::Null);
        let entry = json::Value::Object(BTreeMap::from([
            ("v".into(), json::Value::Number("1".into())),
            ("root".into(), text(&key.root)),
            ("path".into(), text(&key.path)),
            ("row_hash".into(), text(&key.row_hash)),
            ("tzim_sha256".into(), text(key.binary)),
            ("adapter".into(), text(key.adapter)),
            ("policy_hash".into(), text(key.policy)),
            ("verdict".into(), text(result.status)),
            (
                "tzim_exit_code".into(),
                result.tzim_exit_code.map_or(json::Value::Null, |code| {
                    json::Value::Number(code.to_string())
                }),
            ),
            (
                "decided_at".into(),
                text(&planner::timestamp(
                    elapsed.as_secs(),
                    elapsed.subsec_nanos(),
                )),
            ),
            (
                "tzim_status".into(),
                optional(result.tzim_status.as_deref()),
            ),
            ("reason".into(), optional(result.reason.as_deref())),
            ("note".into(), optional(result.note.as_deref())),
            ("stderr".into(), optional(result.tzim_stderr.as_deref())),
            (
                "error_kind".into(),
                optional(result.tzim_error.as_ref().map(|e| e.kind.as_str())),
            ),
            (
                "error_message".into(),
                optional(result.tzim_error.as_ref().map(|e| e.message.as_str())),
            ),
        ]));
        let path = self.state.join("eligibility.jsonl");
        let line = encode_value(&entry);
        if line.len() >= json::MAX_BYTES {
            return Ok(());
        }
        self.entries.insert((key.root, key.path), entry);
        if self.torn_tail || fs::metadata(&path).is_ok_and(|meta| meta.len() >= COMPACT_AT) {
            let next = self.state.join("eligibility.jsonl.next");
            match fs::remove_file(&next) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
            let mut file = OpenOptions::new()
                .create_new(true)
                .write(true)
                .mode(0o600)
                .open(&next)?;
            for entry in self.entries.values() {
                writeln!(file, "{}", encode_value(entry))?;
            }
            file.sync_all()?;
            fs::rename(next, path)?;
            self.torn_tail = false;
        } else {
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .mode(0o600)
                .open(path)?;
            writeln!(file, "{line}")?;
            file.sync_all()?;
        }
        File::open(&self.state)?.sync_all()
    }
}

pub(crate) struct Key<'a> {
    root: String,
    path: String,
    row_hash: String,
    binary: &'a str,
    adapter: &'a str,
    policy: &'a str,
}

impl<'a> Key<'a> {
    pub fn new(
        root: &Path,
        row: &planner::Row,
        binary: &'a adapter::Identity,
        policy: &'a str,
    ) -> Self {
        Self {
            root: planner::base64(root.as_os_str().as_bytes()),
            path: planner::base64(row.candidate.path.as_os_str().as_bytes()),
            row_hash: planner::plan_hash(std::slice::from_ref(row)),
            binary: &binary.sha256,
            adapter: binary.adapter,
            policy,
        }
    }
}

fn string<'a>(entry: &'a json::Value, field: &str) -> Option<&'a str> {
    match entry.member(field).ok()? {
        json::Value::String(value) => Some(value),
        _ => None,
    }
}
