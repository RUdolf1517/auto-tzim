//! Human-readable scope and refusal accounting for a read-only walk.
use crate::discovery::Discovery;
use std::collections::BTreeMap;
use std::fmt::Write;

/// Render one completed in-memory plan for a human reader. Names and arguments
/// are escaped so control characters cannot manufacture additional output rows.
pub fn plan(plan: &crate::planner::Plan) -> String {
    use crate::planner::{base64, json_string};
    use std::os::unix::ffi::OsStrExt;
    let mut text = String::new();
    let root = json_string(&plan.root.to_string_lossy());
    let _ = writeln!(
        text,
        "root: {root}\nplan id: {}\nplan hash: {}",
        plan.id, plan.hash
    );
    for row in &plan.rows {
        let path = json_string(&row.candidate.path.to_string_lossy());
        let raw = base64(row.candidate.path.as_os_str().as_bytes());
        let id = &row.candidate.identity;
        let argv = row
            .argv
            .iter()
            .map(|arg| json_string(arg))
            .collect::<Vec<_>>()
            .join(", ");
        let _ = writeln!(
            text,
            "planned {path} path_raw_b64={raw} size={} dev={} ino={} mtime_ns={} argv=[{argv}]",
            id.size, id.dev, id.ino, id.mtime_ns
        );
    }
    for skip in &plan.discovery.skipped {
        let path = json_string(&skip.path.to_string_lossy());
        let reason = json_string(&skip.reason);
        let raw = base64(skip.path.as_os_str().as_bytes());
        let _ = writeln!(text, "skipped {path} path_raw_b64={raw} reason={reason}");
    }
    text.push_str(&scope(&plan.discovery));
    text
}

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
        let reason = reason.escape_debug();
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
                let reason = reason.escape_debug();
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
    fn plan_display_escapes_names_and_only_claims_planned() {
        use crate::discovery::{Candidate, Discovery};
        use crate::planner::{Plan, Row};
        use crate::stability::Identity;
        let candidate = Candidate {
            path: "odd\nname".into(),
            identity: Identity {
                dev: 1,
                ino: 2,
                size: 3,
                mtime_ns: 4,
                links: 1,
            },
        };
        let discovery = Discovery {
            candidates: vec![candidate.clone()],
            directories: 1,
            files_seen: 1,
            ..Discovery::default()
        };
        let plan = Plan {
            id: "id".into(),
            root: "/test".into(),
            hash: "hash".into(),
            rows: vec![Row {
                candidate,
                argv: vec!["--".into(), "odd\nname".into()],
            }],
            discovery,
        };
        let output = super::plan(&plan);
        assert!(output.contains("planned \"odd\\u000aname\""));
        assert!(output.contains("argv=[\"--\", \"odd\\u000aname\"]"));
        assert!(!output.contains("odd\nname"));
        for forbidden in ["save", "smaller", "optimiz", "%"] {
            assert!(!output.contains(forbidden));
        }
    }

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
