//! Apply a single approved plan row using its recorded format selection.

use crate::adapter;
use crate::install::{self, InstallError, InstallJournal};
use crate::planner::Row;
use crate::runner::{self, Cancellation, RunError};
use crate::staging::{self, StageError};
use std::fs;
use std::io;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, PartialEq, Eq)]
pub struct AppliedFile {
    pub status: &'static str,
    pub tzim_status: Option<String>,
    pub tzim_exit_code: Option<i32>,
    pub tzim_error: Option<runner::TzimError>,
    pub tzim_stderr: Option<String>,
    pub reason: Option<String>,
    pub note: Option<String>,
    pub before: u64,
    pub after: u64,
}

#[derive(Debug)]
pub enum ApplyError {
    InvalidArguments,
    Stage(StageError),
    Runner(RunError),
    Install(InstallError),
    Cleanup(io::Error),
}

/// Apply one row while the caller holds the run lock. The run owns the private
/// staging directory and cleans it up after any needed journal replay.
pub fn apply_row(
    root: &Path,
    row: &Row,
    binary: &adapter::Identity,
    staging_dir: &Path,
    journal: &mut InstallJournal,
    timeout: Duration,
    cancellation: &Cancellation,
) -> Result<AppliedFile, ApplyError> {
    let formats = match row.argv.as_slice() {
        [separator, path] if separator == "--" && path == row.candidate.path.as_os_str() => None,
        [flag, formats, separator, path]
            if flag == "--formats"
                && !formats.is_empty()
                && separator == "--"
                && path == row.candidate.path.as_os_str() =>
        {
            Some(formats.as_os_str())
        }
        _ => return Err(ApplyError::InvalidArguments),
    };

    let staged = match staging::stage_file(root, &row.candidate, staging_dir) {
        Ok(staged) => staged,
        Err(StageError::SourceChanged) => return Ok(changed(row)),
        Err(error) => return Err(ApplyError::Stage(error)),
    };
    let (result, tzim_exit_code) =
        match runner::process_staged_file(binary, &staged, formats, timeout, cancellation) {
            Ok(result) => (result, Some(0)),
            Err(RunError::Exit {
                status,
                report: Some(report),
                ..
            }) => (*report, status.code()),
            Err(error) => return Err(ApplyError::Runner(error)),
        };
    let output = AppliedFile {
        status: result.status,
        tzim_status: Some(result.tzim_status.clone()),
        tzim_exit_code,
        tzim_error: result.error.clone(),
        tzim_stderr: result.stderr.clone(),
        reason: result.reason.clone(),
        note: result.note.clone(),
        before: result.before,
        after: result.after,
    };
    if let Some(signal) = cancellation.requested() {
        return Err(ApplyError::Runner(RunError::Interrupted(signal)));
    }
    if let Some(replacement) = result.into_replacement() {
        // Once installation starts, leave staging available for recovery on error.
        match staging::check_source(root, &row.candidate) {
            Ok(_) => install::install(journal, replacement).map_err(ApplyError::Install)?,
            Err(StageError::SourceChanged) => {
                fs::remove_file(staged.path()).map_err(ApplyError::Cleanup)?;
                return Ok(changed(row));
            }
            Err(error) => return Err(ApplyError::Stage(error)),
        }
    } else {
        fs::remove_file(staged.path()).map_err(ApplyError::Cleanup)?;
    }
    Ok(output)
}

