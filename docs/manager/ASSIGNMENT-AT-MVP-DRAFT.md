# Assignment — MVP / v0.1 (AT-01 … AT-09)

**Status: prepared, not started.** The text below is issued as
[issue #10](https://github.com/RUdolf1517/auto-tzim/issues/10) and is the single consolidated assignment for the MVP
(charter §5.1, §11). Starting implementation is blocked by charter §3.2 — the previously assigned `tzim` set is still
unfinished ([pre-flight P2](PREFLIGHT-2026-10-29.md), blocker B-01 in the issue). Nothing in this document authorises a
coder to begin before the owner comments "start" on #10.

Issues in scope: #1, #2, #3, #4, #5, #6, #7, #8, #9. One pull request, commits labelled by issue id.

---

**Scope / issue IDs:** AT-01 (#1) bootstrap · AT-02 (#2) config · AT-03 (#3) discovery and read-only plan ·
AT-04 (#4) restricted adapter, policy, planner · AT-05 (#5) runner, state, history · AT-06 (#6) CLI and exit codes ·
AT-07 (#7) update-check · AT-08 (#8) verification with fake and real `tzim` · AT-09 (#9) documentation readiness.

**Final goal of this iteration:** a working `auto-tzim` v0.1 in which a user adds one narrow folder, gets a **read-only
candidate plan** (no file-processing `tzim`, no write in the target root), then applies it with `tzim` invoked one file
per process, and can read status, history and logs — with every invariant of charter §6 demonstrated by a behavioural
test on macOS and Linux at one final head SHA, and the exit-code contract of D-18/D-19 proven by real invocations.

**Required implementation:**

1. `auto_tzim` package with `core/` and `cli/` only; no `tui/`, no `qt/` (#1).
2. TOML config v1 with strict validation, canonical roots, forbidden-root / `AUTO_TZIM_HOME` / cloud-sync checks,
   permissions, atomic write, corrupt-config safety, `config validate` and the `folder` commands (#2).
3. `lstat` discovery and stability per [`docs/safety.md`](../safety.md) §2, and the candidate plan of D-04 with a plan
   id and a plan hash (D-20) (#3).
4. Restricted adapter: `tzim --version` only, `incompatible_tzim` fail-closed, one status/exit mapping table with the
   generic `skipped_by_tzim`, argv with files only, no `-r`, no `--formats` when `formats = []`, empty `transforms`
   (#4).
5. Per-file runner: `shell=False`, `stdin=DEVNULL`, no PTY, NDJSON schema guard, timeout `TERM`→`KILL`, signal handling,
   SQLite state with global/folder/file guards, stale-run recovery, history, invalidation key (#5).
6. The full MVP CLI with `--json`, the own exit codes `0/1/2/3/5/6/7/8/130/143` (`4` unused), the fixed preflight order
   and the precedence `130/143 > 7 > 2 > 8 > 3 > 6 > 5 > 1 > 0` (#6).
7. `update-check`: one documented request, notify only, exit `5` on failure (#7).
8. Fake-`tzim` suite, pinned real-binary suite with the T1–T10 assertions, the invocation counter, before/after
   checksums for every non-candidate outcome (#8).
9. The documentation set of #9 made true against the binary (#9).

**Explicit non-goals:** scheduler or OS units; TUI; Qt; automatic or Homebrew update; any download or replacement of a
binary; backup/sync/undo; own codecs; `--force`; Windows; any change in the `tzim` repository; **any invocation of
`tzim --dry-run`** (charter §6-2); `--strip-metadata` and `--fits-tile` (empty `transforms`); predicted savings or
eligibility claims in the plan; parsing `--info` / `--list-formats` / `note`.

**Safety invariants:** all nineteen of charter §6 as the operative text, plus R-13, R-14, R-18, R-19 and R-22…R-26 of
the [risk register](RISK-REGISTER.md). A pull request that weakens any of them is returned, not discussed.

**Required tests and negative controls:** as listed per issue and in [GATES.md](GATES.md). Non-negotiable ones:

- scan/dry-run makes zero path invocations of `tzim` and leaves the target root byte-identical, and succeeds when the
  target root is not writable (positive control: apply does invoke and does change a file);
- apply without a matching plan id/hash, while paused, or below `min_free_bytes` exits `6` with nothing processed;
- every exit code produced by a real command; `4` never produced; a fake `tzim` returning `4` makes us exit `1`;
- precedence pairs asserted with constructed conflicts;
- hostile filenames never interpreted (`$(touch /tmp/pwned)` must not appear); a symlink out of the root aborts; a file
  changed mid-run ends `skipped_unstable`; a blocked stdin is killed by the timeout; a second concurrent run exits `8`
  with nothing processed; malformed JSON never becomes `optimized`; a child that exits `0` after mutating a file is
  caught by our identity check; `run`/`scan` open no socket; a second run makes no new invocation; no test sleeps for
  `min_file_age_seconds`.

**Required raw artifacts:** CI run IDs per platform at the final head with logs; the command/exit-code matrix with raw
stdout/stderr and `$?`; per-file rows for the fixture corpus including skips and failures; before/after SHA-256 for every
corpus file; the invocation counter for every scan and apply case; the pinned artifact's SHA-256 and the matching
`SHA256SUMS` line; SQLite dumps for the recovery scenarios; fake-`tzim` invocation logs; `python --version`,
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
- The real-binary suite pins `tzim v0.14.0` artifacts verified against the release `SHA256SUMS`; `latest` is forbidden.
  If GitHub is unreachable in CI the job fails rather than silently skipping.
- `tzim` has no machine-readable capability list and no structured refusal reason: the MVP delegates the default set
  (no `--formats`), keeps `skipped_by_tzim` generic, and records version + binary hash. The upstream request
  [RUdolf1517/tzim#86](https://github.com/RUdolf1517/tzim/issues/86) describes the future API and is **not** a
  dependency. (Issue #10's body cites `#85` for this; the correct number is **#86** — correction recorded here because
  the integration that created #10 can neither edit nor comment on issues.)
- `tzim` issue #83 (interactive confirmation) is incompatible with this MVP until its non-TTY behaviour is documented;
  the adapter pins a prompt-free range.
- Apply-time temporary files inside the target root are `tzim`'s behaviour
  (`.tzim-<pid>-<ns>-<attempt>.tmp`) and may survive a hard kill; documented, never deleted by us, never planned.

**Review order:** dependency and safety first — #2 → #3 → #4 → #5 → #6 → #7 → #8 → #9 — against [GATES.md](GATES.md) and
the source, not against the summary.

---

## Blocker B-01 (charter §10 format)

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
