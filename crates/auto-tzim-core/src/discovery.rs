//! Read-only, non-following discovery of one already approved root.
use crate::config::Folder;
use crate::stability::{self, Identity};
use std::collections::BTreeMap;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::FileTypeExt;
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
    /// Diagnostics not represented as plan rows, such as orphan temps.
    pub diagnostics: Vec<Skip>,
    pub directories: u64,
    pub max_depth: usize,
    pub files_seen: u64,
    pub limits: BTreeMap<String, u64>,
    /// Files filtered before stability, including hidden or excluded files.
    pub filtered_files: BTreeMap<String, u64>,
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
    walk_with_hook_and_order(root, profile, now, reverse, &|_| {})
}

/// Inject a callback between enumeration and directory identity recheck.
pub fn walk_with_hook(
    root: &Path,
    profile: &Folder,
    now: SystemTime,
    hook: &dyn Fn(&Path),
) -> Result<Discovery, String> {
    walk_with_hook_and_order(root, profile, now, false, hook)
}

fn walk_with_hook_and_order(
    root: &Path,
    profile: &Folder,
    now: SystemTime,
    reverse: bool,
    hook: &dyn Fn(&Path),
) -> Result<Discovery, String> {
    let metadata = std::fs::symlink_metadata(root).map_err(|e| format!("root: {e}"))?;
    if !metadata.file_type().is_dir() {
        return Err(format!("root is not a directory: {}", root.display()));
    }
    let root_identity = Identity::from_metadata(&metadata);
    let root_dev = root_identity.dev;
    let mut result = Discovery::default();
    let options = WalkOptions {
        root_dev,
        profile,
        now,
        reverse,
        hook,
    };
    visit(
        root,
        Path::new(""),
        0,
        &root_identity,
        &options,
        &mut result,
    )?;
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
    hook: &'a dyn Fn(&Path),
}

fn visit(
    root: &Path,
    relative: &Path,
    depth: usize,
    expected: &Identity,
    options: &WalkOptions<'_>,
    out: &mut Discovery,
) -> Result<(), String> {
    let WalkOptions {
        root_dev,
        profile,
        now,
        reverse,
        hook,
    } = *options;
    let dir = root.join(relative);
    check_directory(root, &dir, expected)?;
    out.directories += 1;
    out.max_depth = out.max_depth.max(depth);
    let mut entries = std::fs::read_dir(&dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("{}: {e}", dir.display()))?;
    // Detect many swaps that occurred during enumeration. This is not an
    // atomic no-follow guarantee: an attacker can still replace and restore it.
    hook(&dir);
    check_directory(root, &dir, expected)?;
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
        if file_type.is_file() {
            out.files_seen += 1;
        }
        if name_bytes.starts_with(b".tzim-") && name_bytes.ends_with(b".tmp") {
            if file_type.is_file() {
                *out.filtered_files.entry("orphan_temp".into()).or_default() += 1;
            }
            if out.diagnostics.is_empty() {
                out.diagnostics.push(Skip {
                    path,
                    reason: "orphan_temp".into(),
                });
            }
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
            if file_type.is_file() {
                *out.filtered_files
                    .entry(format!("excluded by {pattern}"))
                    .or_default() += 1;
            }
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
            if file_type.is_file() {
                *out.filtered_files.entry("hidden".into()).or_default() += 1;
            }
            if file_type.is_dir() {
                limit(out, "hidden directory".into());
            }
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
                out.skipped.push(Skip {
                    path,
                    reason: "skipped_policy: non-recursive directory".into(),
                });
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
                    reason: "skipped_policy: mount point".into(),
                });
                continue;
            }
            let identity = Identity::from_metadata(&metadata);
            visit(root, &path, depth + 1, &identity, options, out)?;
            out.skipped.push(Skip {
                path,
                reason: "skipped_policy: directory".into(),
            });
        } else if file_type.is_file() {
            let identity = Identity::from_metadata(&metadata);
            if !profile.cross_filesystems && identity.dev != root_dev {
                *out.filtered_files.entry("mount point".into()).or_default() += 1;
                out.skipped.push(Skip {
                    path,
                    reason: "skipped_policy: mount point".into(),
                });
                continue;
            }
            hook(&full);
            check_parent(root, &full)?;
            let current = std::fs::symlink_metadata(&full)
                .map_err(|e| format!("tree_changed: {}: {e}", full.display()))?;
            if !current.file_type().is_file() || Identity::from_metadata(&current) != identity {
                return Err(format!(
                    "tree_changed: file changed during scan: {}",
                    full.display()
                ));
            }
            if let Some(reason) = stability::refusal(&identity, now, profile.min_file_age_seconds) {
                out.skipped.push(Skip {
                    path,
                    reason: reason.into(),
                });
            } else if !format_selected(name_bytes, &profile.formats) {
                *out.filtered_files
                    .entry("format_not_selected".into())
                    .or_default() += 1;
                out.skipped.push(Skip {
                    path,
                    reason: "skipped_policy: format_not_selected".into(),
                });
            } else {
                out.candidates.push(Candidate { path, identity });
            }
        } else {
            let kind = if file_type.is_fifo() {
                "fifo"
            } else if file_type.is_socket() {
                "socket"
            } else if file_type.is_char_device() {
                "character device"
            } else if file_type.is_block_device() {
                "block device"
            } else {
                "special file"
            };
            out.skipped.push(Skip {
                path,
                reason: format!("skipped_policy: {kind}"),
            });
        }
    }
    Ok(())
}

