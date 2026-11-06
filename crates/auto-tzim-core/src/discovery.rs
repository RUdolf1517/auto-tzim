//! Read-only, non-following discovery of one already approved root.
use crate::config::Folder;
use crate::stability::{self, Identity};
use std::collections::BTreeMap;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// A regular candidate, whose metadata was captured by lstat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub path: PathBuf,
    pub identity: Identity,
}

/// A refusal or skipped entry; never treated as an eligible plan row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skip {
    pub path: PathBuf,
    pub reason: String,
}

/// Discovery results and observable scope.
#[derive(Debug, Default)]
pub struct Discovery {
    pub candidates: Vec<Candidate>,
    pub skipped: Vec<Skip>,
    pub directories: u64,
    pub max_depth: usize,
    pub files_seen: u64,
    pub limits: BTreeMap<String, u64>,
}

/// Walk a canonical root without opening candidate files.
pub fn walk(root: &Path, profile: &Folder, now: SystemTime) -> Result<Discovery, String> {
    let metadata = std::fs::symlink_metadata(root).map_err(|e| format!("root: {e}"))?;
    let root_dev = Identity::from_metadata(&metadata).dev;
    let mut result = Discovery::default();
    visit(root, Path::new(""), 0, root_dev, profile, now, &mut result)?;
    result.candidates.sort_by(|a, b| {
        a.path
            .as_os_str()
            .as_bytes()
            .cmp(b.path.as_os_str().as_bytes())
    });
    result.skipped.sort_by(|a, b| {
        a.path
            .as_os_str()
            .as_bytes()
            .cmp(b.path.as_os_str().as_bytes())
    });
    Ok(result)
}

fn limit(result: &mut Discovery, key: String) {
    *result.limits.entry(key).or_default() += 1;
}

