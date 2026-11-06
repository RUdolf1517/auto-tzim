//! Batch preflight: validate every root before a walk or plan is started.
use crate::config::{self, Folder};
use std::path::PathBuf;

/// Canonicalise and deduplicate roots in first-occurrence order, rejecting overlaps.
pub fn preflight(profiles: Vec<Folder>) -> Result<Vec<Folder>, String> {
    let mut accepted: Vec<Folder> = Vec::new();
    for mut profile in profiles {
        let original = profile.root.clone();
        let root = config::validate_root(&original, profile.allow_cloud_sync)
            .map_err(|error| format!("{}: {error}", original.display()))?;
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
