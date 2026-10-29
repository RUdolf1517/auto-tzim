# Roadmap and issue breakdown

Internal IDs are used until the issues exist in GitHub; after creation each ID is replaced by its real number, and
no number is ever guessed. Stage boundaries are load-bearing: nothing from v0.2+ may be merged into the MVP pull
request, even "while we are here".

Scope of this stage: **MVP / v0.1**, derived from `SPEC.md` §17. Acceptance is defined clause by clause in
[GATES.md](GATES.md).

---

## Stage 1 — MVP / v0.1

### AT-01 · Repository and package bootstrap

**Scope.** A Python package `auto_tzim` with the `core/` and `cli/` layout of `SPEC.md` §4.2 (the `tui/` and `qt/`
directories are *not* created), an entry point named `auto-tzim`, `pyproject.toml` with pinned build metadata,
`README` linkage to the specification, `CHANGELOG.md` starting at `0.1.0` unreleased, CI with `ubuntu-latest` and
`macos-latest` on Python 3.12 and 3.13, and a `lockfile` job (D-15).

**Acceptance.** A clean checkout installs, `auto-tzim --help` and `auto-tzim --version` run on both platforms, and
the CI jobs are green at a recorded head SHA. No behaviour beyond help/version exists yet.

**Required tests.** Import smoke test; `--help` exit `0`; unknown command exit `2`.

**Evidence.** CI run IDs on both platforms, package metadata dump, checksums of the built wheel/sdist if any.

**Out of scope.** Any config, discovery, runner or scheduler code.

### AT-02 · Configuration and atomic storage

**Scope.** TOML schema `schema = 1` with `[installation]` and `[[folder]]` blocks per §5.2; strict validation that
rejects unknown fields, empty/relative roots, NUL and out-of-range values; canonicalisation; forbidden-root and
cloud-sync checks; `AUTO_TZIM_HOME` handling and permissions `0600`/`0700`; atomic write (temporary file in the same
directory, `flush` + `fsync`, `rename`, directory `fsync`); a corrupt config is never overwritten; `folder add`,
`list`, `show`, `edit`, `enable`, `disable`, `remove` semantics without any file processing; `config validate`.

**Acceptance.** Unknown field → validation error naming the field, exit `2`; a malformed file is left byte-identical
after a failed command; removal of a folder changes no user file; `AUTO_TZIM_HOME` fully relocates config/state/logs.

**Required tests + negative controls.** Unknown field, wrong type, duplicate folder id, root `= ""`, root `= "."`,
root `= "/"`, root `= $HOME`, root inside `AUTO_TZIM_HOME` and vice versa (D-08), a root that is a symlink, a root
that is a file; atomic write interrupted at each of {write, fsync, rename} with an injected fault; corrupt file;
permissions asserted under `umask 000`; write on a read-only directory fails without partial output.

**Evidence.** Config fixtures with expected error strings, fault-injection logs, `ls -l` of created paths.

### AT-03 · Safe discovery, stability, and the dry-run plan

**Scope.** `lstat`-based walk over one approved root, deterministic order (sorted relative POSIX paths), depth
limit, `include_hidden`, `cross_filesystems`, `exclude_globs`, extension allowlist, `min_file_age_seconds`,
identity capture `(st_dev, st_ino, size, mtime_ns)`, `st_nlink > 1` → `hardlink_skip`, symlinks and special files
skipped with a reason, `.tzim-*.tmp` never planned (R-14), canonical containment re-check before every spawn, and a
plan object with the exact argv per file. `scan` renders that plan in human and `--json` form.

**Acceptance.** Same tree → byte-identical plan twice (order and content); no plan entry outside the root; a file
modified after discovery never reaches the runner; no `tzim` process is started by `scan`.

