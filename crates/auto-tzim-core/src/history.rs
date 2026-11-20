//! Durable, per-folder apply records and append-only result history.

use crate::adapter;
use crate::apply::AppliedFile;
use crate::planner::{base64, json_string, timestamp, Plan, Row};
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy)]
pub enum Mode {
    Auto,
    Confirm,
    Declined,
}

pub struct RunMetadata<'a> {
    pub folder: &'a str,
    pub mode: Mode,
    pub batch_id: Option<&'a str>,
    pub config_hash: &'a str,
    pub cooldown_hours: u32,
}

pub(crate) struct RunHistory {
    pub directory: PathBuf,
    header: String,
    folder: String,
    counts: BTreeMap<&'static str, u64>,
    files: File,
}

pub(crate) fn private_directory(path: &Path) -> io::Result<()> {
    match fs::DirBuilder::new().mode(0o700).create(path) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error),
    }
    let meta = fs::symlink_metadata(path)?;
    if !meta.is_dir() || meta.permissions().mode() & 0o077 != 0 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "expected private state directory",
        ));
    }
    Ok(())
}

impl RunHistory {
    pub fn start(
        runs: &Path,
        id: &str,
        plan: &Plan,
        binary: &adapter::Identity,
        metadata: &RunMetadata<'_>,
    ) -> io::Result<Self> {
        let directory = runs.join(id);
        fs::DirBuilder::new().mode(0o700).create(&directory)?;
        File::open(runs)?.sync_all()?;
        let files = OpenOptions::new()
            .create_new(true)
            .append(true)
            .mode(0o600)
            .open(directory.join("files.jsonl"))?;
        let mode = match metadata.mode {
            Mode::Auto => "auto",
            Mode::Confirm => "confirm",
            Mode::Declined => "declined",
        };
        let header = format!(
            "\"schema\":1,\"run_id\":{},\"started_at\":{},\"mode\":{},\"folder\":{},\"root\":{},\"root_raw_b64\":{},\"plan_id\":{},\"plan_hash\":{},\"batch_id\":{},\"config_hash\":{},\"tzim\":{{\"path\":{},\"sha256\":{},\"version\":{},\"capabilities_schema\":{},\"result_schema\":{},\"adapter\":{}}}",
            json_string(id), now()?, json_string(mode), json_string(metadata.folder),
            json_string(&plan.root.to_string_lossy()), json_string(&base64(plan.root.as_os_str().as_bytes())),
            json_string(&plan.id), json_string(&plan.hash), nullable(metadata.batch_id),
            json_string(metadata.config_hash), os_json(binary.path.as_os_str()),
            json_string(&binary.sha256), json_string(&binary.version),
            binary.capabilities_schema, binary.result_schema, json_string(binary.adapter)
        );
        let history = Self {
            directory,
            header,
            folder: metadata.folder.to_owned(),
            counts: BTreeMap::new(),
            files,
        };
        history.write_record("running", "null", "null")?;
        Ok(history)
    }

    pub fn append(&mut self, row: &Row, result: &AppliedFile, elapsed: Duration) -> io::Result<()> {
        writeln!(self.files, "{}", self.file_record(row, result, elapsed)?)?;
        self.files.sync_all()?;
        *self.counts.entry(result.status).or_default() += 1;
        Ok(())
    }

    pub fn begin_file(&self, row: &Row) -> io::Result<()> {
        let result = AppliedFile {
            status: "interrupted",
            tzim_status: None,
            tzim_exit_code: None,
            tzim_error: None,
            tzim_stderr: None,
            reason: None,
            note: None,
            before: row.candidate.identity.size,
            after: row.candidate.identity.size,
        };
        let path = self.directory.join("in-flight.json");
        let next = self.directory.join("in-flight.json.next");
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(0o600)
            .open(&next)?;
        writeln!(
            file,
            "{{\"offset\":{},\"outcome\":{}}}",
            self.files.metadata()?.len(),
            self.file_record(row, &result, Duration::ZERO)?
        )?;
        file.sync_all()?;
        fs::rename(next, path)?;
        File::open(&self.directory)?.sync_all()
    }