fn visit(
    root: &Path,
    relative: &Path,
    depth: usize,
    root_dev: u64,
    profile: &Folder,
    now: SystemTime,
    out: &mut Discovery,
) -> Result<(), String> {
    let dir = root.join(relative);
    let canonical = std::fs::canonicalize(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    if !canonical.starts_with(root) {
        return Err(format!("containment violation: {}", dir.display()));
    }
    out.directories += 1;
    out.max_depth = out.max_depth.max(depth);
    let mut entries = std::fs::read_dir(&dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("{}: {e}", dir.display()))?;
    entries.sort_by(|a, b| a.file_name().as_bytes().cmp(b.file_name().as_bytes()));
    for entry in entries {
        let name = entry.file_name();
        let name_bytes = name.as_bytes();
        let path = relative.join(&name);
        let full = root.join(&path);
        let metadata =
            std::fs::symlink_metadata(&full).map_err(|e| format!("{}: {e}", full.display()))?;
        let file_type = metadata.file_type();
        let raw = path.as_os_str().as_bytes();
        if name_bytes.starts_with(b".tzim-") && name_bytes.ends_with(b".tmp") {
            out.skipped.push(Skip {
                path,
                reason: "orphan_temp".into(),
            });
            continue;
        }
        let exclusion = profile.exclude_globs.iter().find(|pattern| {
            glob(pattern.as_bytes(), raw)
                || (file_type.is_dir() && glob(pattern.as_bytes(), &[raw, b"/"].concat()))
        });
        if let Some(pattern) = exclusion {
            if file_type.is_dir() {
                limit(out, format!("excluded by {pattern}"));
            }
            out.skipped.push(Skip {
                path,
                reason: format!("excluded by {pattern}"),
            });
            continue;
        }
        if !profile.include_hidden && name_bytes.starts_with(b".") {
            continue;
        }
        if file_type.is_symlink() {
            out.skipped.push(Skip {
                path,
                reason: "skipped_policy: symlink".into(),
            });
        } else if file_type.is_dir() {
            if !profile.recursive {
                limit(out, "non-recursive".into());
                continue;
            }
            if depth >= 64 {
                limit(out, "depth 64".into());
                out.skipped.push(Skip {
                    path,
                    reason: "depth_exceeded".into(),
                });
                continue;
            }
            if !profile.cross_filesystems && Identity::from_metadata(&metadata).dev != root_dev {
                limit(out, "mount point".into());
                out.skipped.push(Skip {
                    path,
                    reason: "mount_point".into(),
                });
                continue;
            }
            visit(root, &path, depth + 1, root_dev, profile, now, out)?;
        } else if file_type.is_file() {
            out.files_seen += 1;
            // Recheck the parent after lstat; an escaping swap aborts the whole walk.
            let parent = full
                .parent()
                .ok_or_else(|| "candidate has no parent".to_owned())?;
            let resolved =
                std::fs::canonicalize(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
            if !resolved.starts_with(root) {
                return Err(format!("containment violation: {}", full.display()));
            }
            let identity = Identity::from_metadata(&metadata);
            if let Some(reason) = stability::refusal(&identity, now, profile.min_file_age_seconds) {
                out.skipped.push(Skip {
                    path,
                    reason: reason.into(),
                });
            } else {
                out.candidates.push(Candidate { path, identity });
            }
        } else {
            out.skipped.push(Skip {
                path,
                reason: "skipped_policy: special file".into(),
            });
        }
    }
    Ok(())
}

// Simple glob over path bytes: '*' stays within a component, '**' may cross '/'.
fn glob(pattern: &[u8], path: &[u8]) -> bool {
    fn match_at(p: &[u8], s: &[u8]) -> bool {
        if p.is_empty() {
            return s.is_empty();
        }
        if p[0] == b'*' {
            let double = p.get(1) == Some(&b'*');
            let rest = &p[if double { 2 } else { 1 }..];
            if double && rest.first() == Some(&b'/') && match_at(&rest[1..], s) {
                return true;
            }
            if match_at(rest, s) {
                return true;
            }
            return !s.is_empty() && (double || s[0] != b'/') && match_at(p, &s[1..]);
        }
        if !s.is_empty() && (p[0] == s[0] || (p[0] == b'?' && s[0] != b'/')) {
            return match_at(&p[1..], &s[1..]);
        }
        false
    }
    match_at(pattern, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn glob_boundaries() {
        assert!(glob(b".git/**", b".git/"));
        assert!(glob(b"**/*.png", b"nested/a.png"));
        assert!(glob(b"**/*.png", b"a.png"));
        assert!(!glob(b"*.png", b"nested/a.png"));
    }
}

#[cfg(test)]
mod walk_tests {
    use super::*;
    use std::os::unix::fs::symlink;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "atz-discovery-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir_all(&path).unwrap();
            Self(std::fs::canonicalize(path).unwrap())
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn recursive_sorted_excludes_and_non_recursive_coverage() {
        let root = Fixture::new();
        std::fs::create_dir(root.0.join("nested")).unwrap();
        std::fs::create_dir(root.0.join(".git")).unwrap();
        for path in ["z", "a", "nested/x", ".git/hidden", ".tzim-1-2-3.tmp"] {
            std::fs::write(root.0.join(path), "data").unwrap();
        }
        symlink("nested", root.0.join("link")).unwrap();
        let profile = Folder {
            min_file_age_seconds: 0,
            ..Folder::default()
        };
        let result = walk(&root.0, &profile, SystemTime::now()).unwrap();
        assert_eq!(
            result
                .candidates
                .iter()
                .map(|x| x.path.as_os_str().as_bytes())
                .collect::<Vec<_>>(),
            [b"a".as_slice(), b"nested/x", b"z"]
        );
        assert_eq!(result.directories, 2);
        assert_eq!(result.max_depth, 1);
        assert_eq!(result.limits.get("excluded by .git/**"), Some(&1));
        assert!(result.skipped.iter().any(|s| s.reason == "orphan_temp"));
        assert!(result.skipped.iter().any(|s| s.reason.contains("symlink")));
        let shallow = Folder {
            recursive: false,
            ..profile
        };
        let result = walk(&root.0, &shallow, SystemTime::now()).unwrap();
        assert_eq!(result.candidates.len(), 2);
        assert_eq!(result.limits.get("non-recursive"), Some(&1));
    }
}