fn changed(row: &Row) -> AppliedFile {
    AppliedFile {
        status: "skipped_changed",
        tzim_status: None,
        tzim_exit_code: None,
        tzim_error: None,
        tzim_stderr: None,
        reason: Some("source_changed".into()),
        note: None,
        before: row.candidate.identity.size,
        after: row.candidate.identity.size,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{discovery::Candidate, json, stability::Identity};
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn applies_a_planned_file_through_the_staged_runner() {
        struct Fixture(PathBuf);
        impl Drop for Fixture {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let base =
            std::env::temp_dir().join(format!("auto-tzim-apply-{}-{nonce}", std::process::id()));
        fs::DirBuilder::new().mode(0o700).create(&base).unwrap();
        let fixture = Fixture(fs::canonicalize(base).unwrap());
        let root = fixture.0.join("source");
        let stage = fixture.0.join("stage");
        for dir in [&root, &stage] {
            fs::DirBuilder::new().mode(0o700).create(dir).unwrap();
        }
        let source = root.join("photo.png");
        fs::write(&source, b"original").unwrap();
        let fake = fixture.0.join("tzim");
        fs::write(&fake, r#"#!/bin/sh
[ "$#" -eq 3 ] && [ "$1" = --json ] && [ "$2" = -- ] || exit 2
case "$3" in */staging/*/photo.png) ;; *) exit 2 ;; esac
printf small > "$3"
printf '%s\n' '{"type":"file","status":"optimized","reason_code":null,"note":null,"before":8,"after":5,"saved":3}' '{"type":"summary","schema":2,"files":1,"optimized":1,"would_optimize":0,"already_optimal":0,"unsupported":0,"skipped":0,"failed":0,"before":8,"after":5,"saved":3,"dry_run":false,"strip_metadata":false}'
"#).unwrap();
        fs::set_permissions(&fake, fs::Permissions::from_mode(0o700)).unwrap();

        let row = Row {
            candidate: Candidate {
                path: "photo.png".into(),
                identity: Identity::from_metadata(&fs::symlink_metadata(&source).unwrap()),
            },
            argv: vec!["--".into(), "photo.png".into()],
        };
        let binary = adapter::Identity {
            path: fake.clone(),
            realpath: fake,
            size: 0,
            sha256: String::new(),
            version: "0.16.1".into(),
            capabilities_schema: 1,
            result_schema: 2,
            adapter: "test",
            capabilities: json::Value::Null,
        };
        let rows = vec![row];
        let plan = crate::planner::Plan {
            id: "approved-plan".into(),
            hash: crate::planner::plan_hash(&rows),
            root,
            rows,
            discovery: Default::default(),
        };
        let metadata = crate::history::RunMetadata {
            folder: "test",
            mode: crate::history::Mode::Auto,
            batch_id: None,
            config_hash: "test-config",
            cooldown_hours: 24,
        };
        let declined = apply_plan(
            &stage,
            &plan,
            &binary,
            &crate::history::RunMetadata {
                mode: crate::history::Mode::Declined,
                ..metadata
            },
            Duration::ZERO,
            &Cancellation::default(),
        )
        .unwrap();
        assert_eq!(declined.exit_code, 0);
        assert_eq!(fs::read(&source).unwrap(), b"original");
        assert!(!stage.join("last-compression.json").exists());
        assert!(!stage.join("staging").exists());
        let declined_run = stage.join("runs").join(declined.run_id);
        let declined_record =
            json::parse(&fs::read(declined_run.join("run.json")).unwrap()).unwrap();
        assert_eq!(
            declined_record.member("mode").unwrap(),
            &json::Value::String("declined".into())
        );
        assert_eq!(
            declined_record
                .member("counts")
                .unwrap()
                .integer("skipped_by_user")
                .unwrap(),
            1
        );

        let output = apply_plan(
            &stage,
            &plan,
            &binary,
            &metadata,
            Duration::from_secs(2),
            &Cancellation::default(),
        )
        .unwrap();
        assert_eq!(output.exit_code, 0);
        assert_eq!(fs::read(source).unwrap(), b"small");
        assert_eq!(fs::read_dir(stage.join("staging")).unwrap().count(), 0);
        let run = stage.join("runs").join(output.run_id);
        let summary = json::parse(&fs::read(run.join("run.json")).unwrap()).unwrap();
        assert_eq!(
            summary.member("state").unwrap(),
            &json::Value::String("completed".into())
        );
        assert_eq!(
            summary
                .member("counts")
                .unwrap()
                .integer("optimized")
                .unwrap(),
            1
        );
        let file = json::parse(&fs::read(run.join("files.jsonl")).unwrap()).unwrap();
        assert_eq!(
            file.member("status").unwrap(),
            &json::Value::String("optimized".into())
        );
        let run_record = run.join("run.json");
        let interrupted = fs::read_to_string(&run_record)
            .unwrap()
            .replace("\"state\":\"completed\"", "\"state\":\"running\"");
        fs::write(&run_record, interrupted).unwrap();
        let files = run.join("files.jsonl");
        let offset = fs::metadata(&files).unwrap().len();
        fs::write(
            run.join("in-flight.json"),
            format!(r#"{{"offset":{offset},"outcome":{{"status":"interrupted","relative_path":"next.png"}}}}"#),
        ).unwrap();
        let recovered = crate::history::recover_runs(&stage).unwrap();
        assert_eq!(recovered.len(), 1);
        let summary = json::parse(&fs::read(run_record).unwrap()).unwrap();
        assert_eq!(
            summary.member("state").unwrap(),
            &json::Value::String("interrupted".into())
        );
        assert_eq!(
            summary
                .member("counts")
                .unwrap()
                .integer("optimized")
                .unwrap(),
            1
        );
        assert_eq!(
            summary
                .member("counts")
                .unwrap()
                .integer("interrupted")
                .unwrap(),
            1
        );
        let rows = fs::read_to_string(&files).unwrap();
        assert_eq!(rows.lines().count(), 2);
        assert!(rows.lines().last().unwrap().contains("next.png"));
        assert!(crate::history::recover_runs(&stage).unwrap().is_empty());
        let cancellation = Cancellation::default();
        cancellation.request(runner::Interrupt::Int);
        cancellation.request(runner::Interrupt::Term);
        let cancelled = apply_plan(
            &stage,
            &plan,
            &binary,
            &metadata,
            Duration::ZERO,
            &cancellation,
        )
        .unwrap();
        assert_eq!(cancelled.exit_code, 130);
        let summary = json::parse(
            &fs::read(stage.join("runs").join(cancelled.run_id).join("run.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            summary.member("state").unwrap(),
            &json::Value::String("interrupted".into())
        );
        assert_eq!(
            summary
                .member("counts")
                .unwrap()
                .integer("interrupted")
                .unwrap(),
            1
        );
        let source = plan.root.join("photo.png");
        let fixture_script = fs::read_to_string(&binary.realpath).unwrap();
        for (edit, expected) in [
            (
                r#"printf modified > "${0%/*}/source/photo.png""#,
                b"modified".as_slice(),
            ),
            (
                r#"/usr/bin/touch -t 200001010000 "${0%/*}/source/photo.png""#,
                b"original".as_slice(),
            ),
        ] {
            fs::write(&source, b"original").unwrap();
            fs::write(
                &binary.realpath,
                fixture_script.replace("printf small", &format!("{edit}\nprintf small")),
            )
            .unwrap();
            let mut rows = plan.rows.clone();
            rows[0].candidate.identity =
                Identity::from_metadata(&fs::symlink_metadata(&source).unwrap());
            let changed_plan = crate::planner::Plan {
                id: "changed-plan".into(),
                hash: crate::planner::plan_hash(&rows),
                root: plan.root.clone(),
                rows,
                discovery: Default::default(),
            };
            let outcome = apply_plan(
                &stage,
                &changed_plan,
                &binary,
                &metadata,
                Duration::from_secs(2),
                &Cancellation::default(),
            )
            .unwrap();
            assert_eq!(outcome.exit_code, 0);
            assert_eq!(fs::read(&source).unwrap(), expected);
            assert!(!stage.join("staging").join(&outcome.run_id).exists());
            let record =
                fs::read(stage.join("runs").join(outcome.run_id).join("files.jsonl")).unwrap();
            let row = json::parse(&record).unwrap();
            assert_eq!(
                row.member("status").unwrap(),
                &json::Value::String("skipped_changed".into())
            );
        }
    }
}

#[derive(Debug)]
pub enum PlanError {
    Changed,
    Io(io::Error),
}

impl From<io::Error> for PlanError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Debug)]
pub struct RunOutcome {
    pub run_id: String,
    pub exit_code: u8,
    pub stopped_early: bool,
    pub recovered: Vec<crate::history::RecoveredRun>,
}

/// Execute an approved plan under the caller's run lock. A failed row stops the
/// run and is persisted along with interrupted outcomes for the remaining rows.
pub fn apply_plan(
    state: &Path,
    plan: &crate::planner::Plan,
    binary: &adapter::Identity,
    metadata: &crate::history::RunMetadata<'_>,
    timeout: Duration,
    cancellation: &Cancellation,
) -> Result<RunOutcome, PlanError> {
    use crate::history::{private_directory, RunHistory};
    use std::os::unix::fs::DirBuilderExt;
    use std::time::Instant;

    if crate::planner::plan_hash(&plan.rows) != plan.hash {
        return Err(PlanError::Changed);
    }
    let state = fs::canonicalize(state)?;
    if state.starts_with(&plan.root) {
        return Err(PlanError::Changed);
    }
    let runs = state.join("runs");
    let staging = state.join("staging");
    private_directory(&runs)?;
    fs::File::open(&state)?.sync_all()?;

    let declined = matches!(metadata.mode, crate::history::Mode::Declined);
    let recovered = if declined {
        Vec::new()
    } else {
        crate::history::recover_runs(&state)?
    };
    let id = crate::planner::plan_id()?;
    let mut history = RunHistory::start(&runs, &id, plan, binary, metadata)?;
    if declined {
        for row in &plan.rows {
            history.append(
                row,
                &unapplied(row, "skipped_by_user", None),
                Duration::ZERO,
            )?;
        }
        history.finish("completed", 0)?;
        return Ok(RunOutcome {
            run_id: id,
            exit_code: 0,
            stopped_early: false,
            recovered,
        });
    }
    private_directory(&staging)?;
    fs::File::open(&state)?.sync_all()?;
    let mut journal = InstallJournal::create(&history.directory, &id)?;
    let staging = staging.join(&id);
    fs::DirBuilder::new().mode(0o700).create(&staging)?;
    let mut exit_code = 0;
    let mut stopped_early = false;
    let mut cache = crate::eligibility::Cache::open(&state)?;
    let mut started_processing = false;
    for row in &plan.rows {
        let started = Instant::now();
        if let Some(signal) = cancellation.requested() {
            exit_code = signal as u8;
        }
        let processing = !stopped_early && exit_code < 128;
        let result = if processing {
            history.begin_file(row)?;
            let key = crate::eligibility::Key::new(&plan.root, row, binary, metadata.config_hash);
            let hit = match staging::check_source(&plan.root, &row.candidate) {
                Ok(_) => cache.lookup(&key, row.candidate.identity.size),
                Err(_) => None,
            };
            let outcome = if let Some(result) = hit {
                result
            } else {
                if !started_processing {
                    crate::cooldown::Cooldown::record(
                        &state,
                        &plan.root,
                        &id,
                        metadata.cooldown_hours,
                    )?;
                    started_processing = true;
                }
                match apply_row(
                    &plan.root,
                    row,
                    binary,
                    &staging,
                    &mut journal,
                    timeout,
                    cancellation,
                ) {
                    Ok(result) => {
                        cache.add(key, &result)?;
                        result
                    }
                    Err(ApplyError::Runner(RunError::Interrupted(signal))) => {
                        exit_code = signal as u8;
                        unapplied(row, "interrupted", None)
                    }
                    Err(ApplyError::Runner(RunError::Timeout)) => {
                        let mut result = unapplied(row, "failed", None);
                        result.reason = Some("timeout".into());
                        result
                    }
                    Err(error) => unapplied(row, "failed", Some(format!("{error:?}"))),
                }
            };
            if outcome.status == "failed" {
                exit_code = 1;
                stopped_early = outcome.tzim_exit_code != Some(1);
            }
            outcome
        } else {
            unapplied(row, "interrupted", None)
        };
        history.append(row, &result, started.elapsed())?;
        if processing {
            history.clear_file()?;
        }
    }
    if let Some(signal) = cancellation.requested() {
        exit_code = signal as u8;
    }
    stopped_early |= exit_code >= 128;
    if !stopped_early {
        fs::remove_dir(&staging)?;
    }
    let state = match (stopped_early, exit_code) {
        (_, 130 | 143) => "interrupted",
        (true, _) => "aborted",
        (false, _) => "completed",
    };
    history.finish(state, exit_code)?;
    Ok(RunOutcome {
        run_id: id,
        exit_code,
        stopped_early,
        recovered,
    })
}

fn unapplied(row: &Row, status: &'static str, note: Option<String>) -> AppliedFile {
    AppliedFile {
        status,
        tzim_status: None,
        tzim_exit_code: None,
        tzim_error: None,
        tzim_stderr: None,
        reason: None,
        note,
        before: row.candidate.identity.size,
        after: row.candidate.identity.size,
    }
}
