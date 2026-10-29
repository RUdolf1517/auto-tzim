# Roadmap and issue breakdown

**Issues exist.** Created 2026-10-29 after the owner's charter v0.2 resolved P-01…P-03:

| ID | GitHub issue | Title |
| --- | --- | --- |
| AT-01 | [#1](https://github.com/RUdolf1517/auto-tzim/issues/1) | Repository and package bootstrap |
| AT-02 | [#2](https://github.com/RUdolf1517/auto-tzim/issues/2) | Config and atomic storage |
| AT-03 | [#3](https://github.com/RUdolf1517/auto-tzim/issues/3) | Safe discovery, stability and the read-only candidate plan |
| AT-04 | [#4](https://github.com/RUdolf1517/auto-tzim/issues/4) | Restricted `tzim` adapter, policy and planner |
| AT-05 | [#5](https://github.com/RUdolf1517/auto-tzim/issues/5) | JSON runner, state, history and cancellation |
| AT-06 | [#6](https://github.com/RUdolf1517/auto-tzim/issues/6) | CLI and the exit-code contract |
| AT-07 | [#7](https://github.com/RUdolf1517/auto-tzim/issues/7) | Update-check (notify only) |
| AT-08 | [#8](https://github.com/RUdolf1517/auto-tzim/issues/8) | Verification with fake and real `tzim` |
| AT-09 | [#9](https://github.com/RUdolf1517/auto-tzim/issues/9) | Safety documentation and release readiness |
| assignment | [#10](https://github.com/RUdolf1517/auto-tzim/issues/10) | Consolidated coder assignment — **prepared, start blocked** |
| upstream | [RUdolf1517/tzim#86](https://github.com/RUdolf1517/tzim/issues/86) | Capabilities and structured skip/refusal reasons (request, not a dependency) |

Known defects in the issue set, recorded rather than hidden — the integration used here may create issues and labels but
every edit, comment and label-attach call returns HTTP 403 ("Resource not accessible by integration"):

1. **#10's body refers to `RUdolf1517/tzim#85`; the upstream request is #86.** One manual edit fixes it; nothing else
   depends on the number.
2. **Labels were created but cannot be attached.** `v0.1` belongs on #1…#9, `assignment` and `manager` on #10, `upstream`
   on nothing here (it describes `tzim#86`).

Both are one-click actions for the owner, with copy-paste commands and verification steps in
[OWNER-ACTIONS.md](OWNER-ACTIONS.md).

Stage boundaries are load-bearing: nothing from v0.2+ may be merged into the MVP pull request, even "while we are here".

---

## Stage 1 — MVP / v0.1 (assigned as one set, #1…#10)

### AT-01 · Repository and package bootstrap

**Scope.** A Python package `auto_tzim` with `core/` and `cli/` only (the `tui/` and `qt/` directories are *not*
created), an entry point `auto-tzim`, `pyproject.toml`, CI on `ubuntu-latest` and `macos-latest` for Python 3.12 and
3.13, and a `lockfile` job that fails on an unpinned dependency.

**Acceptance.** Clean checkout installs; `--help` and `--version` run on both platforms with exit `0`; an unknown
command exits `2`; CI green at a recorded head SHA; no behaviour beyond help/version exists yet.

**Tests.** Import smoke; `--help` exit `0`; unknown command exit `2`; unpinned dependency fails CI.

**Out of scope.** Config, discovery, planner, runner, state, scheduler, TUI, Qt, network.

### AT-02 · Config and atomic storage

**Scope.** TOML schema `1`; strict validation (unknown fields, empty/relative roots, NUL, control characters, duplicate
ids and roots); canonicalisation; forbidden-root, `AUTO_TZIM_HOME` containment (D-08) and cloud-sync checks; permissions
`0600`/`0700`; atomic write with `fsync` of file and directory; corrupt config never overwritten; `config validate` and
`folder add|list|show|edit|enable|disable|remove`.

**Acceptance.** Unknown field ⇒ error naming the field, exit `2`; a rejected edit leaves the file byte-identical;
removal deletes no user file; `AUTO_TZIM_HOME` relocates everything; `transforms` non-empty is rejected (D-05);
`follow_symlinks = true`, `allow_hard_links = true`, `jobs > 1` and `[folder.schedule]` are rejected with the stage that
will own them.

**Tests.** Malformed TOML; unknown field; wrong type; duplicate id/root; roots `""`, `"."`, `"/"`, `$HOME`, a file, a
symlink, inside/containing `AUTO_TZIM_HOME`; atomic write interrupted at write/fsync/rename; corrupt file intact;
permissions under `umask 000`; read-only directory fails without a partial file.

### AT-03 · Safe discovery, stability and the read-only candidate plan

**Scope.** `lstat` walk over one approved root; sorted relative POSIX paths; depth limit; hidden files, mount
boundaries, `exclude_globs`; regular files only; identity `(st_dev, st_ino, size, mtime_ns)`; `st_nlink > 1` ⇒
`hardlink_skip`; age ⇒ `skipped_unstable`; `.tzim-*.tmp` never planned; canonical containment re-check; the candidate
plan of D-04 with plan id and plan hash (D-20).

**Acceptance.** Two scans are byte-identical; no entry outside the root; scan invokes no `tzim` with a path, passes no
`--dry-run`, writes nothing in the target root, and works with the target root non-writable; a file modified or deleted
between stages never reaches the runner; apply without a matching plan exits `6`; every planned row is `planned` and no
output claims eligibility or savings (D-23).

**Tests.** Hostile names (spaces, newline, `;`, `$(...)`, backticks, quotes, leading `-`, `--help`, emoji, CJK,
combining, non-UTF-8); symlink to file/dir/outside/broken, FIFO, socket, device, hard-linked pair, hidden file, orphan
`.tzim-*.tmp`; modified/deleted between stages; two runs with forced different enumeration order; depth beyond limit; 10
000 entries; plan hash mismatch after one file is touched.

### AT-04 · Restricted `tzim` adapter, policy and planner

**Scope.** `tzim --version` only; supported range; `incompatible_tzim` ⇒ exit `3`; one mapping table with the generic
`skipped_by_tzim` (D-02); `formats = []` ⇒ omit `--formats` (D-11); `transforms` empty; argv with files only, no `-r`,
no directories, `--` separator (D-12); binary identity recorded.

**Acceptance.** Deterministic argv per profile with no user-controlled token besides the path; unknown version,
garbage version, missing binary or unknown `schema` ⇒ `3` before planning; no code path spawns `tzim` with `--info`,
`--list-formats`, `--check-deps` or `--dry-run`; `skipped` ⇒ one generic status with the verbatim note; a filename that
looks like a flag cannot become one.

**Tests.** Versions `0.13.9`, `0.14.0`, `0.15.0`, `1.0.0`, garbage, empty output, non-zero exit; files named
`--zopfli`, `-r`, `--json`, `--`, `-`; `formats = ["png"]` and `[]`; a test double that parses `--list-formats` must fail
the source check.

### AT-05 · JSON runner, state, history and cancellation

**Scope.** Per-file subprocess (`shell=False`, `stdin=DEVNULL`, captured output with a size cap, minimal env, **no
PTY**, D-21); NDJSON schema guard; timeout `TERM`→`KILL`; signal handling that starts no new files; SQLite state with
transactions, global/folder/file guards; stale-run recovery to `interrupted`; history; invalidation key (D-06);
post-run identity re-read.

**Acceptance.** No shell, no `pty` import, child sees `isatty() == False`; hung child killed and recorded `failed`;
malformed JSON is `failed`, never `optimized`; a child that exits `0` after mutating a file is caught; concurrent run
exits `8`; no row stays `running`; recovery never yields `optimized`; a second run makes zero invocations.

**Tests.** Each status; exits `0/1/2/3/4`; death by signal; truncated JSON; wrong `type`; two summaries; no summary; huge
stdout; 10 MiB stderr; stdin-blocking child; `SIGTERM`-ignoring child; mutating child; lock stolen; crash before and
after commit; `SIGINT` during a run.

### AT-06 · CLI and the exit-code contract

**Scope.** Every MVP command with `--json`; own codes `0/1/2/3/5/6/7/8/130/143` with `4` unused; fixed preflight order;
precedence `130/143 > 7 > 2 > 8 > 3 > 6 > 5 > 1 > 0`; stdout/stderr separation; no prompts; first-run gate by plan id and
hash.

**Acceptance.** Every code produced by a real invocation, including `6` for no-plan / `pause` / `min_free_bytes` and `8`
for a held lock; `4` never produced; precedence pairs asserted; a raw `tzim` code never becomes `$?`; refusals return
`6`, never `0/1/2`; `--json` stdout stays parseable with warnings on stderr.

**Tests.** Command × code matrix; precedence matrix; unknown `--status` ⇒ `2`; corrupt state ⇒ `7`; corrupt config ⇒ `2`;
apply while paused ⇒ `6`; fabricated plan id/hash ⇒ `6`; one file touched after scan ⇒ `6`; positive control: after a
valid scan ⇒ `0`.

### AT-07 · Update-check (notify only)

**Scope.** One documented request; version comparison against `allowed_major_versions`; `notify`/`manual`; 24 h cache;
exit `5` on failure; nothing downloaded or replaced.

**Acceptance.** No socket outside `update-check`; an error response never rendered as "up to date"; incompatible major
reported and not acted on; the single request documented in the shipped privacy page.

**Tests.** Mocked 200 (newer/older), 403, 429, 500, malformed JSON, HTML page, non-semver tag, timeout; socket-blocking
fixture for every other command; filesystem diff of the binary path and `AUTO_TZIM_HOME`.

### AT-08 · Verification with fake and real `tzim`

**Scope.** Fake suite (every status, exit code, malformed output, hang, stdin read, version mismatch, file mutation);
real suite against a pinned release artifact verified against `SHA256SUMS`, with T1–T10 of the study; the invocation
counter; before/after checksums for every non-candidate outcome; the traceability table filled in.

**Acceptance.** T1–T10 green on macOS and Linux against `tzim v0.14.0`; a mismatch fails with the observed JSON attached;
every skip/refusal/failure has an equal-checksum proof; the counter shows one `--version` and zero path invocations for
scan/dry-run, ≥1 path invocation for apply; a deliberately broken expectation fails the suite; `docs/testing.md` §6
moves from `planned` to `exists`.

### AT-09 · Safety documentation and release readiness

**Scope.** README (installation, first read-only plan, what is not promised), config reference generated from the
schema, limitations, privacy, recovery, update policy, `--help`, the statement that a candidate plan is not a guarantee
of eligibility or gain.

**Acceptance.** Every documented default equals the schema default by reflection; every documented exit code appears in
the AC-8 table; the limitations list is complete for the MVP (no undo, no Windows, untested platforms named, apply
leaves temporary files, no `transforms`, capabilities only as version + binary hash, `skipped_by_tzim` is generic); no
document claims `tzim --dry-run` behaviour or predicted savings.

---

## Stage 2 — v0.2 (not assigned)

launchd user agent and systemd user timer, the same core, no second eligibility path, no catch-up storm, then a TUI on
the same core. Gates: no overlapping runs under a forced schedule, `pause`/`disable` provably change OS state, no prompt
in any non-TTY path, UI contains no discovery/runner/updater code.

## Stage 3 — v0.3 (not assigned)

Controlled update through an explicitly configured official Homebrew tap/formula: lock held for the whole operation,
binary identity before and after, blocked while a run is active, and a real in-place upgrade test from the previously
installed version. A formula review alone is not evidence.

## Stage 4 — v0.4 (not assigned)

Qt GUI (PySide6) as a thin wrapper over the same core, dangerous changes behind a confirmation whose default is **No**,
packaging and notarisation documented before a release.

## Explicitly out of every stage

Own codecs or repackers; `--force`; an undo promise; backup or cloud sync; self-update of `auto-tzim`; replacing the
`tzim` binary during an active job; any change in the `tzim` repository; `tzim --dry-run` in the MVP.