    pub fn clear_file(&self) -> io::Result<()> {
        fs::remove_file(self.directory.join("in-flight.json"))?;
        File::open(&self.directory)?.sync_all()
    }

    fn file_record(
        &self,
        row: &Row,
        result: &AppliedFile,
        elapsed: Duration,
    ) -> io::Result<String> {
        let argv = row
            .argv
            .iter()
            .map(|arg| os_json(arg))
            .collect::<Vec<_>>()
            .join(",");
        let path = &row.candidate.path;
        let identity = &row.candidate.identity;
        Ok(format!(
            "{{\"v\":1,\"ts\":{},\"folder\":{},\"relative_path\":{},\"path_raw_b64\":{},\"status\":{},\"reason\":{},\"before\":{},\"after\":{},\"saved\":{},\"identity_before\":{{\"dev\":{},\"ino\":{},\"size\":{},\"mtime_ns\":{}}},\"tzim_status\":{},\"tzim_exit_code\":{},\"tzim_note\":{},\"tzim_error_kind\":{},\"tzim_error_message\":{},\"tzim_stderr\":{},\"duration_ms\":{},\"argv\":[{}]}}",
            now()?, json_string(&self.folder), json_string(&path.to_string_lossy()),
            json_string(&base64(path.as_os_str().as_bytes())), json_string(result.status),
            nullable(result.reason.as_deref()), result.before, result.after,
            result.before.saturating_sub(result.after), identity.dev, identity.ino,
            identity.size, identity.mtime_ns, nullable(result.tzim_status.as_deref()),
            result.tzim_exit_code.map_or_else(|| "null".into(), |code| code.to_string()),
            nullable(result.note.as_deref()),
            nullable(result.tzim_error.as_ref().map(|error| error.kind.as_str())),
            nullable(result.tzim_error.as_ref().map(|error| error.message.as_str())),
            nullable(result.tzim_stderr.as_deref()), elapsed.as_millis(), argv
        ))
    }

    pub fn finish(self, state: &str, exit: u8) -> io::Result<()> {
        self.write_record(state, &now()?, &exit.to_string())
    }

    fn write_record(&self, state: &str, finished: &str, exit: &str) -> io::Result<()> {
        let counts = self
            .counts
            .iter()
            .map(|(status, count)| format!("{}:{count}", json_string(status)))
            .collect::<Vec<_>>()
            .join(",");
        let path = self.directory.join("run.json.next");
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(0o600)
            .open(&path)?;
        writeln!(file, "{{{},\"state\":{},\"finished_at\":{finished},\"exit_code\":{exit},\"counts\":{{{counts}}}}}", self.header, json_string(state))?;
        file.sync_all()?;
        fs::rename(path, self.directory.join("run.json"))?;
        File::open(&self.directory)?.sync_all()
    }
}

fn nullable(value: Option<&str>) -> String {
    value.map(json_string).unwrap_or_else(|| "null".into())
}

fn os_json(value: &OsStr) -> String {
    match value.to_str() {
        Some(value) => json_string(value),
        None => format!("{{\"b64\":{}}}", json_string(&base64(value.as_bytes()))),
    }
}

fn now() -> io::Result<String> {
    current_timestamp().map(|value| json_string(&value))
}

fn current_timestamp() -> io::Result<String> {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?;
    Ok(timestamp(elapsed.as_secs(), elapsed.subsec_nanos()))
}

