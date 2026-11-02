# Assignment — MVP / v0.1 (AT-01 … AT-09)

> **This file is the source of truth for the assignment.** A copy is published as
> [#10](https://github.com/RUdolf1517/auto-tzim/issues/10); when the two disagree, this file wins — it is versioned and
> the issue is not. Reviewed against decisions D-01…D-36 and the pre-registered gates.


> **Upstream contract moved (2026-11-01).** `tzim` v0.16.0 shipped `--capabilities`, a stable `reason_code` and
> JSON result schema 2; v0.16.1 is the current release and its exit-code precedence is now documented and
> enforced. Statements below written against `tzim` 0.14 that assume no capability document, an untyped
> refusal or an older pin are stale: see the [2026-11-01 audit](../audits/2026-11-01-tzim-v0.16.1.md).

**Status: authorised and started, 2026-11-02.** The text below is the single consolidated assignment for the MVP
(charter §5.1, §11) and is issued as [issue #10](https://github.com/RUdolf1517/auto-tzim/issues/10), which now carries this
file's current text (verified: one trailing blank line of difference).

Two gates that used to block the start are gone, and the record says how:

- **charter §3.2 / blocker B-01 — cleared 2026-10-30** (the `tzim` set finished; PR #84 merged). The blocker block at the
  end of this file is kept as history, not as a live gate.
- **D-25 — satisfied 2026-11-02**: every severity-A finding of the [2026-11-01 audit](../audits/2026-11-01-tzim-v0.16.1.md)
  is closed and the audit itself is closed with all ten findings mapped.

The start was authorised by the owner on 2026-11-02 ([STATUS-LOG](STATUS-LOG.md)), and the assignment was re-read
line by line against D-01…D-36 the same day — the twelve discrepancies that read found are listed in
[READINESS-2026-11-02](READINESS-2026-11-02.md), with what was fixed and what the coder must not follow.

**Start here: AT-01 (#1) only.** It is a skeleton — workspace, two crates, pinned toolchain, CI, `--help`/`--version`,
unknown command ⇒ `2` — and nothing else. No config, no discovery, no runner. The purpose is to settle the shape and the
pipelines on a change that cannot damage a user's file, so the first review is about structure and evidence rather than
about behaviour. AT-02 begins only after AT-01 is reviewed at a final SHA.

Issues in scope: #1, #2, #3, #4, #5, #6, #7, #8, #9. One pull request, commits labelled by issue id.

---

**Scope / issue IDs:** AT-01 (#1) bootstrap · AT-02 (#2) config · AT-03 (#3) discovery and read-only plan ·
AT-04 (#4) restricted adapter, policy, planner · AT-05 (#5) staging, install, runner, state, history · AT-06 (#6) CLI,
modes and exit codes · ~~AT-07 (#7) update-check~~ **not in this iteration** (D-28) · AT-08 (#8) verification with fake
and real `tzim` · AT-09 (#9) documentation readiness.

**Final goal of this iteration:** a working `auto-tzim` v0.1 in which a user adds one narrow folder, gets a **read-only
candidate plan** (no file-processing `tzim`, no write in the target root), then applies it in one of **two modes** —
`confirm`, where the user reads a proposal and originals are deleted file by file only after their verified replacement is
in place, or `auto`, where the same work happens without a question — and can read status, history and logs. Every
invariant of charter §6 **plus 20…23** (D-29) demonstrated by a behavioural test on macOS and Linux at one final head SHA,
and the exit-code contract of D-18/D-19 proven by real invocations. No network, at all (D-28).

**Required implementation:**

1. Rust workspace (D-33) with `auto-tzim-core` and the `auto-tzim` CLI crate only; no `tui`, no `qt` crate.
   `Cargo.lock` committed, `rust-toolchain.toml` pinned, `deny.toml` present, and both manifests carrying the metadata a
   registry requires (name, version, edition, licence, description, repository, `rust-version`). **No CI step runs
   `cargo package`**: packaging is a release step, because it cannot succeed before the core is published (D-42) (#1).
2. TOML config v1 with strict validation, canonical roots, forbidden-root / `AUTO_TZIM_HOME` / cloud-sync checks,
   permissions, atomic write, corrupt-config safety, `config validate` and the `folder` commands — plus the **three state
   primitives** of D-35 that everything after this point sits on: `atomic_write` (temp → write → `fsync` → `rename` →
   `fsync` of the directory), `append_line`, and `create_exclusive` with a start-time discriminator. No C dependency
   anywhere (#2). The run, plan, journal and history *records* built out of those primitives are #5; #2 owns the machine,
   #5 owns what it stores.
3. `lstat` discovery and stability per [`docs/safety.md`](../safety.md) §2, and the candidate plan of D-04 with a plan
   id and a plan hash (D-20) (#3).
4. Restricted adapter: `tzim --version` **and `tzim --capabilities`**, the accepted window `>=0.16.0, <0.17` with
   `capabilities_schema_version: 1` / `result_schema_version: 2`, `incompatible_tzim` fail-closed, one mapping table
   carrying status + the stable `reason` code + the verbatim `note`, argv with files only, no `-r`, no `--formats` when
   `formats = []`, empty `transforms` (#4, D-26/D-27/D-30).
5. Staging and install (D-29), then the per-file runner: copy into `AUTO_TZIM_HOME/staging/<run_id>/`, `tzim` runs on
   the copy, verified result installed by a journaled swap (`rename`-away → `rename`-in → `fsync` → `unlink`), identity
   re-checked immediately before each file, `SIGKILL` at any step leaves a restorable file. Then: `std::process::Command`
   with argv only and no shell, stdin on `/dev/null`, no PTY, NDJSON schema guard, timeout `TERM`→`KILL`, signal
   handling, the **file-based state layer** of D-35 (append-only JSONL, `atomic_write`, one exclusive lock) with
   global/folder/file guards, stale-run recovery, history, invalidation key (#5, AC-14…AC-16, `state.md`).
6. The full MVP CLI with `--json`, **`--mode auto|confirm`** and the single approval gate (fail-closed on no TTY,
   `--json`, EOF; a declined proposal is exit `0` with zero writes), the own exit codes `0/1/2/3/6/7/8/130/143` with `4`
   and `5` unused, the fixed preflight order and the precedence `130/143 > 7 > 2 > 8 > 3 > 6 > 5 > 1 > 0` (#6, AC-17).
7. ~~`update-check`~~ — **not in this iteration** (D-28). Instead, the negative work it left behind: no
   network crate and no `std::net` outside a test guard; no `update-check` command; exit `5` reserved and never produced; AC-9 asserts that
   no command opens a socket, with a planted socket call proving the check can fail (#7 stays open for v0.2).
8. Fake-`tzim` suite, pinned real-binary suite (v0.16.1 by checksum) with the T1–T10 assertions, the invocation counter
   (**two spawns per scan**, zero path invocations, and no path invocation naming a file inside the folder), before/after
   checksums for every non-candidate outcome, and `M-01`…`M-14` for the modes and the install journal (#8).
9. The documentation set of #9 made true against the binary (#9).

**Explicit non-goals:** scheduler or OS units; TUI; Qt; automatic or Homebrew update; any download or replacement of a
binary; backup/sync/undo; own codecs; `--force`; Windows; any change in the `tzim` repository; **any invocation of
`tzim --dry-run`** (charter §6-2); `--strip-metadata` and `--fits-tile` (empty `transforms`); predicted savings or
eligibility claims in the plan; parsing `--info` / `--list-formats` / `note`.

**Safety invariants:** all nineteen of charter §6, **plus 20…23** added by D-29, as the operative text, plus R-13, R-14,
R-18, R-19 and R-22…R-26 of the [risk register](RISK-REGISTER.md). A pull request that weakens any of them is returned,
not discussed.

**Required tests and negative controls:** as listed per issue and in [GATES.md](GATES.md). Non-negotiable ones:

- scan/dry-run makes zero path invocations of `tzim` (two flag-only spawns: `--version`, `--capabilities`) and leaves the
  target root byte-identical, and succeeds when the target root is not writable (positive control: apply does invoke and
  does change a file);
- no `tzim` invocation in any mode names a file inside the folder — the work happens on a staged copy, and this is
  asserted from the invocation log rather than from reading the code (INV-20);
- the same corpus in `confirm` and `auto` produces identical per-file outcomes and an identical install sequence
  (AC-14); a `SIGKILL` at every step of the install journal leaves a restorable file (AC-15);
- apply without a matching plan id/hash, while paused, or below `min_free_bytes` exits `6` with nothing processed;
- every exit code produced by a real command; `4` never produced; a fake `tzim` returning `4` makes us exit `1`;
- precedence pairs asserted with constructed conflicts;
- hostile filenames never interpreted (`$(touch /tmp/pwned)` must not appear); a symlink out of the root aborts; a file
  changed mid-run ends `skipped_unstable`; a blocked stdin is killed by the timeout; a second concurrent run exits `8`
  with nothing processed; malformed JSON never becomes `optimized`; a child that exits `0` after mutating a file is
  caught by our identity check; `run`/`scan` open no socket; a second run makes no new **per-file** spawn (the two flag-only probes are expected); no test sleeps for
  `min_file_age_seconds`.

**Required raw artifacts:** CI run IDs per platform at the final head with logs; the command/exit-code matrix with raw
stdout/stderr and the exit status; per-file rows for the fixture corpus including skips and failures; before/after SHA-256 for every
corpus file; the invocation counter for every scan and apply case; the pinned artifact's SHA-256 and the matching
`SHA256SUMS` line; the state files for each recovery scenario; fake-`tzim` invocation logs; `rustc --version` and the pinned
toolchain,
`uname -a`, the `auto-tzim` head SHA and the `tzim` identity per platform.

**Documentation to update:** `README` (installation, first read-only plan, what is not promised), config reference
generated from the schema, limitations, privacy, recovery, update policy, `--help`, and `CHANGELOG`. Documentation is
part of acceptance.

**Commit/PR requirements:** one pull request; commits labelled `AT-0N: …`; no merge commits mixing stages; the
specification copies untouched; CI green at the head handed over; the PR description carries the handoff block below.
The coder does not merge and does not close issues.

**Final handoff format:** charter §12, verbatim:

```text
Final head SHA:
Issues and commits:
Implemented behavior:
Non-goals preserved:
CI runs and artifacts:
Raw evidence locations:
Independent commands/results:
Tests added and negative controls:
Docs updated:
Known limitations:
Remaining blocker: none / exact blocker
Merge/Release performed by coder: no
```

**Known external constraints:**

- Accepted on macOS and Linux only; Windows untested and must be named as such.
- The real-binary suite pins `tzim v0.16.1` artifacts verified against the release `SHA256SUMS`; `latest` is forbidden.
  If GitHub is unreachable in CI the job fails rather than silently skipping.
- **`tzim` now has both interfaces** and the MVP consumes them (D-26, D-27): `--capabilities` is read and schema-checked
  (capabilities schema 1, result schema 2), and the stable `reason_code` is passed through as `reason` beside the verbatim
  `note`. The default set is still delegated — `--formats` is never passed — and observed rather than promised. The request
  that produced this, [tzim#85](https://github.com/RUdolf1517/tzim/issues/85), was implemented in PR #91 and shipped in
  v0.16.0; my duplicate #86 is closed.
- **There is no network code in the MVP** (D-28): `update-check` moves to v0.2, exit `5` is reserved, and AC-9 asserts
  that no command opens a socket.
- **Two modes, one install path** (D-29): `confirm` (default) shows a proposal and installs only on approval; `auto` does
  not ask. Staged copies, a journaled swap, an original deleted only after its verified replacement is in place.
- `tzim`'s FITS confirmation (#83) shipped with a documented non-TTY refusal: no prompt for non-TTY, `--json`,
  `--dry-run`, recursive or multi-file runs. Our adapter never enables `--fits-tile` (empty `transforms`), and every
  child gets `Stdio::null()` on stdin, so the case cannot arise — and `R-01` still proves that a child waiting on stdin is killed.
- Apply-time temporary files appear **in our staging directory**, not in the target root: `tzim` runs on the copy. A hard
  kill can therefore leave `.tzim-<pid>-<ns>-<attempt>.tmp` or a staged copy inside `AUTO_TZIM_HOME/staging/<run_id>/`;
  the next start removes them and says so. Nothing of `tzim`'s ever lands in the user's folder.

**Review order:** dependency and safety first — #1 → #2 → #3 → #4 → #5 → #6 → #8 → #9, with #7 struck by D-28 — against
[GATES.md](GATES.md) and the source, not against the summary.

---

## Blocker B-01 — cleared 2026-10-30 (kept as the record, charter §10 format)

**Cleared.** `tzim` PR #84 merged on 2026-10-30T14:55:59Z; the upstream tracker has no open pull requests and no open
issues; v0.16.1 is the current release. Nothing in this block is a live instruction — it is what the start was waiting
for in October, preserved because a blocker that disappears without a trace teaches nobody anything.

```text
Blocker ID: B-01
Affected issue: #10 (start only; the content of #1…#9 is not blocked)
Exact operation/input/URL: https://github.com/RUdolf1517/tzim/pull/84 and its two failing jobs
  (run 40939520316 "measure the frozen 21-PDF set", run 40939520233 "re-run the published measurements")
Exact tool/version/environment: tzim main d4e4597; PR #84 branch arena/99418c32-tzim; gh CLI in this session
Observed error and evidence: `gh pr view 84` → state OPEN, MERGEABLE, updated 2026-10-29T19:26:30Z;
  `gh pr checks 84` → both measurement jobs fail; `gh issue list --repo RUdolf1517/tzim` → #57, #59, #78, #79, #80,
  #82, #83 open
What was already tried: recorded in docs/manager/PREFLIGHT-2026-10-29.md P2; nothing further is possible from here
Why it cannot be solved inside the repo: charter §3.2 forbids starting a new project as a reason to leave the previous
  set unfinished; the failing jobs belong to the tzim repository and to that PR
Single exact owner/external action required: merge or explicitly park PR #84 (and say which of #57/#59/#78–#83 are
  deferred), then comment "start" on #10
Independent work continuing meanwhile: the issue set, this assignment, the acceptance matrix and the documentation are
  complete; implementation, tests and fixtures must not begin before that comment
```
