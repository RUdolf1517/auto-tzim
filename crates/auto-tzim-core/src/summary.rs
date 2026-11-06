//! Human-readable scope and refusal accounting for a read-only walk.
use crate::discovery::Discovery;
use std::collections::BTreeMap;
use std::fmt::Write;

/// Summarize the actual walk, without predicting outcomes of a future apply.
pub fn scope(discovery: &Discovery) -> String {
    let top = discovery
        .candidates
        .iter()
        .filter(|c| c.path.components().count() == 1)
        .count();
    let below = discovery.candidates.len() - top;
    let mut text = format!("scope: {top} planned at top level, {below} below; {} directories walked; maximum depth {}\n", discovery.directories, discovery.max_depth);
    for (reason, count) in &discovery.limits {
        let _ = writeln!(text, "{count} directories not walked ({reason})");
    }
    if discovery.candidates.is_empty() {
        let _ = writeln!(
            text,
            "0 candidates — {} files looked at",
            discovery.files_seen
        );
        let mut reasons: BTreeMap<&str, u64> = discovery
            .filtered_files
            .iter()
            .map(|(reason, count)| (reason.as_str(), *count))
            .collect();
        for skip in &discovery.skipped {
            if matches!(skip.reason.as_str(), "hardlink_skip" | "skipped_unstable") {
                *reasons.entry(skip.reason.as_str()).or_default() += 1;
            }
        }
        if reasons.is_empty() {
            text.push_str("no regular files qualified in the walked scope\n");
        } else {
            for (reason, count) in reasons {
                let _ = writeln!(text, "{count} files: {reason}");
            }
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_walk_names_scope_limits_and_reasons() {
        let mut found = Discovery {
            directories: 1,
            max_depth: 0,
            files_seen: 2,
            ..Discovery::default()
        };
        found.limits.insert("non-recursive".into(), 3);
        found.filtered_files.insert("hidden".into(), 2);
        let output = scope(&found);
        assert!(output.contains("1 directories walked"));
        assert!(output.contains("maximum depth 0"));
        assert!(output.contains("3 directories not walked (non-recursive)"));
        assert!(output.contains("0 candidates — 2 files looked at"));
        assert!(output.contains("2 files: hidden"));
        for forbidden in ["save", "smaller", "optimiz", "%"] {
            assert!(!output.contains(forbidden));
        }
    }
}
