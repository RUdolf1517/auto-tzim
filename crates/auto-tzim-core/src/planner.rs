//! Deterministic canonical plan rows and per-scan random identifiers.
use crate::discovery::Candidate;
use crate::hash;
use std::io::Read;
use std::os::unix::ffi::OsStrExt;

/// A plan row and the exact argv determined by the caller's adapter contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub candidate: Candidate,
    pub argv: Vec<std::ffi::OsString>,
}

/// An in-memory plan for a single canonical root. This type does not publish files.
#[derive(Debug)]
pub struct Plan {
    pub id: String,
    pub root: std::path::PathBuf,
    pub hash: String,
    pub rows: Vec<Row>,
    pub discovery: crate::discovery::Discovery,
}

/// Failure while assembling already-discovered roots.
#[derive(Debug)]
pub enum BuildError {
    Arguments(String),
    Entropy(std::io::Error),
}

/// Assemble one plan per root without invoking a process or writing plan files.
/// Exact future arguments are injected by the caller, not inferred here.
pub fn build_plans<F>(
    discoveries: Vec<(crate::config::Folder, crate::discovery::Discovery)>,
    arguments: F,
) -> Result<Vec<Plan>, BuildError>
where
    F: FnMut(&crate::config::Folder, &Candidate) -> Result<Vec<std::ffi::OsString>, String>,
{
    build_plans_with_ids(discoveries, arguments, plan_id)
}

fn build_plans_with_ids<F, G>(
    discoveries: Vec<(crate::config::Folder, crate::discovery::Discovery)>,
    mut arguments: F,
    mut next_id: G,
) -> Result<Vec<Plan>, BuildError>
where
    F: FnMut(&crate::config::Folder, &Candidate) -> Result<Vec<std::ffi::OsString>, String>,
    G: FnMut() -> std::io::Result<String>,
{
    let mut plans = Vec::with_capacity(discoveries.len());
    let mut ids = std::collections::HashSet::new();
    for (folder, discovery) in discoveries {
        let mut rows = Vec::with_capacity(discovery.candidates.len());
        for candidate in &discovery.candidates {
            rows.push(Row {
                candidate: candidate.clone(),
                argv: arguments(&folder, candidate).map_err(BuildError::Arguments)?,
            });
        }
        rows.sort_by(|a, b| {
            a.candidate
                .path
                .as_os_str()
                .as_bytes()
                .cmp(b.candidate.path.as_os_str().as_bytes())
        });
        let id = next_id().map_err(BuildError::Entropy)?;
        if !ids.insert(id.clone()) {
            return Err(BuildError::Entropy(std::io::Error::other(
                "plan_id_collision",
            )));
        }
        let hash = plan_hash(&rows);
        plans.push(Plan {
            id,
            root: folder.root,
            hash,
            rows,
            discovery,
        });
    }
    Ok(plans)
}

/// Construct the profile-owned argv described by D-59, preserving path bytes.
pub fn arguments(folder: &crate::config::Folder, candidate: &Candidate) -> Vec<std::ffi::OsString> {
    let mut args = Vec::new();
    if !folder.formats.is_empty() {
        let mut formats = folder.formats.clone();
        formats.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
        args.push("--formats".into());
        args.push(formats.join(",").into());
    }
    args.push("--".into());
    args.push(candidate.path.as_os_str().to_owned());
    args
}

/// Return canonical JSON lines in D-52's fixed key and bytewise path order.
pub fn canonical_rows(rows: &[Row]) -> String {
    let mut rows: Vec<_> = rows.iter().collect();
    rows.sort_by(|a, b| {
        a.candidate
            .path
            .as_os_str()
            .as_bytes()
            .cmp(b.candidate.path.as_os_str().as_bytes())
    });
    let mut output = String::new();
    for row in rows {
        let id = &row.candidate.identity;
        let path = base64(row.candidate.path.as_os_str().as_bytes());
        let argv = row
            .argv
            .iter()
            .map(|item| match item.to_str() {
                Some(value) => json_string(value),
                None => format!("{{\"b64\":{}}}", json_string(&base64(item.as_bytes()))),
            })
            .collect::<Vec<_>>()
            .join(",");
        output.push_str(&format!("{{\"path_raw_b64\":{},\"size\":{},\"dev\":{},\"ino\":{},\"mtime_ns\":{},\"argv\":[{}]}}\n", json_string(&path), id.size, id.dev, id.ino, id.mtime_ns, argv));
    }
    output
}

