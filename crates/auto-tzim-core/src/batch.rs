//! Batch preflight: validate every root before a walk or plan is started.
use crate::config::{self, Folder};
use std::path::PathBuf;

/// One typed input, kept in the order in which the user supplied it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RootInput {
    Path(PathBuf),
    Folder(String),
}

/// Resolve named profiles without walking roots or mutating the configuration.
/// The recursive override, when supplied, applies only to this scan.
pub fn resolve_inputs(
    inputs: Vec<RootInput>,
    config: &config::Config,
    recursive: Option<bool>,
) -> Result<Vec<Folder>, String> {
    inputs
        .into_iter()
        .map(|input| {
            let mut profile = match input {
                RootInput::Path(path) => {
                    let absolute = if path.is_absolute() {
                        path
                    } else {
                        std::env::current_dir()
                            .map_err(|e| format!("current directory: {e}"))?
                            .join(path)
                    };
                    direct(absolute)
                }
                RootInput::Folder(name) => config
                    .folder
                    .iter()
                    .find(|folder| folder.name.to_lowercase() == name.to_lowercase())
                    .cloned()
                    .ok_or_else(|| format!("folder not found: {name}"))?,
            };
            if let Some(recursive) = recursive {
                profile.recursive = recursive;
            }
            Ok(profile)
        })
        .collect()
}

/// Canonicalise and deduplicate roots in first-occurrence order, rejecting overlaps.
pub fn preflight(profiles: Vec<Folder>) -> Result<Vec<Folder>, String> {
    let mut accepted: Vec<Folder> = Vec::new();
    let config_path = config::config_path().map_err(|error| error.to_string())?;
    let state = config_path
        .parent()
        .ok_or_else(|| "missing state directory".to_owned())?;
    let state =
        canonical_with_missing(state).map_err(|error| format!("state directory: {error}"))?;
    for mut profile in profiles {
        let original = profile.root.clone();
        let root = config::validate_root(&original, profile.allow_cloud_sync)
            .map_err(|error| format!("{}: {error}", original.display()))?;
        if root.starts_with(&state) || state.starts_with(&root) {
            return Err(format!("{}: state_inside_root", original.display()));
        }
        if accepted.iter().any(|item| item.root == root) {
            continue;
        }
        if let Some(other) = accepted
            .iter()
            .find(|item| item.root.starts_with(&root) || root.starts_with(&item.root))
        {
            return Err(format!(
                "overlapping_roots: {} and {}",
                other.root.display(),
                root.display()
            ));
        }
        profile.root = root;
        accepted.push(profile);
    }
    if accepted.is_empty() {
        return Err("no roots specified".into());
    }
    Ok(accepted)
}

// Resolve every existing ancestor, retaining missing final components. This
// performs no directory enumeration and handles a not-yet-created state home.
fn canonical_with_missing(path: &std::path::Path) -> std::io::Result<PathBuf> {
    use std::path::Component;
    let mut result = if path.is_absolute() {
        PathBuf::new()
    } else {
        std::env::current_dir()?
    };
    for component in path.components() {
        match component {
            Component::RootDir => result.push(component.as_os_str()),
            Component::CurDir => (),
            Component::ParentDir => {
                result.pop();
            }
            Component::Normal(part) => {
                result.push(part);
                match std::fs::symlink_metadata(&result) {
                    Ok(_) => result = std::fs::canonicalize(&result)?,
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
                    Err(error) => return Err(error),
                }
            }
            Component::Prefix(_) => return Err(std::io::Error::other("unsupported path prefix")),
        }
    }
    Ok(result)
}

