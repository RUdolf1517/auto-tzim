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