/// Hash the canonical rows; IDs, clock and batch composition are excluded.
pub fn plan_hash(rows: &[Row]) -> String {
    hash::hex(&hash::sha256(canonical_rows(rows).as_bytes()))
}

/// Draw 128 bits from the OS-provided random device for an event ID.
pub fn plan_id() -> std::io::Result<String> {
    let mut bytes = [0u8; 16];
    std::fs::File::open("/dev/urandom")?.read_exact(&mut bytes)?;
    Ok(hash::hex(&bytes))
}

/// Encode arbitrary bytes losslessly as standard padded base64.
pub fn base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::new();
    for chunk in bytes.chunks(3) {
        let a = chunk[0];
        let b = chunk.get(1).copied().unwrap_or(0);
        let c = chunk.get(2).copied().unwrap_or(0);
        output.push(TABLE[(a >> 2) as usize] as char);
        output.push(TABLE[(((a & 3) << 4) | (b >> 4)) as usize] as char);
        output.push(if chunk.len() > 1 {
            TABLE[(((b & 15) << 2) | (c >> 6)) as usize] as char
        } else {
            '='
        });
        output.push(if chunk.len() > 2 {
            TABLE[(c & 63) as usize] as char
        } else {
            '='
        });
    }
    output
}

/// Escape a Unicode string as a JSON string without a JSON dependency.
pub fn json_string(value: &str) -> String {
    let mut result = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            ch if ch.is_control() => result.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => result.push(ch),
        }
    }
    result.push('"');
    result
}

