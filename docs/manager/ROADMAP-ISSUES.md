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
| AT-07 | [#7](https://github.com/RUdolf1517/auto-tzim/issues/7) | Update-check (notify only) — **removed from the MVP by D-28**; the issue stays open, relabelled for v0.2 |
| AT-08 | [#8](https://github.com/RUdolf1517/auto-tzim/issues/8) | Verification with fake and real `tzim` |
| AT-09 | [#9](https://github.com/RUdolf1517/auto-tzim/issues/9) | Safety documentation and release readiness |
| assignment | [#10](https://github.com/RUdolf1517/auto-tzim/issues/10) | Consolidated coder assignment — **prepared**; the body needs the D-26…D-29 rewrite below before it can be started |
| upstream | [RUdolf1517/tzim#85](https://github.com/RUdolf1517/tzim/issues/85) | Capabilities and structured result reasons (request, not a dependency). **Canonical** — it existed before I filed my own #86, which is a duplicate |

Known defects in the issue set, recorded rather than hidden — the integration used here may create issues and labels but
every edit, comment and label-attach call returns HTTP 403 ("Resource not accessible by integration"):

1. **I filed a duplicate upstream issue.** `tzim#86` repeats `tzim#85`, which was created twelve minutes earlier and
   says the same thing better. #85 is canonical; #86 should be closed as a duplicate. Nothing in #10 needs editing — its
   reference to #85 was correct, and my earlier "correction" of it was the mistake. Root cause: I filed the upstream
   request without re-listing the open issues first.
2. **Labels were created but cannot be attached.** `v0.1` belongs on #1…#9, `assignment` and `manager` on #10, `upstream`
   on nothing here (it describes `tzim#86`).

Both are one-click actions for the owner, with copy-paste commands and verification steps in
[OWNER-ACTIONS.md](OWNER-ACTIONS.md).

Stage boundaries are load-bearing: nothing from v0.2+ may be merged into the MVP pull request, even "while we are here".

---

## Versions and issues

The product roadmap lives in [docs/roadmap.md](../roadmap.md); this page is the work breakdown for the version currently
being delivered. Only one version has issues.

| Version | Issues | State | Acceptance |
| --- | --- | --- | --- |
| v0.1 — MVP | #1…#9 (AT-01…AT-09), assignment #10 | **prepared**, not started (AT-07 struck by D-28) | [GATES.md](GATES.md) — 17 rows, 23 invariants at one final SHA |
| v0.2 — scheduler, TUI, `update-check` | none yet | deliberately not created | written before its first line, from [roadmap §3](../roadmap.md) |
| v0.3 — controlled update | none yet | deliberately not created | charter §11.3 real upgrade test, from [roadmap §4](../roadmap.md) |
| v0.4 — Qt GUI | none yet | deliberately not created | from [roadmap §5](../roadmap.md) |

**Why v0.2+ has no issues.** An issue set written today would describe software whose contracts have not been through a
single release, and every one of them would be rewritten after the first real user report. The version is committed in the
roadmap; the breakdown is written when the version starts, in the same week, against the GATES text that will judge it.
This is not a queue of promises — it is a refusal to pretend we know v0.3's details before v0.1 exists.

The v0.2 set, when it is written, inherits one constraint from v0.1 that cannot be renegotiated: the scheduler calls the
same `run --once` and has no eligibility rules of its own, and the mode gate of D-29 is not bypassable by any front end.

---

## Stage 1 — MVP / v0.1 (assigned as one set, #1…#10)

### AT-01 · Repository and package bootstrap — **accepted 2026-11-02** at `017ae80`

> Merged as `fc17b9b`; verdict and evidence in [REVIEW-017ae80](reviews/REVIEW-017ae80.md). Six acceptance points
> reproduced by the reviewer, three Low follow-ups named (help subcommand, the Python pin checker, a deprecated checkout
> action). AT-02 has not started: each stage begins on the owner's word.

**Scope.** A Rust workspace (D-33) with two crates, `auto-tzim-core` (library) and `auto-tzim` (binary) — the names are
fixed by D-36 because they are published and a crate name is forever; the `tui` and `qt` crates are *not* created.
`Cargo.toml` with `[[bin]] name = "auto-tzim"`, committed `Cargo.lock`, `rust-toolchain.toml` pinning the MSRV,
`deny.toml`, CI on `ubuntu-latest` and `macos-latest` on the pinned toolchain, and a job that fails on an unpinned
dependency.

**Acceptance.** A clean checkout builds and runs; `--help` and `--version` exit `0` on both platforms; an unknown command
exits `2`; both manifests carry the metadata a registry requires — *publishable* is a metadata property, and packaging
itself is a release step that cannot run before the core is published (D-42); `cargo install --path crates/auto-tzim`
produces a binary named `auto-tzim`; CI green at a recorded head SHA; no behaviour beyond help/version exists yet.

**Tests.** Build and run smoke on both platforms; `--help` exit `0`; unknown command exit `2`; a dependency without a
version pin fails CI; the installed binary's name is `auto-tzim` and its `--version` prints one line.

**Out of scope.** Config, discovery, planner, runner, state, scheduler, TUI, Qt, network.

**Brief.** [HANDOFF-AT-01](HANDOFF-AT-01.md) — the six acceptance points, the negative control that is part of the
deliverable, and the two traps already present in the documents.

### AT-02 · Config and atomic storage — **ACCEPTED 2026-11-05 at `d0ac17e`, merged as `70ab3d0`**

> Coder brief: [HANDOFF-AT-02](HANDOFF-AT-02.md) — 12 acceptance points, five negative controls, the F-18 metadata check
> folded in, and the three decisions the documents were missing (D-43/D-44/D-45). **Part B** of the same brief fixes the
> macOS queue ([D-51](DECISIONS.md), [R-27](RISK-REGISTER.md)): the owner asked for it "заодно". The stage begins when the
> owner says so in #2.

**Scope.** TOML schema `1`; strict validation (unknown fields, empty/relative roots, NUL, control characters, duplicate
ids and roots); canonicalisation; forbidden-root, `AUTO_TZIM_HOME` containment (D-08) and cloud-sync checks; permissions
`0600`/`0700`; atomic write with `fsync` of file and directory; corrupt config never overwritten; `config validate` and
`folder add|list|show|edit|enable|disable|remove`.

**Acceptance.** Unknown field ⇒ error naming the field, exit `2`; a folder is identified by its own name ([D-43](DECISIONS.md)); **`config.toml` with zero folders is valid and the program still runs a path given on the command line ([D-46](DECISIONS.md))**; a rejected edit leaves the file byte-identical;
removal deletes no user file; `AUTO_TZIM_HOME` relocates everything; `transforms` non-empty is rejected (D-05);
`follow_symlinks = true`, `allow_hard_links = true`, `jobs > 1` and `[folder.schedule]` are rejected with the stage that
will own them.

**Tests.** Malformed TOML; unknown field; wrong type; duplicate name/root; roots `""`, `"."`, `"/"`, `$HOME`, a file, a
symlink, inside/containing `AUTO_TZIM_HOME`; atomic write interrupted at write/fsync/rename; corrupt file intact;
permissions under `umask 000`; read-only directory fails without a partial file.

**Outcome.** Green at `d0ac17e` on Linux and macOS (run `42046562705`), accepted in
[REVIEW-d0ac17e](reviews/REVIEW-d0ac17e.md), merged as `70ab3d0`. Seven negative controls were run and reverted, and each
red run's SHA was verified as an ancestor of the commit that removed it — including the one that put the F-22 trap back and
failed the acceptance test itself on both platforms. Two Low findings carried into AT-03: **F-26** (the internal-error
branch returns `2`, the contract says `7`) and **F-27** (`structural_path` does syscalls despite the name).

### AT-03 · Safe discovery, stability and the read-only candidate plan — **NEXT, brief ready**

> Coder brief: [HANDOFF-AT-03](HANDOFF-AT-03.md) — 13 acceptance points, five negative controls, and the four numbers that
> make a plan reproducible ([D-52](DECISIONS.md): id, hash, order, depth). AT-02 is accepted, so this stage is next; it
> begins when the owner says so in #3, and its first commit carries F-26 and F-27.

**Scope.** `lstat` walk over each approved root of the batch ([D-57](DECISIONS.md)); sorted relative POSIX paths; depth limit; hidden files, mount
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

**Scope.** `tzim --version` **and `tzim --capabilities`** (D-26); accepted window `>=0.16.0, <0.17` with
`capabilities_schema_version: 1` / `result_schema_version: 2` (D-30); `incompatible_tzim` ⇒ exit `3`; one mapping table
carrying `status`, the stable `reason` code (D-27) and the verbatim `note`; `formats = []` ⇒ omit `--formats` (D-11);
`transforms` empty; argv with files only, no `-r`, no directories, `--` separator (D-12); binary identity and capability
document recorded in history.

**Acceptance.** Deterministic argv per profile with no user-controlled token besides the path; unknown version, garbage
version, missing binary, missing capability document, unknown `capabilities_schema_version`, unknown
`result_schema_version` ⇒ `3` **before planning**; no code path spawns `tzim` with `--info`, `--list-formats`,
`--check-deps` or `--dry-run`; a refusal keeps the verbatim note **and** the reason code, with the two never mixed; a
filename that looks like a flag cannot become one.

**Tests.** Versions `0.15.0`, `0.16.0`, `0.16.1`, `0.17.0`, `1.0.0`, garbage, empty output, non-zero exit; capability
documents with each schema number wrong in turn, a missing field we depend on, and an unknown field we must tolerate;
files named
`--zopfli`, `-r`, `--json`, `--`, `-`; `formats = ["png"]` and `[]`; a test double that parses `--list-formats` must fail
the source check.

### AT-05 · JSON runner, state, history and cancellation

> Also owns `state/last-compression.json` and the `--time` cooldown it feeds ([D-47](DECISIONS.md)): consumed when
> processing starts, keyed by canonical root, never consumed by `--dry-run`.

**Scope.** **Staging and install** (D-29): copy into `AUTO_TZIM_HOME/staging/<run_id>/`, `tzim` runs on the copy,
journaled install (`rename`-away → `rename`-in → `fsync` → `unlink`), identity re-check immediately before each file;
per-file subprocess (one `Command::arg` per item, no shell, stdin on `/dev/null`, captured output with a size cap, minimal env, **no PTY**, D-21);
NDJSON schema guard; timeout `TERM`→`KILL`; signal handling that starts no new files; file-based state (D-35): run
directories with append-only `files.jsonl`, atomic `run.json` replacement, an exclusive-create run lock with a start-time
discriminator; stale-run recovery to `interrupted` and install recovery from the journal; history; the `reason` field
(D-27); invalidation key (D-06).

**Acceptance.** No shell, no pseudo-terminal, child sees `is_terminal() == false`; hung child killed and recorded `failed`;
malformed JSON is `failed`, never `optimized`; a child that exits `0` after mutating a file is caught; **no `tzim`
invocation ever receives a path inside the user's folder**; a `SIGKILL` at any journal step leaves the file restorable
(AC-15); a file changed after planning is never installed over (AC-16); concurrent run exits `8`; no row stays
`running`; recovery never yields `optimized`; a second run makes zero **per-file** spawns, with only the two flag-only
probes at start.

**Tests.** Cases `R-01`…`R-15`, `M-01`…`M-14` and `L-01`…`L-06` of [FIXTURE-CORPUS.md](FIXTURE-CORPUS.md): each status; exits
our exits `0/1/2/3/7/130/143` and a fake `tzim` exit `4` arriving as our `1`; death by signal; truncated JSON; wrong `type`; two summaries; no summary; huge stdout; 10 MiB stderr;
stdin-blocking child; `SIGTERM`-ignoring child; mutating child; lock held; crash before and after commit; `SIGINT`.

### AT-06 · CLI and the exit-code contract

**Scope.** The path-first one-shot form `auto-tzim PATH [PATH…] [--time HOURS] [--auto] [--force]` ([D-46](DECISIONS.md), [D-47](DECISIONS.md), [D-49](DECISIONS.md) for the batch rules), the two visible modes ([D-48](DECISIONS.md): `confirm` prints the plan and asks, `auto` prints nothing), every MVP command
with `--json`; **`--auto` / `--mode auto|confirm`** and the proposal gate (D-29), including the
fail-closed rules for a missing terminal, `--json`, EOF and a closed stdin; own codes `0/1/2/3/5/6/7/8/130/143` with `4`
and (in v0.1) `5` unused; fixed preflight order extended with the staging free-space check; precedence
`130/143 > 7 > 2 > 8 > 3 > 6 > 5 > 1 > 0`; stdout/stderr separation; the plan a `confirm` run showed is the plan it executes, and `--plan <ID>` still binds an earlier one.

**Acceptance.** Every code produced by a real invocation, including `6` for the cooldown `too_soon` / a stale `--plan` / `pause` / `min_free_bytes` /
non-TTY `confirm` and `8` for a held lock; `4` and `5` never produced; precedence pairs asserted; a raw `tzim` code never
becomes our exit status; refusals return `6`, never `0/1/2`; **a declined proposal returns `0` with zero installs** (a refusal by the
user is an outcome, not an error); `--json` stdout stays parseable with warnings and the proposal on stderr.

**Tests.** Command × code matrix; precedence matrix; unknown `--status` ⇒ `2`; corrupt state ⇒ `7`; corrupt config ⇒ `2`;
apply while paused ⇒ `6`; fabricated plan id/hash ⇒ `6`; one file touched after scan ⇒ `6`; positive control: after a
valid scan ⇒ `0`.

### AT-07 · ~~Update-check (notify only)~~ — **removed from the MVP, deferred to v0.2** (D-28)

**Why it left.** It was the only network code in v0.1, and its value before anyone has installed v0.1 is close to zero.
Removing it makes the privacy page one paragraph instead of a section with an exception, removes a whole failure surface
(timeouts, proxies, rate limits, TLS), and lets `invariant 15` become absolute: *no request, of any kind, ever*.

**What replaces it in the MVP.** Nothing — that is the point. The only work is negative, and it is still tested: AC-9 now
asserts that **no command opens a socket** — structurally, because no crate in `Cargo.lock` can open one, and by running
every command with no route and no DNS — and that `update-check`
does not exist. Exit code `5` is reserved and unused, so v0.2 can land under the original rules without renumbering.

**When it returns (v0.2):** one documented `GET`, notify only, 24 h cache, exit `5` on failure, opt-out, and the privacy
page updated *before* the code is merged rather than after.

### AT-08 · Verification with fake and real `tzim`

**Scope.** Fake suite (every status, exit code, malformed output, hang, stdin read, version mismatch, file mutation);
real suite against a pinned release artifact verified against `SHA256SUMS`, with T1–T10 of the study; the invocation
counter; before/after checksums for every non-candidate outcome; the traceability table filled in.

**Acceptance.** T1–T10 green on macOS and Linux against `tzim v0.16.1`; a mismatch fails with the observed JSON attached;
every skip/refusal/failure has an equal-checksum proof; the counter shows two flag-only probes and zero path invocations
for scan/dry-run, and ≥1 path invocation naming a staged copy for apply; a deliberately broken expectation fails the suite; `docs/testing.md` §6
moves from `planned` to `exists`.

### AT-09 · Safety documentation and release readiness

**Scope.** README (installation, first read-only plan, what is not promised), config reference generated from the
schema, limitations, privacy, recovery, update policy, `--help`, the statement that a candidate plan is not a guarantee
of eligibility or gain.

**Acceptance.** Every documented default equals the schema default by reflection; every documented exit code appears in
the AC-8 table; the limitations list is complete for the MVP (no undo, no Windows, untested platforms named, apply
leaves temporary files only in staging, no `transforms`, capabilities read from the document and recorded as version +
hash + both schema numbers per D-26, and a `tzim` skip carrying **both** the stable `reason` and the verbatim `note` per
D-27); no
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

Qt GUI as a **separate binary and package** built on the same core crate (D-34), dangerous changes behind a confirmation
whose default is **No**,
packaging and notarisation documented before a release.

## Explicitly out of every stage

Own codecs or repackers; `--force`; an undo promise; backup or cloud sync; self-update of `auto-tzim`; replacing the
`tzim` binary during an active job; any change in the `tzim` repository; `tzim --dry-run` in the MVP.