**Required tests + negative controls.** A tree containing: spaces, Unicode (including a non-UTF-8 name on Unix),
leading `-`, newline, shell metacharacters; symlink to file, to dir, out of the root, and a loop; FIFO, socket,
device; hard-linked pair; hidden files; a mount boundary (skipped as environment-dependent when unavailable); file
modified between discovery and pre-run; file deleted between discovery and pre-run; a directory whose entries are
created in a different order on two runs (plan must still match).

**Evidence.** Plan JSON for the hostile-name corpus, per-file rows, the enumeration-order test log.

### AT-04 · Capabilities, policy, planner

**Scope.** The version-keyed adapter (D-01) including `--version` parsing, schema guard, the optional PNG probe, and
`incompatible_tzim` behaviour; the status/exit mapping table (D-02) as a single module; format/transform policy with
`formats = []` meaning *recorded* `tzim` defaults and an empty `transforms` allowlist (D-05); argv construction with
no shell, no `-r`, files only (D-12); the dry-run planner of D-04.

**Acceptance.** For every supported profile the argv is deterministic and contains no user-controlled token other
than the file path; a filename that looks like a flag cannot become one (`--` separator asserted); an unsupported
`tzim` version produces exit `3` before any file is planned; `transforms = ["fits-tile"]` is rejected with a reason.

**Required tests + negative controls.** Fake `tzim` reporting versions `{0.13.9, 0.14.0, 0.15.0, 1.0.0, garbage}`
→ expected adapter outcomes; probe failure; a file literally named `--zopfli`, `-r`, `--json`; a profile with
`formats = ["fits"]` against a build without FITS (must be a documented policy skip, not a crash).

**Evidence.** Golden argv files per profile, adapter decision table executed as a test, fake-`tzim` transcripts.

### AT-05 · JSON runner, state, history, cancellation

**Scope.** One subprocess per file (`argv`, `stdin=DEVNULL`, `stdout=PIPE`, `stderr=PIPE`, `env` minimal), NDJSON
parse and validation, aggregation of exactly one file row + one summary row, timeout with `TERM`→`KILL` (D-10),
`SIGINT`/`SIGTERM` handling that stops new spawns, SQLite state with transactions, a global lock and per-file guard,
stale-run recovery to `interrupted`, history rows per §7.4, and the invalidation key (D-06).

**Acceptance.** No shell; a `tzim` that hangs is killed and recorded as `failed` (`timeout`); a second concurrent run
exits `4` without processing anything; after `SIGKILL` of our own process, the next start marks the stale run
`interrupted` and never `optimized`; a malformed JSON line is `failed`, never `optimized`.

**Required tests + negative controls.** Fake `tzim` producing: each status; exit `1`/`2`/`3`/`4`; truncated JSON;
JSON with a wrong `type`; two summary lines; no summary; huge stdout (stream-limited); 10 MiB of stderr; a binary
that reads stdin and blocks; a binary that ignores `SIGTERM`; a binary that exits `0` while mutating the file
(must be caught by our post-run identity check, not trusted); lock stolen by a second process; concurrent
invocation; crash injection before/after commit.

**Evidence.** SQLite dumps per scenario, transcripts, timeout timings, the crash-recovery log sequence.

### AT-06 · CLI

**Scope.** Every command of §8.1 with `--json`, exact exit codes of §8.2 (plus D-03 once answered), human output on
stdout vs diagnostics on stderr, no prompts in non-TTY, `pause`/`resume`, `status`, `history`, `logs`, and the
first-run gate of §8.1.

**Acceptance.** A table-driven test executes every command in human and `--json` mode and asserts exit code, stdout
JSON validity, and that stdout contains no human diagnostics; `run --once` on a new folder without a preview refuses
and changes nothing.

**Required tests + negative controls.** Each exit code produced by a real invocation; `--json` with a warning on
stderr (stdout must stay parseable); `run` while paused; `run` with a corrupted state DB; `history --status` with an
unknown status (exit `2`); command invoked with a closed stdin.

**Evidence.** Command matrix table with raw captures, exit-code matrix, a diff showing zero byte changes after the
refusal cases.