/// Recover incomplete installations before starting another apply. The caller
/// must hold the run lock for this scan and any subsequent processing.
pub fn recover_runs(state: &Path) -> io::Result<Vec<RecoveredRun>> {
    use crate::{install, json::Value};

    let runs = state.join("runs");
    let directories = match fs::read_dir(&runs) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    let mut recovered = Vec::new();
    for entry in directories {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let Some(id) =
            entry.file_name().into_string().ok().filter(|name| {
                name.len() == 32 && name.bytes().all(|byte| byte.is_ascii_hexdigit())
            })
        else {
            continue;
        };
        let directory = entry.path();
        let record = directory.join("run.json");
        let bytes = match read_record(&record) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        };
        let Value::Object(mut fields) = crate::json::parse(&bytes).map_err(io::Error::other)?
        else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid run record",
            ));
        };
        let string = |key| match fields.get(key) {
            Some(Value::String(value)) => Ok(value.as_str()),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "missing run record field",
            )),
        };
        let running = string("state")? == "running";
        if !running
            && (!matches!(string("state")?, "aborted" | "interrupted")
                || fields.contains_key("recovered_at"))
        {
            continue;
        }
        if string("run_id")? != id {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "run directory id mismatch",
            ));
        }

        let root = install::decode_path(string("root_raw_b64")?)?;
        let journal = directory.join("journal.jsonl");
        let outcomes = match fs::symlink_metadata(&journal) {
            Ok(meta) if meta.is_file() => install::recover(&journal, &root, &id)?,
            Ok(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid install journal",
                ))
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound && running => Vec::new(),
            Err(error) => return Err(error),
        };
        if outcomes
            .iter()
            .any(|(_, result)| *result == install::Recovery::Unresolved)
        {
            return Err(io::Error::other(format!("unresolved install in run {id}")));
        }

        clean_recovered_staging(state, &id)?;
        if running {
            fields.insert("state".into(), Value::String("interrupted".into()));
            fields.insert("finished_at".into(), Value::String(current_timestamp()?));
            fields.insert("exit_code".into(), Value::Number("1".into()));
            let files = directory.join("files.jsonl");
            let (mut counts, complete_bytes) = count_outcomes(&files)?;
            if let Some(outcome) = recover_in_flight(&directory, &files, complete_bytes)? {
                *counts.entry("interrupted".into()).or_default() += 1;
                if complete_bytes != fs::metadata(&files)?.len() {
                    // Preserve an incomplete append in place and keep the recovered
                    // outcome in the run record rather than writing past the tail.
                    fields.insert("tail_recovered".into(), Value::Bool(true));
                    fields.insert("interrupted_file".into(), outcome);
                } else {
                    let mut output = OpenOptions::new().append(true).open(&files)?;
                    writeln!(output, "{}", encode_value(&outcome))?;
                    output.sync_all()?;
                }
            }
            fields.insert(
                "counts".into(),
                Value::Object(
                    counts
                        .into_iter()
                        .map(|(status, count)| (status, Value::Number(count.to_string())))
                        .collect(),
                ),
            );
        }
        fields.insert("recovered_at".into(), Value::String(current_timestamp()?));
        let next = directory.join("run.json.next");
        // A previous writer may have died before publishing its temporary file.
        match fs::remove_file(&next) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        let mut output = OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(0o600)
            .open(&next)?;
        writeln!(output, "{}", encode_value(&Value::Object(fields)))?;
        output.sync_all()?;
        fs::rename(next, record)?;
        File::open(&directory)?.sync_all()?;
        recovered.push(RecoveredRun { id, outcomes });
    }
    Ok(recovered)
}

