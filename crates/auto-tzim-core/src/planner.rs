//! Deterministic canonical plan rows and per-scan random identifiers.
use crate::discovery::Candidate;
use crate::hash;
use std::io::Read;
use std::os::unix::ffi::OsStrExt;

/// A plan row and the exact argv determined by the caller's adapter contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub candidate: Candidate,
    pub argv: Vec<String>,
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
            .map(|item| json_string(item))
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
    fn encoding() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(json_string("a\n\"\\"), "\"a\\u000a\\\"\\\\\"");
    }
}