/// Construct the default profile for an unconfigured root.
pub fn direct(root: PathBuf) -> Folder {
    Folder {
        root,
        ..Folder::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    #[test]
    fn validates_every_root_deduplicates_and_rejects_containment() {
        let root = std::env::temp_dir().join(format!(
            "atz-batch-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let a = root.join("a");
        let b = root.join("b");
        std::fs::create_dir_all(a.join("nested")).unwrap();
        std::fs::create_dir_all(&b).unwrap();
        let a = std::fs::canonicalize(a).unwrap();
        let b = std::fs::canonicalize(b).unwrap();
        let profiles = preflight(vec![
            direct(b.clone()),
            direct(a.clone()),
            direct(b.clone()),
        ])
        .unwrap();
        assert_eq!(
            profiles.iter().map(|x| &x.root).collect::<Vec<_>>(),
            [&b, &a]
        );
        let error = preflight(vec![direct(a.clone()), direct(a.join("nested"))]).unwrap_err();
        assert!(error.contains("overlapping_roots"));
        let error =
            preflight(vec![direct(a), direct(root.join("missing")), direct(b)]).unwrap_err();
        assert!(error.contains("root_missing"));
        std::fs::remove_dir_all(root).unwrap();
    }
}

/// Failure before scanning is invalid input; a discovery error aborts the batch.
#[derive(Debug)]
pub enum BatchError {
    Invalid(String),
    Discovery(String),
}

/// Preflight all roots, then walk them independently in their typed order.
/// A failed walk returns no partial batch to the caller.
pub fn discover(
    profiles: Vec<Folder>,
    now: std::time::SystemTime,
) -> Result<Vec<(Folder, crate::discovery::Discovery)>, BatchError> {
    discover_with(profiles, |profile| {
        crate::discovery::walk(&profile.root, profile, now)
    })
}

fn discover_with<F>(
    profiles: Vec<Folder>,
    mut walk: F,
) -> Result<Vec<(Folder, crate::discovery::Discovery)>, BatchError>
where
    F: FnMut(&Folder) -> Result<crate::discovery::Discovery, String>,
{
    let profiles = preflight(profiles).map_err(BatchError::Invalid)?;
    let mut result = Vec::with_capacity(profiles.len());
    for profile in profiles {
        let found = walk(&profile).map_err(BatchError::Discovery)?;
        result.push((profile, found));
    }
    Ok(result)
}

#[cfg(test)]
mod order_tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    #[test]
    fn invalid_batch_performs_no_walk_and_good_batch_preserves_order() {
        let dir = std::env::temp_dir().join(format!(
            "atz-preflight-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let a = dir.join("a");
        let b = dir.join("b");
        std::fs::create_dir_all(&a).unwrap();
        std::fs::create_dir_all(&b).unwrap();
        let mut seen = Vec::new();
        let invalid = discover_with(
            vec![
                direct(a.clone()),
                direct(dir.join("missing")),
                direct(b.clone()),
            ],
            |folder| {
                seen.push(folder.root.clone());
                Ok(crate::discovery::Discovery::default())
            },
        );
        assert!(matches!(invalid, Err(BatchError::Invalid(_))));
        assert!(seen.is_empty());
        let result = discover_with(
            vec![direct(b.clone()), direct(a.clone()), direct(b.clone())],
            |folder| {
                seen.push(folder.root.clone());
                Ok(crate::discovery::Discovery::default())
            },
        )
        .unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(
            seen,
            [
                std::fs::canonicalize(b).unwrap(),
                std::fs::canonicalize(a).unwrap()
            ]
        );
        std::fs::remove_dir_all(dir).unwrap();
    }
}

#[cfg(test)]
mod input_tests {
    use super::*;
    #[test]
    fn mixed_inputs_preserve_order_and_do_not_mutate_profiles() {
        let folder = Folder {
            name: "Pictures".into(),
            root: PathBuf::from("/named"),
            recursive: false,
            ..Folder::default()
        };
        let config = config::Config {
            schema: 1,
            installation: config::Installation::default(),
            folder: vec![folder.clone()],
        };
        let profiles = resolve_inputs(
            vec![
                RootInput::Path("/first".into()),
                RootInput::Folder("pictures".into()),
                RootInput::Path("/last".into()),
            ],
            &config,
            Some(true),
        )
        .unwrap();
        assert_eq!(
            profiles
                .iter()
                .map(|p| p.root.as_path())
                .collect::<Vec<_>>(),
            [
                std::path::Path::new("/first"),
                std::path::Path::new("/named"),
                std::path::Path::new("/last")
            ]
        );
        assert!(profiles.iter().all(|p| p.recursive));
        assert_eq!(config.folder[0], folder);
        assert!(
            resolve_inputs(vec![RootInput::Folder("missing".into())], &config, None)
                .unwrap_err()
                .contains("missing")
        );
    }
}

#[cfg(test)]
mod batch_hash_tests {
    use super::*;
    use crate::planner::{canonical_rows, plan_hash, plan_id, Row};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::SystemTime;
    static NEXT: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn same_root_has_same_rows_and_hash_alone_or_in_a_batch() {
        let fixture = std::env::temp_dir().join(format!(
            "atz-batch-hash-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let roots: Vec<_> = ["a", "b", "c"]
            .into_iter()
            .map(|n| fixture.join(n))
            .collect();
        for (i, root) in roots.iter().enumerate() {
            std::fs::create_dir_all(root).unwrap();
            std::fs::write(root.join(format!("file-{i}")), b"data").unwrap();
        }
        let profile = |path: &std::path::Path| Folder {
            min_file_age_seconds: 0,
            ..direct(path.to_path_buf())
        };
        let now = SystemTime::now();
        let single = discover(vec![profile(&roots[0])], now).unwrap();
        let combined = discover(
            vec![profile(&roots[0]), profile(&roots[1]), profile(&roots[2])],
            now,
        )
        .unwrap();
        assert_eq!(combined.len(), 3);
        // The actual adapter argv is undecided; inject fixed argv to test batch independence only.
        let rows = |result: &crate::discovery::Discovery| {
            result
                .candidates
                .iter()
                .cloned()
                .map(|candidate| Row {
                    candidate,
                    argv: vec!["test-argument".into()],
                })
                .collect::<Vec<_>>()
        };
        let alone = rows(&single[0].1);
        let together = rows(&combined[0].1);
        assert_eq!(canonical_rows(&alone), canonical_rows(&together));
        assert_eq!(plan_hash(&alone), plan_hash(&together));
        let hashes: std::collections::HashSet<_> = combined
            .iter()
            .map(|(_, result)| plan_hash(&rows(result)))
            .collect();
        assert_eq!(hashes.len(), 3);
        let ids: std::collections::HashSet<_> = (0..3).map(|_| plan_id().unwrap()).collect();
        assert_eq!(ids.len(), 3);
        std::fs::remove_dir_all(fixture).unwrap();
    }
}