fn clean_recovered_staging(state: &Path, id: &str) -> io::Result<()> {
    let staging = state.join("staging");
    let metadata = match fs::symlink_metadata(&staging) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    if !metadata.is_dir() || metadata.permissions().mode() & 0o077 != 0 {
        return Err(io::Error::other("expected private staging directory"));
    }
    let directory = staging.join(id);
    match fs::symlink_metadata(&directory) {
        Ok(metadata) if metadata.is_dir() && metadata.permissions().mode() & 0o077 == 0 => {
            fs::remove_dir_all(directory)?;
            File::open(staging)?.sync_all()
        }
        Ok(_) => Err(io::Error::other("invalid run staging directory")),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

#[derive(Debug)]
pub struct RecoveredRun {
    pub id: String,
    pub outcomes: Vec<(PathBuf, crate::install::Recovery)>,
}

fn read_record(path: &Path) -> io::Result<Vec<u8>> {
    use std::io::Read;
    let mut bytes = Vec::new();
    File::open(path)?
        .take(crate::json::MAX_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn count_outcomes(path: &Path) -> io::Result<(BTreeMap<String, u64>, u64)> {
    use crate::json::{self, Value};
    use std::io::{BufRead, BufReader, Read};

    let mut reader = BufReader::new(File::open(path)?);
    let mut counts = BTreeMap::<String, u64>::new();
    let mut complete_bytes = 0;
    let mut line = Vec::new();
    loop {
        line.clear();
        if reader
            .by_ref()
            .take(json::MAX_BYTES as u64 + 1)
            .read_until(b'\n', &mut line)?
            == 0
        {
            break;
        }
        if line.len() > json::MAX_BYTES {
            return Err(io::Error::other("oversized history row"));
        }
        if !line.ends_with(b"\n") {
            break;
        }
        let row = json::parse(&line).map_err(io::Error::other)?;
        let Value::String(status) = row.member("status").map_err(io::Error::other)? else {
            return Err(io::Error::other("invalid history status"));
        };
        *counts.entry(status.to_owned()).or_default() += 1;
        complete_bytes += line.len() as u64;
    }
    Ok((counts, complete_bytes))
}

fn recover_in_flight(
    directory: &Path,
    files: &Path,
    complete_bytes: u64,
) -> io::Result<Option<crate::json::Value>> {
    use crate::json;
    let marker = match read_record(&directory.join("in-flight.json")) {
        Ok(bytes) => json::parse(&bytes).map_err(io::Error::other)?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let offset = marker.integer("offset").map_err(io::Error::other)?;
    if offset > complete_bytes || offset > fs::metadata(files)?.len() {
        return Err(io::Error::other("invalid in-flight history offset"));
    }
    if offset < complete_bytes {
        return Ok(None);
    }
    marker
        .member("outcome")
        .cloned()
        .map(Some)
        .map_err(io::Error::other)
}

pub fn encode_value(value: &crate::json::Value) -> String {
    use crate::json::Value;
    match value {
        Value::Null => "null".into(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.clone(),
        Value::String(value) => json_string(value),
        Value::Array(values) => format!(
            "[{}]",
            values
                .iter()
                .map(encode_value)
                .collect::<Vec<_>>()
                .join(",")
        ),
        Value::Object(fields) => format!(
            "{{{}}}",
            fields
                .iter()
                .map(|(key, value)| { format!("{}:{}", json_string(key), encode_value(value)) })
                .collect::<Vec<_>>()
                .join(",")
        ),
    }
}

pub const STATUSES: &[&str] = &[
    "planned",
    "optimized",
    "already_optimal",
    "skipped_unsupported",
    "skipped_by_tzim",
    "skipped_policy",
    "skipped_unstable",
    "skipped_by_user",
    "skipped_changed",
    "hardlink_skip",
    "failed",
    "interrupted",
    "incompatible_tzim",
];

pub struct HistoryRow {
    pub run_id: String,
    pub tzim: crate::json::Value,
    pub outcome: crate::json::Value,
}

/// Read recorded file outcomes newest first, without altering run state.
pub fn list(
    state: &Path,
    folder: Option<&str>,
    status: Option<&str>,
    limit: usize,
) -> io::Result<Vec<HistoryRow>> {
    use crate::json::{self, Value};
    use std::collections::VecDeque;
    use std::io::{BufRead, BufReader, Read};

    let entries = match fs::read_dir(state.join("runs")) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    let mut runs = Vec::new();
    for entry in entries {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let bytes = match read_record(&entry.path().join("run.json")) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        };
        let run = json::parse(&bytes).map_err(io::Error::other)?;
        if let Some(folder) = folder {
            if !string_field(&run, "folder")?.eq_ignore_ascii_case(folder) {
                continue;
            }
        }
        let started = string_field(&run, "started_at")?.to_owned();
        let id = string_field(&run, "run_id")?.to_owned();
        runs.push((started, id, entry.path(), run));
    }
    runs.sort_unstable_by(|a, b| (&b.0, &b.1).cmp(&(&a.0, &a.1)));

    let mut results = Vec::new();
    for (_, id, directory, run) in runs {
        let remaining = limit.saturating_sub(results.len());
        if remaining == 0 {
            break;
        }
        let mut latest = VecDeque::new();
        let mut keep = |row: Value| {
            if status.is_some_and(|status| {
                row.member("status").ok() != Some(&Value::String(status.into()))
            }) {
                return;
            }
            if latest.len() == remaining {
                latest.pop_front();
            }
            latest.push_back(row);
        };
        let mut reader = BufReader::new(File::open(directory.join("files.jsonl"))?);
        let mut line = Vec::new();
        loop {
            line.clear();
            let count = reader
                .by_ref()
                .take(json::MAX_BYTES as u64 + 1)
                .read_until(b'\n', &mut line)?;
            if count > json::MAX_BYTES {
                return Err(io::Error::other("oversized history row"));
            }
            if !line.ends_with(b"\n") {
                break;
            }
            keep(json::parse(&line).map_err(io::Error::other)?);
        }
        if let Ok(row) = run.member("interrupted_file") {
            keep(row.clone());
        }
        let binary = run.member("tzim").map_err(io::Error::other)?;
        for outcome in latest.into_iter().rev() {
            results.push(HistoryRow {
                run_id: id.clone(),
                tzim: binary.clone(),
                outcome,
            });
        }
    }
    Ok(results)
}

fn string_field<'a>(row: &'a crate::json::Value, key: &str) -> io::Result<&'a str> {
    match row.member(key).map_err(io::Error::other)? {
        crate::json::Value::String(value) => Ok(value),
        _ => Err(io::Error::other(format!("expected string in {key}"))),
    }
}

impl HistoryRow {
    pub fn json(&self) -> String {
        format!(
            "{{\"type\":\"file\",\"run_id\":{},\"tzim\":{},\"outcome\":{}}}",
            json_string(&self.run_id),
            encode_value(&self.tzim),
            encode_value(&self.outcome)
        )
    }

    pub fn summary(&self) -> io::Result<String> {
        let path = string_field(&self.outcome, "relative_path")?;
        let status = string_field(&self.outcome, "status")?;
        let mut result = format!("{} {status}", json_string(path));
        for (key, label) in [
            ("before", "before"),
            ("after", "after"),
            ("reason", "reason"),
            ("tzim_status", "tzim_status"),
            ("tzim_note", "note"),
            ("tzim_error_kind", "error_kind"),
            ("tzim_error_message", "error"),
        ] {
            if let Ok(value) = self.outcome.member(key) {
                if *value != crate::json::Value::Null {
                    result.push_str(&format!(" {label}={}", encode_value(value)));
                }
            }
        }
        result.push_str(&format!(
            " tzim={} sha256={}",
            json_string(string_field(&self.tzim, "version")?),
            json_string(string_field(&self.tzim, "sha256")?)
        ));
        Ok(result)
    }
}

/// Stream the durable file outcomes followed by the final run record.
pub fn write_run_json(state: &Path, id: &str, mut output: impl Write) -> io::Result<()> {
    use crate::json::{self, Value};
    use std::io::{BufRead, BufReader, Read};

    if id.len() != 32 || !id.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(io::Error::other("invalid run id"));
    }
    let directory = state.join("runs").join(id);
    let run = json::parse(&read_record(&directory.join("run.json"))?).map_err(io::Error::other)?;
    let plan_id = string_field(&run, "plan_id")?;
    let mut reader = BufReader::new(File::open(directory.join("files.jsonl"))?);
    let mut line = Vec::new();
    let encode = |value, kind: &str| -> io::Result<String> {
        let Value::Object(mut fields) = value else {
            return Err(io::Error::other("expected run record object"));
        };
        fields.insert("schema".into(), Value::Number("1".into()));
        fields.insert("type".into(), Value::String(kind.into()));
        fields.insert("run_id".into(), Value::String(id.into()));
        fields.insert("plan_id".into(), Value::String(plan_id.into()));
        Ok(encode_value(&Value::Object(fields)))
    };
    loop {
        line.clear();
        let count = reader
            .by_ref()
            .take(json::MAX_BYTES as u64 + 1)
            .read_until(b'\n', &mut line)?;
        if count == 0 {
            break;
        }
        let value = json::parse(&line).map_err(io::Error::other)?;
        writeln!(output, "{}", encode(value, "file")?)?;
    }
    writeln!(output, "{}", encode(run.clone(), "run")?)
}

pub struct StatusRecords {
    pub last_runs: Vec<crate::json::Value>,
    pub pending_plans: Vec<crate::json::Value>,
}

/// Return the latest run and unused plan records for each canonical root.
pub fn status_records(state: &Path) -> io::Result<StatusRecords> {
    use crate::json;
    let runs = match fs::read_dir(state.join("runs")) {
        Ok(runs) => Some(runs),
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(error) => return Err(error),
    };
    let mut latest = BTreeMap::<String, (String, String, json::Value)>::new();
    let mut used = std::collections::BTreeSet::new();
    for entry in runs.into_iter().flatten() {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let record = match read_record(&entry.path().join("run.json")) {
            Ok(record) => record,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        };
        let record = json::parse(&record).map_err(io::Error::other)?;
        if string_field(&record, "mode")? != "declined" {
            used.insert(string_field(&record, "plan_id")?.to_owned());
        }
        let root = string_field(&record, "root_raw_b64")?.to_owned();
        let started = string_field(&record, "started_at")?.to_owned();
        let id = string_field(&record, "run_id")?.to_owned();
        if latest
            .get(&root)
            .is_none_or(|(time, old_id, _)| (&started, &id) > (time, old_id))
        {
            latest.insert(root, (started, id, record));
        }
    }
    let last_runs = latest.into_values().map(|(_, _, record)| record).collect();
    let plans = match fs::read_dir(state.join("plans")) {
        Ok(plans) => Some(plans),
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(error) => return Err(error),
    };
    let mut latest = BTreeMap::new();
    let now = SystemTime::now();
    for entry in plans.into_iter().flatten() {
        let entry = entry?;
        let path = entry.path();
        if !entry.file_type()?.is_file() || path.extension() != Some(OsStr::new("json")) {
            continue;
        }
        let header = crate::planner::read_header(&path)?;
        let root = string_field(&header, "root_raw_b64")?.to_owned();
        let created = string_field(&header, "created")?.to_owned();
        let id = string_field(&header, "plan_id")?.to_owned();
        let newer = latest
            .get(&root)
            .is_none_or(|(time, previous, _)| (&created, &id) > (time, previous));
        if newer {
            let age = now
                .duration_since(entry.metadata()?.modified()?)
                .unwrap_or_default()
                .as_secs();
            let summary = json::Value::Object(BTreeMap::from([
                ("plan_id".into(), json::Value::String(id.clone())),
                (
                    "root".into(),
                    header.member("root").map_err(io::Error::other)?.clone(),
                ),
                ("created".into(), json::Value::String(created.clone())),
                (
                    "rows".into(),
                    json::Value::Number(
                        header
                            .integer("row_count")
                            .map_err(io::Error::other)?
                            .to_string(),
                    ),
                ),
                ("age_seconds".into(), json::Value::Number(age.to_string())),
            ]));
            latest.insert(root, (created, id, summary));
        }
    }
    let pending_plans = latest
        .into_values()
        .filter_map(|(_, id, summary)| (!used.contains(&id)).then_some(summary))
        .collect();
    Ok(StatusRecords {
        last_runs,
        pending_plans,
    })
}
