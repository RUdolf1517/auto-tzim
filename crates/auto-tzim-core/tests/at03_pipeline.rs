//! A reproducible partial AT-03 pipeline fixture. This is not the full frozen corpus.
use auto_tzim_core::{batch, config::Folder, discovery, hash, planner, summary};
use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "atz-pipeline-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("sub/.git")).unwrap();
        Self(fs::canonicalize(root).unwrap())
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn file(root: &Path, name: &str, now: SystemTime, age_seconds: u64) {
    let path = root.join(name);
    fs::write(&path, name.as_bytes()).unwrap();
    let handle = fs::OpenOptions::new().write(true).open(path).unwrap();
    handle
        .set_modified(now - Duration::from_secs(age_seconds))
        .unwrap();
}

#[test]
fn reproducible_partial_fixture_across_discovery_planning_and_rendering() {
    let fixture = Fixture::new();
    let now = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
    // Four old, ordinary candidates: two at top-level and two in the subdirectory.
    let eligible = ["a.png", "z.png", "sub/b.png", "sub/c.png"];
    for name in eligible {
        file(&fixture.0, name, now, 120);
    }
    file(&fixture.0, "young.png", now, 10);
    file(&fixture.0, "hard-a", now, 120);
    fs::hard_link(fixture.0.join("hard-a"), fixture.0.join("hard-b")).unwrap();
    file(&fixture.0, "sub/.tzim-123-456-0.tmp", now, 120);
    file(&fixture.0, "sub/.git/config", now, 120);
    symlink("../a.png", fixture.0.join("sub/link")).unwrap();
    let contents = [
        "a.png",
        "z.png",
        "sub/b.png",
        "sub/c.png",
        "young.png",
        "hard-a",
        "hard-b",
        "sub/.tzim-123-456-0.tmp",
        "sub/.git/config",
    ];
    let before: BTreeMap<_, _> = contents
        .into_iter()
        .map(|name| (name, hash::sha256(&fs::read(fixture.0.join(name)).unwrap())))
        .collect();
    let profile = Folder {
        root: fixture.0.clone(),
        include_hidden: true,
        min_file_age_seconds: 60,
        ..Folder::default()
    };
    let first = discovery::walk(&fixture.0, &profile, now).unwrap();
    assert_eq!(
        first
            .candidates
            .iter()
            .map(|row| row.path.as_path())
            .collect::<Vec<_>>(),
        [
            Path::new("a.png"),
            Path::new("sub/b.png"),
            Path::new("sub/c.png"),
            Path::new("z.png")
        ]
    );
    assert_eq!(first.files_seen, 8); // .git is excluded before entering it
    assert_eq!(first.directories, 2);
    assert_eq!(first.limits.get("excluded by .git/**"), Some(&1));
    assert_eq!(
        first
            .skipped
            .iter()
            .filter(|s| s.reason == "hardlink_skip")
            .count(),
        2
    );
    assert_eq!(
        first
            .skipped
            .iter()
            .filter(|s| s.reason == "skipped_unstable")
            .count(),
        1
    );
    assert_eq!(
        first
            .skipped
            .iter()
            .filter(|s| s.reason == "orphan_temp")
            .count(),
        1
    );
    assert_eq!(
        first
            .skipped
            .iter()
            .filter(|s| s.reason == "skipped_policy: symlink")
            .count(),
        1
    );
    // Injected test argv is not an adapter contract and is never executed.
    let plans = planner::build_plans(vec![(profile.clone(), first)], |_, candidate| {
        Ok(vec![
            "fixture-only".into(),
            candidate.path.to_string_lossy().into_owned(),
        ])
    })
    .unwrap();
    assert_eq!(plans.len(), 1);
    assert_eq!(plans[0].rows.len(), 4);
    let output = summary::plan(&plans[0]);
    assert_eq!(
        output
            .lines()
            .filter(|line| line.starts_with("planned "))
            .count(),
        4
    );
    assert!(output.contains("2 directories walked; maximum depth 1"));
    assert!(output.contains("1 directories not walked (excluded by .git/**)"));
    for text in ["save", "smaller", "optimiz", "%"] {
        assert!(!output.contains(text));
    }
    let batch = batch::discover(vec![profile.clone(), profile], now).unwrap();
    assert_eq!(batch.len(), 1);
    assert_eq!(batch[0].1.candidates, plans[0].discovery.candidates);
    let after: BTreeMap<_, _> = contents
        .into_iter()
        .map(|name| (name, hash::sha256(&fs::read(fixture.0.join(name)).unwrap())))
        .collect();
    assert_eq!(before, after);
    eprintln!("AT03_PARTIAL_FIXTURE planned=4 candidates=4 hardlinks=2 young=1 temp=1 symlink=1 directories_walked=2 excluded_dirs=1");
}
