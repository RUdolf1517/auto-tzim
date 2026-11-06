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
    walk_with_order(root, profile, now, false)
}

// Test injection simulates a hostile filesystem enumerating entries in reverse.
fn walk_with_order(
    root: &Path,
    profile: &Folder,
    now: SystemTime,
    reverse: bool,
) -> Result<Discovery, String> {
    let metadata = std::fs::symlink_metadata(root).map_err(|e| format!("root: {e}"))?;
    let root_dev = Identity::from_metadata(&metadata).dev;
    let mut result = Discovery::default();
    let options = WalkOptions {
        root_dev,
        profile,
        now,
        reverse,
    };
    visit(root, Path::new(""), 0, &options, &mut result)?;
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

struct WalkOptions<'a> {
    root_dev: u64,
    profile: &'a Folder,
    now: SystemTime,
    reverse: bool,
}

fn visit(
    root: &Path,
    relative: &Path,
    depth: usize,
    options: &WalkOptions<'_>,
    out: &mut Discovery,
) -> Result<(), String> {
    let WalkOptions {
        root_dev,
        profile,
        now,
        reverse,
    } = *options;
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
    if reverse {
        entries.sort_by(|a, b| b.file_name().as_bytes().cmp(a.file_name().as_bytes()));
    }
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
                || (file_type.is_dir()
                    && matches!(
                        (pattern.as_str(), name_bytes),
                        (".git/**", b".git") | (".hg/**", b".hg") | (".svn/**", b".svn")
                    ))
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
            visit(root, &path, depth + 1, options, out)?;
        } else if file_type.is_file() {
            out.files_seen += 1;
            check_parent(root, &full)?;
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

// A parent outside the canonical root is a hard abort, never a skipped row.
fn check_parent(root: &Path, full: &Path) -> Result<(), String> {
    let parent = full
        .parent()
        .ok_or_else(|| "candidate has no parent".to_owned())?;
    let resolved =
        std::fs::canonicalize(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    if !resolved.starts_with(root) {
        return Err(format!("containment violation: {}", full.display()));
    }
    Ok(())
}

// Simple glob over path bytes: '*' stays within a component, '**' may cross '/'.
fn glob(pattern: &[u8], path: &[u8]) -> bool {
    // Dynamic programming bounds work by pattern length times path length,
    // rather than recursively backtracking over adversarial wildcard patterns.
    let mut table = vec![vec![false; path.len() + 1]; pattern.len() + 1];
    table[pattern.len()][path.len()] = true;
    for i in (0..pattern.len()).rev() {
        for j in (0..=path.len()).rev() {
            if pattern[i] == b'*' {
                let double = pattern.get(i + 1) == Some(&b'*');
                let next = i + if double { 2 } else { 1 };
                table[i][j] = table[next][j]
                    || (double && pattern.get(next) == Some(&b'/') && table[next + 1][j])
                    || (j < path.len() && (double || path[j] != b'/') && table[i][j + 1]);
            } else if j < path.len()
                && (pattern[i] == path[j] || (pattern[i] == b'?' && path[j] != b'/'))
            {
                table[i][j] = table[i + 1][j + 1];
            }
        }
    }
    table[0][0]
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

#[cfg(test)]
mod ordering_and_containment_tests {
    use super::*;
    use std::os::unix::fs::symlink;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn reverse_enumeration_and_escaping_parents() {
        let base = std::env::temp_dir().join(format!(
            "atz-order-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let root = base.join("root");
        let outside = base.join("escape");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        let root = std::fs::canonicalize(root).unwrap();
        for name in ["z", "A", "a"] {
            std::fs::write(root.join(name), b"x").unwrap();
        }
        std::fs::write(outside.join("file"), b"outside").unwrap();
        symlink(&outside, root.join("external-dir")).unwrap();
        symlink("/etc/hostname", root.join("external-file")).unwrap();
        let profile = Folder {
            min_file_age_seconds: 0,
            ..Folder::default()
        };
        let now = SystemTime::now();
        let normal = walk_with_order(&root, &profile, now, false).unwrap();
        let reverse = walk_with_order(&root, &profile, now, true).unwrap();
        assert_eq!(normal.candidates, reverse.candidates);
        assert_eq!(normal.skipped, reverse.skipped);
        assert_eq!(
            normal
                .candidates
                .iter()
                .map(|c| c.path.as_os_str().as_bytes())
                .collect::<Vec<_>>(),
            [b"A".as_slice(), b"a", b"z"]
        );
        assert_eq!(
            normal
                .skipped
                .iter()
                .filter(|s| s.reason == "skipped_policy: symlink")
                .count(),
            2
        );
        assert!(check_parent(&root, &root.join("../escape/file"))
            .unwrap_err()
            .contains("containment violation"));
        assert!(check_parent(&root, &root.join("external-dir/file"))
            .unwrap_err()
            .contains("containment violation"));
        std::fs::remove_dir_all(base).unwrap();
    }
}