### AT-07 · Update check

**Scope.** `update-check` only (D-07): one documented request, version comparison against `allowed_major_versions`,
`notify`/`manual` policies, cache, exit `5` on failure, and a documented statement that it never downloads or
replaces anything.

**Acceptance.** No network activity in `run`/`scan` (asserted); an unavailable network is exit `5` with a clear
message; an "up to date" answer is never produced from an error response; an incompatible major is reported as
incompatible and nothing else happens.

**Required tests + negative controls.** Mocked responses: 200 with a newer tag, 200 with an older tag, 403 rate
limit, 429, 500, malformed JSON, HTML error page, a tag that is not semver. A test that runs `run --once` with the
resolver disabled.

**Evidence.** Mocked-response matrix, packet-level assertion that no socket is opened outside `update-check`, the
privacy text as shipped.

### AT-08 · Verification with fake and real `tzim`

**Scope.** A fake `tzim` covering §14.2 (all statuses, malformed/partial output, nonzero exits, timeout, hang,
stdin reads, version mismatch, unexpected file mutation, temporary-file failure, arbitrary filenames), a real-binary
integration suite on a pinned release artifact (D-13) with the T1–T10 assertions of the
[study §4](TZIM-STUDY-2026-10-29.md#4-exact-checks-for-the-real-binary-input-to-at-08), and byte-identity checks for
every non-optimizing outcome.

**Acceptance.** Every T1–T10 assertion runs in CI on macOS and Linux against a checksum-verified artifact; a
mismatch fails the job with the observed JSON attached; every skip/failure case has a pre/post checksum proving the
bytes are identical; a second run of the same corpus makes no new `tzim` invocation.

**Required tests + negative controls.** Positive control (an optimizable PNG must be optimized) and negative
controls (a refused file, an unsupported file, an unstable file: none may change); a deliberately broken expectation
to prove the suite can fail; a fixture renamed to a URL-encoded form to prove no path decoding happens.

**Evidence.** Raw per-case rows, checksums before/after, the artifact's SHA-256 and the `SHA256SUMS` extract, CI run
IDs at the final head.

### AT-09 · Safety documentation and release readiness

**Scope.** `README` (installation, first dry run, what is *not* promised), a configuration reference generated from
the schema, a limitations document, a privacy document (the single request of D-07, local logs, retention), a
recovery document (interrupted runs, stale state, orphaned temporary files), the update policy text, and the
"undo is not provided" statement.

**Acceptance.** Every documented default matches the code (checked by a test that reflects the schema), every exit
code in the docs is produced by a test, and the limitations list includes: dry run needs write access to the target
directory, no undo, no Windows, untested platforms named, `fits-tile` unavailable in v0.1.

**Evidence.** Documentation-to-behaviour table, the test that reflects the schema, a review record of every claim
verified against the binary.

---

## Stage 2 — v0.2 (not assigned)

launchd user agent and systemd user timer, the same core, no second eligibility path, no catch-up storm, plus a TUI
on the same core. Requires: the scheduler cannot prompt, `pause`/`disable` provably change OS state, and overlapping
runs are impossible.

## Stage 3 — v0.3 (not assigned)

Controlled `tzim` update through an explicitly configured official Homebrew tap/formula, blocked while a run is
active, followed by version, capability and binary-hash re-checks and a read-only smoke test. A real in-place upgrade
of a previously installed version in an isolated environment is the acceptance test; a formula review alone is not.

## Stage 4 — v0.4 (not assigned)

Qt GUI (PySide6) as a thin wrapper over the same core: no discovery, subprocess or updater code, dangerous changes
behind a confirmation screen whose default is **No**.

## Explicitly out of every stage

Own codecs or repackers; `--force` or bypassing a refusal; an undo promise; backup or cloud sync; self-update of
`auto-tzim`; replacing the `tzim` binary during an active job; any change in the `tzim` repository.
