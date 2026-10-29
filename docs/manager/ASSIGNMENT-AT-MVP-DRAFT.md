# Assignment — MVP / v0.1 (AT-01 … AT-09)

**Status: DRAFT — not issued.** It becomes valid only when the owner answers Q1 (authorisation) in
[QUESTIONS-TO-OWNER.md](QUESTIONS-TO-OWNER.md) and Q4 (the pre-flight exit code), after which the same text is
posted as the body of the MVP tracking issue with an explicit start date. Until then no implementation is
contracted, and work started against this draft would have no acceptance criteria.

Format follows charter §11.

---

**Scope / issue IDs:** AT-01 … AT-09 ([ROADMAP-ISSUES.md](ROADMAP-ISSUES.md)). Everything in stage 1 of that file,
nothing else.

**Final goal of this iteration:** a working `auto-tzim` v0.1 that lets a user add one narrow folder, see a
deterministic plan, run `tzim` on the approved files one process at a time, and read status, history and logs —
with every safety invariant in this document demonstrated by a behavioural test on macOS and Linux at one final
head SHA. Deliver the whole set; the review is consolidated, not per issue.

**Required implementation:**

1. `auto_tzim` package with `core/` and `cli/` (AT-01); no `tui/`, no `qt/`.
2. TOML config schema `1` with strict validation, canonical roots, forbidden-root and cloud-sync checks,
   `AUTO_TZIM_HOME`, permissions, atomic write, corrupt-config safety, and the `folder`/`config` commands (AT-02).
3. Discovery and stability per `SPEC.md` §6, a deterministic plan, and `scan` in human and `--json` form (AT-03).
4. Version-keyed `tzim` adapter (D-01), one status/exit mapping table (D-02), policy and argv construction with no
   shell and no `-r` (D-12), and the two dry-run modes (D-04) (AT-04).
5. Per-file JSON subprocess runner, timeout and signals (D-10), SQLite state with locking and stale-run recovery,
   history per §7.4, invalidation key per D-06 (AT-05).
6. The full CLI of §8.1, `--json` everywhere, the exit codes of §8.2 plus the owner's answer to Q4 (AT-06).
7. `update-check`: one documented request, notify only, exit `5` on failure (D-07) (AT-07).
8. Fake-`tzim` suite for §14.2 and real-binary suite for §14.3 with the T1–T10 assertions of
   [the study §4](TZIM-STUDY-2026-10-29.md#4-exact-checks-for-the-real-binary-input-to-at-08) (AT-08).
9. Documentation set of AT-09, including the honest limitations list.

**Explicit non-goals:** launchd/systemd units or any scheduler; TUI; Qt; automatic or Homebrew update; download or
replacement of any binary; backup, sync or undo; own codecs or repackers; `--force`; Windows support; changes in
the `tzim` repository; `--strip-metadata` and `--fits-tile` opt-ins (empty `transforms` allowlist, D-05);
re-architecting the module layout in §4.2.

**Safety invariants:** all seventeen of charter §6, plus the project-specific rules R-13, R-14, R-18, R-19 in the
[risk register](RISK-REGISTER.md). A pull request that touches any of them is returned, not discussed.

**Required tests and negative controls:** per issue in [ROADMAP-ISSUES.md](ROADMAP-ISSUES.md), and the matrix in
[GATES.md](GATES.md). The non-negotiable ones: hostile filenames never interpreted (`$(…)`, newline, leading `-`,
non-UTF-8); symlink out of the root aborts; a file changed mid-run ends `skipped_unstable`; a `tzim` exit `4` aborts
the plan and marks the rest `interrupted`; a blocked `stdin` is killed by the timeout instead of hanging; a second
concurrent run exits `4` with nothing processed; malformed JSON never becomes `optimized`; a `tzim` that exits `0`
after mutating a file is caught by our identity check; `run`/`scan` open no socket; a second run makes no new
`tzim` invocation; no test sleeps for `min_file_age_seconds` (D-14).

**Required raw artifacts:** CI run IDs per platform at the final head SHA with their logs; the command table with
raw stdout/stderr and exit codes; per-file rows for the fixture corpus including skips and failures; before/after
SHA-256 for every file in the corpus; the pinned `tzim` artifact's SHA-256 and the matching `SHA256SUMS` extract;
SQLite dumps for the recovery scenarios; the fake-`tzim` invocation logs; `python --version`, `uname -a`, the
`auto-tzim` commit, and the `tzim` binary identity per platform.

**Documentation to update:** `README.md` (installation, first dry run, what is not promised), a configuration
reference generated from the schema, limitations, privacy (one request, local logs, retention), recovery, update
policy, `CHANGELOG.md`, and the `--help` text. Documentation is part of acceptance, not a follow-up ticket.

**Commit/PR requirements:** one pull request for the MVP; commits labelled `AT-0N: …` so the issue map can be
reconstructed; no merge commits mixing stages; `CHANGELOG.md` updated; the specification copies untouched; CI green
at the head that is handed over; the PR description contains the handoff block below. The coder does not merge and
does not close issues.

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

- The MVP is accepted on macOS and Linux only; Windows is out of scope and must be named as untested.
- `tzim`'s release archives do not contain the FITS handler, which is why `fits-tile` stays disabled (D-05).
- The real-binary tests use a pinned release artifact verified against the release `SHA256SUMS`; "latest" is
  forbidden (D-13). If GitHub is unreachable in CI, the job fails rather than silently skipping.
- Two upstream gaps are known and accepted (no capability JSON, no reason code): the adapter is fail-closed and
  the first run records the `tzim` version and binary hash, so a future change is detectable.
- `tzim` issue #83 (interactive confirmation) is not compatible with this MVP until its non-TTY behaviour is
  documented; the adapter pins a prompt-free version range.

**Manager's review order:** dependency order and safety first — AT-02 → AT-03 → AT-04 → AT-05 → AT-06 → AT-07 →
AT-08 → AT-09 — with the invariant table of GATES.md checked against source, not against the coder's summary.