/// Serialize a plan record. The hash is computed only from D-52 canonical rows.
pub fn document(plan: &Plan, created: std::time::SystemTime) -> std::io::Result<String> {
    let created = format_timestamp(created)?;
    let rows = document_rows(&plan.rows)?;
    let skipped = plan
        .discovery
        .skipped
        .iter()
        .map(|skip| {
            let status = if skip.reason == "hardlink_skip"
                || skip.reason == "skipped_unstable"
                || skip.reason == "depth_exceeded"
            {
                skip.reason.as_str()
            } else {
                "skipped_policy"
            };
            format!(
                "{{\"path\":{},\"path_raw_b64\":{},\"status\":{},\"reason\":{}}}",
                json_string(&skip.path.to_string_lossy()),
                json_string(&base64(skip.path.as_os_str().as_bytes())),
                json_string(status),
                json_string(&skip.reason)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    Ok(format!("{{\"schema\":1,\"plan_id\":{},\"created\":{},\"root\":{},\"root_raw_b64\":{},\"plan_hash\":{},\"row_count\":{},\"rows\":[{}],\"skipped\":[{}]}}\n", json_string(&plan.id), json_string(&created), json_string(&plan.root.to_string_lossy()), json_string(&base64(plan.root.as_os_str().as_bytes())), json_string(&plan.hash), rows.len(), rows.join(","), skipped))
}

// UTC civil date conversion, Gregorian calendar; no external time dependency.
fn document_rows(plan_rows: &[Row]) -> std::io::Result<Vec<String>> {
    let canonical = canonical_rows(plan_rows);
    let mut rows = Vec::with_capacity(plan_rows.len());
    let mut sorted: Vec<_> = plan_rows.iter().collect();
    sorted.sort_by(|a, b| {
        a.candidate
            .path
            .as_os_str()
            .as_bytes()
            .cmp(b.candidate.path.as_os_str().as_bytes())
    });
    for (line, row) in canonical.lines().zip(sorted) {
        let prefix = line
            .strip_suffix('}')
            .ok_or_else(|| std::io::Error::other("invalid canonical row"))?;
        rows.push(format!(
            "{prefix},\"path\":{},\"status\":\"planned\"}}",
            json_string(&row.candidate.path.to_string_lossy())
        ));
    }
    Ok(rows)
}

/// Compare a stored plan with the current plan, streaming the saved rows.
pub fn matches_file(plan: &Plan, path: &std::path::Path, id: &str) -> std::io::Result<bool> {
    use crate::json::{self, Value};

    let mut input = open_plan(path)?;
    let header = read_plan_header(&mut input)?;
    let root = base64(plan.root.as_os_str().as_bytes());
    if header.integer("schema").ok() != Some(1)
        || (header.member("row_count").is_ok()
            && header.integer("row_count").ok() != Some(plan.rows.len() as u64))
        || [
            ("plan_id", id),
            ("plan_hash", &plan.hash),
            ("root_raw_b64", &root),
        ]
        .iter()
        .any(|(key, expected)| header.member(key).ok() != Some(&Value::String((*expected).into())))
    {
        return Ok(false);
    }

    let mut bytes = Vec::new();
    for (index, row) in document_rows(&plan.rows)?.iter().enumerate() {
        if index != 0 && !consume(&mut input, b",", &mut bytes)? {
            return Ok(false);
        }
        if !consume(&mut input, row.as_bytes(), &mut bytes)? {
            return Ok(false);
        }
    }
    if !consume(&mut input, b"],\"skipped\":[", &mut bytes)? {
        return Ok(false);
    }

    // Skipped entries are descriptive and can differ from today's scan. Check
    // their syntax and the document end without materializing the whole array.
    let mut next = [0];
    input.read_exact(&mut next)?;
    if next != [b']'] {
        loop {
            if next != [b'{'] {
                return Ok(false);
            }
            bytes.clear();
            bytes.push(b'{');
            let mut quoted = false;
            let mut escaped = false;
            loop {
                input.read_exact(&mut next)?;
                bytes.push(next[0]);
                if bytes.len() > json::MAX_BYTES {
                    return Ok(false);
                }
                if escaped {
                    escaped = false;
                } else if quoted && next == [b'\\'] {
                    escaped = true;
                } else if next == [b'"'] {
                    quoted = !quoted;
                } else if !quoted && next == [b'}'] {
                    break;
                }
            }
            json::parse(&bytes).map_err(std::io::Error::other)?;
            input.read_exact(&mut next)?;
            match next[0] {
                b']' => break,
                b',' => input.read_exact(&mut next)?,
                _ => return Ok(false),
            }
        }
    }
    if !consume(&mut input, b"}", &mut bytes)? {
        return Ok(false);
    }
    loop {
        match input.read(&mut next)? {
            0 => return Ok(true),
            _ if next[0].is_ascii_whitespace() => {}
            _ => return Ok(false),
        }
    }
}

fn consume(input: &mut impl Read, expected: &[u8], bytes: &mut Vec<u8>) -> std::io::Result<bool> {
    bytes.resize(expected.len(), 0);
    input.read_exact(bytes)?;
    Ok(bytes == expected)
}

fn open_plan(path: &std::path::Path) -> std::io::Result<std::io::BufReader<std::fs::File>> {
    use std::os::unix::fs::OpenOptionsExt;
    let file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(crate::staging::SOURCE_FLAGS)
        .open(path)?;
    if !file.metadata()?.is_file() {
        return Err(std::io::Error::other("expected regular plan file"));
    }
    Ok(std::io::BufReader::new(file))
}

/// Read plan metadata without loading candidate rows. Older plan records
/// without a count use the original bounded JSON reader.
pub fn read_header(path: &std::path::Path) -> std::io::Result<crate::json::Value> {
    use crate::json::{self, Value};
    use std::io::Error;
    let mut input = open_plan(path)?;
    let mut header = read_plan_header(&mut input)?;
    if header.member("row_count").is_err() {
        let mut bytes = b"{\"rows\":[".to_vec();
        input.take(json::MAX_BYTES as u64).read_to_end(&mut bytes)?;
        let tail = json::parse(&bytes).map_err(Error::other)?;
        let Value::Array(rows) = tail.member("rows").map_err(Error::other)? else {
            return Err(Error::other("expected plan rows"));
        };
        if let Value::Object(fields) = &mut header {
            fields.insert("row_count".into(), Value::Number(rows.len().to_string()));
        }
    }
    Ok(header)
}

fn read_plan_header(input: &mut impl Read) -> std::io::Result<crate::json::Value> {
    use crate::json;
    let mut prefix = Vec::new();
    let delimiter = b",\"rows\":[";
    let mut byte = [0];
    while prefix.len() < json::MAX_BYTES && input.read(&mut byte)? != 0 {
        prefix.push(byte[0]);
        if prefix.ends_with(delimiter) {
            prefix.truncate(prefix.len() - delimiter.len());
            prefix.push(b'}');
            return json::parse(&prefix).map_err(std::io::Error::other);
        }
    }
    Err(std::io::Error::other("missing plan header"))
}

/// Render a timestamp in the UTC form used by plan and history records.
pub fn format_timestamp(time: std::time::SystemTime) -> std::io::Result<String> {
    let duration = time
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| std::io::Error::other("clock before Unix epoch"))?;
    Ok(timestamp(duration.as_secs(), duration.subsec_nanos()))
}

pub(crate) fn timestamp(seconds: u64, nanos: u32) -> String {
    let days = (seconds / 86_400) as i64 + 719_468;
    let era = days / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36524 - day_of_era / 146096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    let time = seconds % 86_400;
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{nanos:09}Z",
        time / 3600,
        (time % 3600) / 60,
        time % 60
    )
}

/// Publish an immutable plan using create-new temp and a no-overwrite hard link.
pub fn publish(
    plans_dir: &std::path::Path,
    plan: &Plan,
    created: std::time::SystemTime,
) -> std::io::Result<std::path::PathBuf> {
    use std::io::Write;
    use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
    fn private_dir(path: &std::path::Path) -> std::io::Result<()> {
        if path.is_dir() {
            return Ok(());
        }
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            private_dir(parent)?;
        }
        match std::fs::DirBuilder::new().mode(0o700).create(path) {
            Ok(()) => std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists && path.is_dir() => Ok(()),
            Err(e) => Err(e),
        }
    }
    if plan.id.len() != 32
        || !plan
            .id
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(std::io::Error::other("invalid plan id"));
    }
    let bytes = document(plan, created)?;
    private_dir(plans_dir)?;
    let final_path = plans_dir.join(format!("{}.json", plan.id));
    let temp = plans_dir.join(format!("{}.json.tmp", plan.id));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&temp)?;
    let result = (|| {
        file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
        file.write_all(bytes.as_bytes())?;
        file.sync_all()?;
        std::fs::hard_link(&temp, &final_path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                std::io::Error::other("plan_id_collision")
            } else {
                e
            }
        })?;
        std::fs::remove_file(&temp)?;
        std::fs::File::open(plans_dir)?.sync_all()?;
        Ok(final_path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stability::Identity;
    use std::path::PathBuf;
    fn row() -> Row {
        Row {
            candidate: Candidate {
                path: PathBuf::from("x.png"),
                identity: Identity {
                    dev: 2,
                    ino: 3,
                    size: 4,
                    mtime_ns: 5,
                    links: 1,
                },
            },
            argv: vec!["--".into(), "x.png".into()],
        }
    }
    #[test]
    fn verifies_a_large_saved_plan_and_rejects_a_truncated_tail() {
        let id = plan_id().unwrap();
        let path = std::env::temp_dir().join(format!("auto-tzim-plan-{id}.json"));
        struct Fixture(PathBuf);
        impl Drop for Fixture {
            fn drop(&mut self) {
                let _ = std::fs::remove_file(&self.0);
            }
        }
        let fixture = Fixture(path);
        let rows = (0..8_000)
            .map(|index| {
                let path = format!("image-{index:04}.png");
                let mut row = row();
                row.candidate.path = path.clone().into();
                row.argv[1] = path.into();
                row
            })
            .collect::<Vec<_>>();
        let plan = Plan {
            id: id.clone(),
            root: "/example".into(),
            hash: plan_hash(&rows),
            rows,
            discovery: crate::discovery::Discovery {
                skipped: vec![crate::discovery::Skip {
                    path: "odd}name.png".into(),
                    reason: "skip \"quoted\"".into(),
                }],
                ..Default::default()
            },
        };
        let document = document(&plan, std::time::UNIX_EPOCH).unwrap();
        assert!(document.len() > crate::json::MAX_BYTES);
        std::fs::write(&fixture.0, &document).unwrap();
        assert!(matches_file(&plan, &fixture.0, &id).unwrap());
        assert_eq!(
            read_header(&fixture.0)
                .unwrap()
                .integer("row_count")
                .unwrap(),
            8_000
        );
        std::fs::write(&fixture.0, &document[..document.len() - 3]).unwrap();
        assert!(!matches_file(&plan, &fixture.0, &id).unwrap_or(false));
    }

    #[test]
    fn profile_arguments_are_relative_sorted_and_support_non_utf8() {
        use std::os::unix::ffi::OsStringExt;
        let mut candidate = row().candidate;
        candidate.path = std::ffi::OsString::from_vec(b"bad\xff.png".to_vec()).into();
        let folder = crate::config::Folder {
            formats: vec!["png".into(), "gzip".into()],
            ..crate::config::Folder::default()
        };
        let args = arguments(&folder, &candidate);
        assert_eq!(args[..3], ["--formats", "gzip,png", "--"]);
        assert_eq!(args[3].as_bytes(), b"bad\xff.png");
        let row = Row {
            candidate,
            argv: args,
        };
        assert!(canonical_rows(&[row]).contains("{\"b64\":\"YmFk/y5wbmc=\"}"));
    }

    #[test]
    fn serialization_and_hash_depend_on_identity_and_argv() {
        let original = row();
        assert_eq!(canonical_rows(std::slice::from_ref(&original)), "{\"path_raw_b64\":\"eC5wbmc=\",\"size\":4,\"dev\":2,\"ino\":3,\"mtime_ns\":5,\"argv\":[\"--\",\"x.png\"]}\n");
        let base = plan_hash(std::slice::from_ref(&original));
        let mut changed = original.clone();
        changed.candidate.identity.mtime_ns += 1;
        assert_ne!(base, plan_hash(&[changed]));
        let mut changed = original;
        changed.argv.push("--new".into());
        assert_ne!(base, plan_hash(&[changed]));
        assert_ne!(plan_id().unwrap(), plan_id().unwrap());
    }
    #[test]
    fn in_memory_plans_are_per_root_ordered_and_do_not_depend_on_batch() {
        let a = row().candidate;
        let mut b = a.clone();
        b.path = "a.png".into();
        let found = || crate::discovery::Discovery {
            candidates: vec![a.clone(), b.clone()],
            ..crate::discovery::Discovery::default()
        };
        let folder = |path: &str| crate::config::Folder {
            root: path.into(),
            ..crate::config::Folder::default()
        };
        let argument = |_: &crate::config::Folder, _: &Candidate| Ok(vec!["test-only".into()]);
        let alone = build_plans(vec![(folder("/a"), found())], argument).unwrap();
        let batch = build_plans(
            vec![(folder("/a"), found()), (folder("/b"), found())],
            argument,
        )
        .unwrap();
        assert_eq!(batch.len(), 2);
        assert_eq!(batch[0].root, PathBuf::from("/a"));
        assert_eq!(batch[1].root, PathBuf::from("/b"));
        assert_ne!(batch[0].id, batch[1].id);
        assert_ne!(alone[0].id, batch[0].id);
        assert_eq!(alone[0].hash, batch[0].hash);
        assert_eq!(
            canonical_rows(&alone[0].rows),
            canonical_rows(&batch[0].rows)
        );
        assert_eq!(batch[0].rows[0].candidate.path, PathBuf::from("a.png"));
        assert_eq!(batch[0].rows[1].candidate.path, PathBuf::from("x.png"));
        let failed = build_plans(vec![(folder("/a"), found())], |_, _| {
            Err("argument refusal".into())
        });
        assert!(matches!(failed, Err(BuildError::Arguments(_))));
    }

    #[test]
    fn id_collision_and_entropy_errors_fail_closed() {
        let roots = || {
            vec![
                (
                    crate::config::Folder::default(),
                    crate::discovery::Discovery::default(),
                ),
                (
                    crate::config::Folder::default(),
                    crate::discovery::Discovery::default(),
                ),
            ]
        };
        let mut calls = 0;
        let error = build_plans_with_ids(
            roots(),
            |_, _| Ok(Vec::new()),
            || {
                calls += 1;
                Ok("same".into())
            },
        )
        .unwrap_err();
        assert_eq!(calls, 2);
        assert!(matches!(error, BuildError::Entropy(_)));
        let error = build_plans_with_ids(
            roots(),
            |_, _| Ok(Vec::new()),
            || Err(std::io::Error::other("unavailable")),
        )
        .unwrap_err();
        assert!(matches!(error, BuildError::Entropy(_)));
    }

    #[test]
    fn actual_mtime_changes_hash_without_changing_size() {
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::time::{Duration, SystemTime};
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "atz-hash-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        let root = std::fs::canonicalize(root).unwrap();
        let path = root.join("file");
        std::fs::write(&path, b"same size").unwrap();
        let profile = crate::config::Folder {
            min_file_age_seconds: 0,
            ..crate::config::Folder::default()
        };
        let now = SystemTime::now() + Duration::from_secs(60);
        let scan = || {
            crate::discovery::walk(&root, &profile, now)
                .unwrap()
                .candidates
                .into_iter()
                .map(|candidate| Row {
                    candidate,
                    argv: vec!["--".into(), "file".into()],
                })
                .collect::<Vec<_>>()
        };
        let first = scan();
        assert_eq!(first.len(), 1);
        let original = plan_hash(&first);
        assert_eq!(original, plan_hash(&scan()));
        let file = std::fs::OpenOptions::new().write(true).open(&path).unwrap();
        file.set_modified(SystemTime::now() - Duration::from_secs(10))
            .unwrap();
        let second = scan();
        assert_eq!(
            first[0].candidate.identity.size,
            second[0].candidate.identity.size
        );
        assert_ne!(original, plan_hash(&second));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn every_hashed_identity_component_changes_digest() {
        let original = row();
        let original_hash = plan_hash(std::slice::from_ref(&original));
        let mut variants = Vec::new();
        let mut changed = original.clone();
        changed.candidate.identity.dev += 1;
        variants.push(changed);
        let mut changed = original.clone();
        changed.candidate.identity.ino += 1;
        variants.push(changed);
        let mut changed = original.clone();
        changed.candidate.identity.size += 1;
        variants.push(changed);
        let mut changed = original.clone();
        changed.candidate.identity.mtime_ns += 1;
        variants.push(changed);
        let mut changed = original.clone();
        changed.candidate.path = "different.png".into();
        variants.push(changed);
        for changed in variants {
            assert_ne!(original_hash, plan_hash(&[changed]));
        }
        let mut changed = original;
        changed.candidate.identity.links += 1;
        assert_eq!(original_hash, plan_hash(&[changed]));
    }

    #[test]
    fn rows_sort_by_raw_bytes_and_preserve_non_utf8_paths() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;
        let a = row();
        let mut b = row();
        b.candidate.path = PathBuf::from(OsString::from_vec(vec![b'x', 0xff]));
        let forwards = canonical_rows(&[a.clone(), b.clone()]);
        let backwards = canonical_rows(&[b.clone(), a.clone()]);
        assert_eq!(forwards, backwards);
        assert!(forwards.contains("\"path_raw_b64\":\"eP8=\""));
        assert_eq!(plan_hash(&[a.clone(), b.clone()]), plan_hash(&[b, a]));
    }

    #[test]
    fn encoding() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(json_string("a\n\"\\"), "\"a\\u000a\\\"\\\\\"");
    }
}