fn format_selected(name: &[u8], formats: &[String]) -> bool {
    if formats.is_empty() {
        return true;
    }
    let Some(dot) = name.iter().rposition(|&byte| byte == b'.') else {
        return false;
    };
    let extension = name[dot + 1..].to_ascii_lowercase();
    let kind = match extension.as_slice() {
        b"png" => "png",
        b"gz" => "gzip",
        b"zip" => "zip",
        b"ico" => "ico",
        b"jpg" | b"jpeg" => "jpeg",
        b"tif" | b"tiff" => "tiff",
        b"zlib" => "zlib",
        _ => return false,
    };
    formats.iter().any(|format| format == kind)
}

// Best-effort race detection only: path-based std APIs cannot make this check
// atomic with read_dir. An authoritative no-follow primitive is still needed.
fn check_directory(root: &Path, dir: &Path, expected: &Identity) -> Result<(), String> {
    let metadata = std::fs::symlink_metadata(dir)
        .map_err(|e| format!("tree_changed: {}: {e}", dir.display()))?;
    let current = Identity::from_metadata(&metadata);
    if !metadata.file_type().is_dir() || current.dev != expected.dev || current.ino != expected.ino
    {
        return Err(format!(
            "tree_changed: directory changed during scan: {}",
            dir.display()
        ));
    }
    let canonical =
        std::fs::canonicalize(dir).map_err(|e| format!("tree_changed: {}: {e}", dir.display()))?;
    if !canonical.starts_with(root) {
        return Err(format!(
            "tree_changed: containment violation: {}",
            dir.display()
        ));
    }
    if canonical != dir {
        return Err(format!(
            "tree_changed: directory path changed during scan: {}",
            dir.display()
        ));
    }
    Ok(())
}

// A parent outside the canonical root is a hard abort, never a skipped row.
fn check_parent(root: &Path, full: &Path) -> Result<(), String> {
    let parent = full
        .parent()
        .ok_or_else(|| "candidate has no parent".to_owned())?;
    let resolved = std::fs::canonicalize(parent)
        .map_err(|e| format!("tree_changed: {}: {e}", parent.display()))?;
    if !resolved.starts_with(root) {
        return Err(format!(
            "tree_changed: containment violation: {}",
            full.display()
        ));
    }
    if resolved != parent {
        return Err(format!(
            "tree_changed: parent changed during scan: {}",
            full.display()
        ));
    }
    Ok(())
}

// Simple glob over path bytes: '*' stays within a component, '**' may cross '/'.
fn glob(pattern: &[u8], path: &[u8]) -> bool {
    // Each recurrence needs only the following two pattern rows. Bound
    // memory by path length even for a very long user-supplied pattern.
    let mut one = vec![false; path.len() + 1];
    let mut two = one.clone();
    one[path.len()] = true;
    for i in (0..pattern.len()).rev() {
        let mut current = vec![false; path.len() + 1];
        for j in (0..=path.len()).rev() {
            if pattern[i] == b'*' {
                let double = pattern.get(i + 1) == Some(&b'*');
                let rest = if double { &two } else { &one };
                current[j] =
                    rest[j] || (j < path.len() && (double || path[j] != b'/') && current[j + 1]);
            } else if j < path.len()
                && (pattern[i] == path[j] || (pattern[i] == b'?' && path[j] != b'/'))
            {
                current[j] = one[j + 1];
            }
        }
        two = one;
        one = current;
    }
    one[0]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_glob_matches_reference_for_small_patterns() {
        fn reference(p: &[u8], s: &[u8]) -> bool {
            if p.is_empty() {
                return s.is_empty();
            }
            if p[0] == b'*' {
                let double = p.get(1) == Some(&b'*');
                let rest = &p[if double { 2 } else { 1 }..];
                return reference(rest, s)
                    || (!s.is_empty() && (double || s[0] != b'/') && reference(p, &s[1..]));
            }
            !s.is_empty()
                && (p[0] == s[0] || (p[0] == b'?' && s[0] != b'/'))
                && reference(&p[1..], &s[1..])
        }
        fn words(alphabet: &[u8], max: usize) -> Vec<Vec<u8>> {
            let mut result = vec![Vec::new()];
            let mut layer = vec![Vec::new()];
            for _ in 0..max {
                layer = layer
                    .iter()
                    .flat_map(|word| {
                        alphabet.iter().map(move |&ch| {
                            let mut next = word.clone();
                            next.push(ch);
                            next
                        })
                    })
                    .collect();
                result.extend(layer.iter().cloned());
            }
            result
        }
        for pattern in words(b"a/*?", 4) {
            for path in words(b"a/", 4) {
                assert_eq!(
                    glob(&pattern, &path),
                    reference(&pattern, &path),
                    "pattern={pattern:?}, path={path:?}"
                );
            }
        }
    }

    #[test]
    fn extension_policy_is_case_insensitive_and_uses_last_suffix() {
        let formats = vec!["jpeg".into(), "png".into()];
        assert!(format_selected(b"a.JPG", &formats));
        assert!(format_selected(b"a.tar.PnG", &formats));
        assert!(!format_selected(b"a.png.gz", &formats));
        assert!(!format_selected(b"no-extension", &formats));
        assert!(format_selected(b"no-extension", &[]));
    }

    #[test]
    fn glob_boundaries() {
        assert!(glob(b".git/**", b".git/"));
        assert!(glob(b"**/*.png", b"nested/a.png"));
        assert!(!glob(b"**/*.png", b"a.png"));
        assert!(!glob(b"*.png", b"nested/a.png"));
        assert!(!glob(b"a/**/z", b"a/z"));
        assert!(glob(b"a/**/z", b"a/x/y/z"));
        assert!(!glob(b"a/*/z", b"a/x/y/z"));
        assert!(glob(b"", b""));
        assert!(!glob(b"", b"a"));
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
        assert!(result.diagnostics.iter().any(|s| s.reason == "orphan_temp"));
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
    fn directory_replacement_or_symlink_is_detected_before_visit() {
        let base = std::env::temp_dir().join(format!(
            "atz-directory-check-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(base.join("root/child")).unwrap();
        std::fs::create_dir(base.join("root/other")).unwrap();
        let root = std::fs::canonicalize(base.join("root")).unwrap();
        let child = root.join("child");
        let expected = Identity::from_metadata(&std::fs::symlink_metadata(&child).unwrap());
        assert!(check_directory(&root, &child, &expected).is_ok());
        std::fs::rename(&child, root.join("old-child")).unwrap();
        std::fs::create_dir(&child).unwrap();
        assert!(check_directory(&root, &child, &expected)
            .unwrap_err()
            .contains("directory changed"));
        std::fs::remove_dir(&child).unwrap();
        symlink("other", &child).unwrap();
        assert!(check_directory(&root, &child, &expected)
            .unwrap_err()
            .contains("directory changed"));
        std::fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn injected_directory_swap_aborts_before_any_plan_is_published() {
        let base = std::env::temp_dir().join(format!(
            "atz-race-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(base.join("root/child")).unwrap();
        std::fs::create_dir(base.join("outside")).unwrap();
        std::fs::write(base.join("root/child/inside"), b"inside").unwrap();
        std::fs::write(base.join("outside/outside"), b"outside").unwrap();
        let root = std::fs::canonicalize(base.join("root")).unwrap();
        let child = root.join("child");
        let outside = std::fs::canonicalize(base.join("outside")).unwrap();
        let swapped = std::cell::Cell::new(false);
        let result = walk_with_hook(&root, &Folder::default(), SystemTime::now(), &|dir| {
            if dir == child && !swapped.replace(true) {
                std::fs::rename(&child, root.join("old-child")).unwrap();
                symlink(&outside, &child).unwrap();
            }
        });
        assert!(swapped.get());
        assert!(result.unwrap_err().contains("tree_changed"));
        assert!(!base.join("state/plans").exists());
        std::fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn injected_file_swap_for_outside_symlink_is_never_a_candidate() {
        let base = std::env::temp_dir().join(format!(
            "atz-file-race-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(base.join("root")).unwrap();
        std::fs::write(base.join("root/file"), b"inside").unwrap();
        std::fs::write(base.join("outside"), b"outside").unwrap();
        let root = std::fs::canonicalize(base.join("root")).unwrap();
        let swapped = std::cell::Cell::new(false);
        let file = root.join("file");
        let found = walk_with_hook(&root, &Folder::default(), SystemTime::now(), &|path| {
            if path == file && !swapped.replace(true) {
                std::fs::remove_file(&file).unwrap();
                symlink(base.join("outside"), &file).unwrap();
            }
        });
        assert!(swapped.get());
        assert!(found.unwrap_err().contains("tree_changed"));
        assert_eq!(std::fs::read(base.join("outside")).unwrap(), b"outside");
        std::fs::remove_dir_all(base).unwrap();
    }

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
        for name in ["z", "B", "a"] {
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
            [b"B".as_slice(), b"a", b"z"]
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

#[cfg(test)]
mod corpus_tests {
    use super::*;
    use std::os::unix::ffi::OsStringExt;
    use std::os::unix::fs::symlink;
    use std::os::unix::net::UnixListener;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::Duration;
    static NEXT: AtomicU64 = AtomicU64::new(0);

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let p = std::env::temp_dir().join(format!(
                "atz-corpus-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir(&p).unwrap();
            Self(std::fs::canonicalize(p).unwrap())
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn special_files_links_temp_and_injected_age() {
        let f = Fixture::new();
        std::fs::write(f.0.join("first"), b"keep").unwrap();
        std::fs::hard_link(f.0.join("first"), f.0.join("second")).unwrap();
        std::fs::write(f.0.join("young"), b"keep young").unwrap();
        std::fs::write(f.0.join(".tzim-123-456-0.tmp"), b"keep temp").unwrap();
        std::fs::create_dir(f.0.join("inner")).unwrap();
        for (name, target) in [
            ("inside-file", "first"),
            ("outside-file", "/etc/hostname"),
            ("inside-dir", "inner"),
            ("broken", "absent"),
            ("loop-a", "loop-b"),
            ("loop-b", "loop-a"),
        ] {
            symlink(target, f.0.join(name)).unwrap();
        }
        let _listener = UnixListener::bind(f.0.join("socket")).unwrap();
        let profile = Folder {
            include_hidden: true,
            min_file_age_seconds: 3600,
            ..Folder::default()
        };
        let now = SystemTime::now();
        let result = walk(&f.0, &profile, now).unwrap();
        assert_eq!(
            result
                .skipped
                .iter()
                .filter(|s| s.reason == "hardlink_skip")
                .count(),
            2
        );
        assert_eq!(
            result
                .skipped
                .iter()
                .filter(|s| s.reason == "skipped_unstable")
                .count(),
            1
        );
        assert_eq!(
            result
                .skipped
                .iter()
                .filter(|s| s.reason == "skipped_policy: symlink")
                .count(),
            6
        );
        assert!(result
            .skipped
            .iter()
            .any(|s| s.path == Path::new("socket") && s.reason == "skipped_policy: socket"));
        assert_eq!(
            result
                .diagnostics
                .iter()
                .filter(|s| s.reason == "orphan_temp")
                .count(),
            1
        );
        assert!(result.candidates.is_empty());
        let later = walk(&f.0, &profile, now + Duration::from_secs(3601)).unwrap();
        assert_eq!(
            later.candidates.iter().map(|c| &c.path).collect::<Vec<_>>(),
            [Path::new("young")]
        );
        assert_eq!(
            std::fs::read(f.0.join(".tzim-123-456-0.tmp")).unwrap(),
            b"keep temp"
        );
        assert_eq!(std::fs::read(f.0.join("first")).unwrap(), b"keep");
    }

    #[test]
    fn format_filter_follows_hardlink_and_age_gates() {
        let f = Fixture::new();
        for name in ["good.PnG", "wrong.gz", "hard.gz", ".tzim-1.tmp"] {
            std::fs::write(f.0.join(name), b"not an image").unwrap();
        }
        std::fs::hard_link(f.0.join("hard.gz"), f.0.join("hard2.gz")).unwrap();
        std::fs::hard_link(f.0.join(".tzim-1.tmp"), f.0.join(".tzim-2.tmp")).unwrap();
        let profile = Folder {
            formats: vec!["png".into()],
            min_file_age_seconds: 0,
            include_hidden: true,
            ..Folder::default()
        };
        let result = walk(&f.0, &profile, SystemTime::now()).unwrap();
        assert_eq!(
            result
                .candidates
                .iter()
                .map(|c| c.path.as_path())
                .collect::<Vec<_>>(),
            [Path::new("good.PnG")]
        );
        assert_eq!(
            result
                .skipped
                .iter()
                .filter(|s| s.reason == "hardlink_skip")
                .count(),
            2
        );
        assert_eq!(
            result
                .skipped
                .iter()
                .filter(|s| s.reason == "skipped_policy: format_not_selected")
                .count(),
            1
        );
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].reason, "orphan_temp");
        assert!(!result
            .skipped
            .iter()
            .any(|s| s.path.to_string_lossy().starts_with(".tzim-")));
    }

    #[test]
    fn nested_temps_and_version_control_are_excluded_with_hidden_enabled() {
        let f = Fixture::new();
        std::fs::create_dir_all(f.0.join("sub/.git")).unwrap();
        std::fs::write(f.0.join("sub/.git/config"), b"private").unwrap();
        std::fs::write(f.0.join("sub/.tzim-99.tmp"), b"temp").unwrap();
        std::fs::write(f.0.join("sub/.visible"), b"ordinary").unwrap();
        let profile = Folder {
            include_hidden: true,
            min_file_age_seconds: 0,
            ..Folder::default()
        };
        let result = walk(&f.0, &profile, SystemTime::now()).unwrap();
        assert_eq!(
            result
                .candidates
                .iter()
                .map(|row| row.path.as_path())
                .collect::<Vec<_>>(),
            [Path::new("sub/.visible")]
        );
        assert_eq!(
            result
                .diagnostics
                .iter()
                .filter(|row| row.reason == "orphan_temp")
                .count(),
            1
        );
        assert_eq!(result.limits.get("excluded by .git/**"), Some(&1));
        assert_eq!(
            std::fs::read(f.0.join("sub/.tzim-99.tmp")).unwrap(),
            b"temp"
        );
    }

    #[test]
    fn unreadable_regular_file_is_discovered_without_opening_it() {
        use std::os::unix::fs::PermissionsExt;
        let f = Fixture::new();
        let path = f.0.join("unreadable");
        std::fs::write(&path, b"unchanged").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000)).unwrap();
        let profile = Folder {
            min_file_age_seconds: 0,
            ..Folder::default()
        };
        let result = walk(&f.0, &profile, SystemTime::now()).unwrap();
        assert_eq!(result.candidates.len(), 1);
        assert_eq!(result.candidates[0].path, Path::new("unreadable"));
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        assert_eq!(std::fs::read(path).unwrap(), b"unchanged");
    }

    #[test]
    fn read_only_tree_remains_byte_identical() {
        use std::os::unix::fs::PermissionsExt;
        let f = Fixture::new();
        let child = f.0.join("nested");
        std::fs::create_dir(&child).unwrap();
        for (name, bytes) in [("a", b"one".as_slice()), ("nested/b", b"two".as_slice())] {
            std::fs::write(f.0.join(name), bytes).unwrap();
        }
        let snapshot = || {
            let mut result = Vec::new();
            for dir in [&f.0, &child] {
                let mut entries: Vec<_> = std::fs::read_dir(dir)
                    .unwrap()
                    .map(|e| e.unwrap().file_name())
                    .collect();
                entries.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
                result.push(entries);
            }
            let hashes = ["a", "nested/b"]
                .map(|name| crate::hash::sha256(&std::fs::read(f.0.join(name)).unwrap()));
            (result, hashes)
        };
        let before = snapshot();
        let profile = Folder {
            min_file_age_seconds: 0,
            ..Folder::default()
        };
        let now = SystemTime::now();
        let first = walk(&f.0, &profile, now).unwrap();
        assert_eq!(snapshot(), before);
        for path in [&f.0, &child] {
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o555)).unwrap();
        }
        for path in [f.0.join("a"), child.join("b")] {
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o444)).unwrap();
        }
        let second = walk(&f.0, &profile, now).unwrap();
        assert_eq!(second.candidates, first.candidates);
        assert_eq!(snapshot(), before);
        for path in [&f.0, &child] {
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).unwrap();
        }
    }

    #[test]
    fn depth_limit_is_once_per_boundary_directory() {
        let f = Fixture::new();
        let mut p = f.0.clone();
        for _ in 0..65 {
            p.push("d");
            std::fs::create_dir(&p).unwrap();
        }
        std::fs::write(p.join("not-visited"), b"x").unwrap();
        let result = walk(&f.0, &Folder::default(), SystemTime::now()).unwrap();
        assert_eq!(result.directories, 65);
        assert_eq!(result.max_depth, 64);
        assert_eq!(result.limits.get("depth 64"), Some(&1));
        assert_eq!(
            result
                .skipped
                .iter()
                .filter(|s| s.reason == "depth_exceeded")
                .count(),
            1
        );
        assert!(result.candidates.is_empty());
    }

    // Run explicitly by CI after its shell creates a real FIFO using mkfifo.
    #[test]
    #[ignore = "requires AT03_FIFO_ROOT with a FIFO created by CI"]
    fn fifo_is_skipped_without_opening() {
        use std::os::unix::fs::FileTypeExt;
        let root = PathBuf::from(
            std::env::var_os("AT03_FIFO_ROOT").expect("CI must provide FIFO fixture"),
        );
        let root = std::fs::canonicalize(root).unwrap();
        assert!(std::fs::symlink_metadata(root.join("fifo"))
            .unwrap()
            .file_type()
            .is_fifo());
        let start = std::time::Instant::now();
        let result = walk(&root, &Folder::default(), SystemTime::now()).unwrap();
        assert!(start.elapsed() < Duration::from_secs(5));
        assert!(result.candidates.is_empty());
        assert!(result
            .skipped
            .iter()
            .any(|skip| skip.path == Path::new("fifo") && skip.reason == "skipped_policy: fifo"));
    }

    #[test]
    fn ten_thousand_entries_record_elapsed_time() {
        let f = Fixture::new();
        for index in 0..10_000 {
            std::fs::File::create(f.0.join(format!("f{index:05}"))).unwrap();
        }
        let profile = Folder {
            min_file_age_seconds: 0,
            ..Folder::default()
        };
        let start = std::time::Instant::now();
        let result = walk(&f.0, &profile, SystemTime::now()).unwrap();
        assert_eq!(result.files_seen, 10_000);
        assert_eq!(result.candidates.len(), 10_000);
        assert_eq!(result.directories, 1);
        let plans = crate::planner::build_plans(vec![(profile, result)], |folder, candidate| {
            Ok(crate::planner::arguments(folder, candidate))
        })
        .unwrap();
        assert_eq!(plans[0].rows.len(), 10_000);
        let elapsed = start.elapsed();
        eprintln!(
            "AT03_BUDGET_10000_MS={} (walk, policy, plan rows and hash)",
            elapsed.as_millis()
        );
        if cfg!(target_os = "linux") {
            assert!(
                elapsed < Duration::from_secs(15),
                "D-65 budget exceeded: {elapsed:?}"
            );
        }
    }

    #[test]
    fn hostile_names_remain_bytes_and_are_sorted() {
        let f = Fixture::new();
        let names: &[&[u8]] = &[
            b"plain.png",
            b"with space.png",
            b"two  spaces .png",
            b"semi;colon.png",
            b"dollar$(touch ats_pwned).png",
            b"back`touch ats_pwned2`.png",
            b"quote'single.png",
            b"quote\"double.png",
            b"newline\nname.png",
            b"tab\tname.png",
            b"-leading-dash.png",
            b"--json",
            b"--",
            b"-",
            "emoji-🙂.png".as_bytes(),
            "кириллица.png".as_bytes(),
            "日本語.png".as_bytes(),
            "combining-e\u{301}.png".as_bytes(),
            b"bad\xff.png",
        ];
        let mut expected = Vec::new();
        for name in names {
            let path = std::ffi::OsString::from_vec(name.to_vec());
            match std::fs::write(f.0.join(path), b"x") {
                Ok(()) => expected.push(name.to_vec()),
                Err(e) if *name == b"bad\xff.png" && e.raw_os_error() == Some(92) => {
                    eprintln!("N-17 skipped: filesystem rejects non-UTF-8 names: {e}")
                }
                Err(e) => panic!("fixture creation: {e}"),
            }
        }
        assert!(!f.0.join("ats_pwned").exists());
        assert!(!f.0.join("ats_pwned2").exists());
        expected.sort();
        let profile = Folder {
            min_file_age_seconds: 0,
            ..Folder::default()
        };
        let result = walk(&f.0, &profile, SystemTime::now()).unwrap();
        assert_eq!(
            result
                .candidates
                .iter()
                .map(|c| c.path.as_os_str().as_bytes().to_vec())
                .collect::<Vec<_>>(),
            expected
        );
        assert!(!f.0.join("ats_pwned").exists());
        assert!(!f.0.join("ats_pwned2").exists());
        assert!(!std::path::Path::new("ats_pwned").exists());
        assert!(!std::path::Path::new("ats_pwned2").exists());
    }
}
