# Review — f4016d03 (AT-03, PR #46)

Reviewer: manager (this session) · Date: 2026-11-07
Scope: **AT-03 (#3)** — safe discovery, stability and the read-only candidate plan, on top of `main` `1421f47`.
Head: `f4016d03d2323565578c8165ff81e76c27e4d740`, 12 files vs the PR base (+3501 −25), draft.

**Verdict: PASS — AT-03 is complete at `f4016d03`, with two obligations carried to the MVP gate.** Owner accepted the stage
on 2026-11-07 (*"1"* to the acceptance question). `#46` stays a draft: the merge of product code is the owner's word, and it
has not been given. F-3 and F-4 below are recorded as **blocking at AT-08**, not as waived.

Levels used throughout, as [GATES](../GATES.md) §3 demands: **recomputed** (I re-derived the artifact from raw bytes),
**executed** (a command I ran against this repository or its API), **source-inspected** (I read the code),
**coder-reported** (words only — a finding, not a pass).

## 1. Binding

| Check | Level | Result |
| --- | --- | --- |
| Head SHA is the claimed one | executed | `gh api pulls/46` → `head.sha = f4016d03…`, `base.sha = 1421f47…` = `main`, `mergeable=true`, `draft=true` |
| The run tested that SHA | executed | Run `42416157628`, `head_sha = f4016d03…`, `event=pull_request`, attempt 1, `conclusion=success` |
| Every required job ran and passed | executed | `test (ubuntu-24.04)`, `test (macos-15)`, `pinned-dependencies`, `deny`, `build` — five of five `success`; no `skipped`, `cancelled`, `neutral` |
| Commits are labelled by issue | source-inspected | 27 commits in `1421f47..f4016d03`, every subject prefixed `AT-03:`, including the five control/revert pairs |

## 2. What I recomputed rather than believed

| Claim | Level | Evidence |
| --- | --- | --- |
| `plan_hash` is the SHA-256 of the ordered canonical rows and excludes `plan_id` and `created` | recomputed | I decoded the two real plan files out of run `42416157628`'s annotations. Concatenating their `rows` as `{path_raw_b64,size,dev,ino,mtime_ns,argv}` and hashing gives `711780457935cb3976695a67982e901c06257661ce1ccaf5100a84d3514e1580` — the `plan_hash` in **both** documents. The documents differ only in `plan_id` (`890758d6…` / `ba03d0d4…`) and `created` (`…785831163Z` / `…789136985Z`) |
| The plan document carries `root` **and** `root_raw_b64` | recomputed | Both files: `"root":"/tmp/tmp.jJcIk6axIS/root"`, `"root_raw_b64":"L3RtcC90bXAuakpjSWs2YXhJUy9yb290"` |
| Two scans of an unchanged tree have the same content | recomputed | `rows` identical byte for byte, `plan_hash` equal, ids and timestamps different — exactly D-59, and the whole-file hash therefore differs by design |
| Scan and dry-run are read-only | executed | The before/after `find` listings and the per-file SHA-256 sets in the same run decode to identical bytes; the second scan ran with the root `chmod -R a-w` and still exited `0`; two plan files exist |
| Five negative controls were red for the right reason | executed | Runs `42292177393`, `42292325016`, `42292513072`, `42292691141`, `42381182145` — all `completed/failure` on the claimed SHAs, `event=pull_request`; in each, the two failing jobs are `test (…)` while `deny`/`build`/`pinned-dependencies` are green, so the red is a test failure, not infrastructure |
| Each control was reverted, not papered over | executed | `git diff --quiet <control>~1 <revert>` is clean for all five pairs; all five reverts are ancestors of `f4016d03` |
| The hostile-name corpus is N-01…N-17 | recomputed | The 19 names in `at03_names_corpus_emits_one_planned_row_per_constructible_name` are exactly the corpus rows (N-15 is three names). CI prints `AT03_NAMES planned=19 corpus_names=19 non_utf8_skipped=0` on Ubuntu and `planned=18 non_utf8_skipped=1` on macOS |
| D-65 budget | executed | `AT03_BUDGET_10000_MS=160` on Ubuntu 24.04 debug, fixture creation excluded (limit 15 000); `276` on macOS, informational |
| K-08, K-06, K-05, K-14 | executed / source-inspected | `AT03_DEVICE /dev/null skipped_policy=character_device opened=no`; the FIFO case runs on CI with `--ignored` under a 2-minute step timeout; the loop test asserts completion under 5 s; the depth test records one limit per boundary directory |
| Mount boundary | executed | `AT03_MOUNT /proc boundary=1 crossed=0` on Linux; macOS does not run it |

## 3. Source inspection of the decisions

| Decision | Where | Result |
| --- | --- | --- |
| D-59 argv | `planner::arguments` | sorted `--formats` (byte order) → `--` → relative POSIX path; no `--json`, no absolute or staged path |
| D-59 rows | `planner::canonical_rows` | `path_raw_b64,size,dev,ino,mtime_ns,argv`, with `{"b64":…}` for a non-UTF-8 argument |
| D-60 publication | `planner::publish` | temp `0600` in a `0700` directory, `hard_link`, unlink, `sync_all` on the directory; a collision yields `plan_id_collision`, no fallback |
| D-61 gate order | `discovery::visit` | `.tzim-*.tmp` returns first (one diagnostic per batch, never a row), then excluded name → hidden → symlink → directory/type gates → mount boundary → identity re-check → `nlink>1` before age (`stability::refusal`) → format → `planned` |
| D-63 glob grammar | `discovery::glob` | `*` never crosses `/`, `**` does, `?` is one non-`/` byte, everything else literal, bounded memory by path length; an empty pattern is rejected at config load (`config.rs`, `"exclude_globs: empty pattern"`) |
| D-52 hash primitives | `hash.rs` | FIPS 180-4 SHA-256 with no dependency, tested on empty, `abc`, 1 MiB and the 55/56/63/64/65/119/120/127-byte padding boundaries |
| D-53 exit map | `main.rs::exit_code` | `Internal→7`, `TreeChanged→6`, `Locked→8`, `Invalid|Io→2`; a unit test asserts `Internal→7` |
| D-35/D-38 structural claims | `crates/auto-tzim-core/src/*.rs` | no `unsafe`, no `libc`, no `Command`, no `thread::spawn`; `main.rs` reaches only `std::process::exit` |

## 4. Findings

1. **F-1 — was mine, fixed.** `HANDOFF-AT-03` item 1 demanded two plan files with an equal whole-file `sha256sum` **and** a
   different `plan_id`. Under D-59 both `plan_id` and `created` are inside the document, so equal files are impossible: the
   brief contradicted the decision it was testing. Corrected in `#49` (`b05ccde`); the coder's evidence was right.
2. **F-2 — Low, carried.** Item 19 is proven at the planner level (a `0600` file in a `0700` directory, the first file
   byte-identical after the collision, no temp left) plus a separate unit test on `Internal→7`. There is no single CLI run
   that exits `7` on a colliding id and then proves the first file intact. Accepted as source-inspected.
3. **F-3 — carried to AT-08, blocking there.** No per-ID reconciliation of the whole corpus. The reachable subsets are
   exercised (names, kinds, and the scan-relevant policy/stability/exit rows); the rest have no row at all, which
   [EVIDENCE-PROTOCOL](../EVIDENCE-PROTOCOL.md) §3 calls a finding. It is also unsatisfiable at this stage: the
   `would_optimize`/`optimized`/`failed` rows, exit `3`, exit `130` and the mode rows describe an apply and a runner that do
   not exist yet. Obligation: the per-ID accounting table ships with the first handoff that includes apply.
4. **F-4 — carried to AT-08, blocking there.** No checksummed artifact bundle (`SHA256SUMS` plus an internal `head_sha`).
   The evidence is SHA-bound in the weaker sense — annotations of run `42416157628`, whose `head_sha` is the claimed SHA,
   carrying the raw plan documents, listings and file hashes in base64.
5. **F-5 — fixed.** The PR body and `EVIDENCE-PROTOCOL.md` quoted "111 IDs"; `FIXTURE-CORPUS.md` carries **120** contiguous
   IDs. The page is now the authority and 111 the floor.
6. **F-6 — not run.** `doctor`, `run --once`, apply, `update-check`/W-02 and exit codes `3`/`4`/`5`/`130` are outside the
   stage and have no evidence. `not-run` is not `passed`, and nothing in this review upgrades them.
7. **F-7 — Low, declared.** macOS skips N-17 (`errno 92`) and does not run the `/proc` boundary test. The corpus rule
   requires the skip to be reported with platform and reason, which it is (`non_utf8_skipped=1`); the test tolerates only
   OS error 92 on that one name and panics otherwise. Those two rows are proven on Linux only.

## 5. Acceptance points, point by point

All 21 rows of the acceptance table in [HANDOFF-AT-03](../HANDOFF-AT-03.md) have evidence. The mapping from each row to its
test or CI artifact is the coder's table in the `#46` body, which I checked against the code and the run: rows 1–2, 5, 6,
10–13 and 21 are recomputed or executed above; rows 3, 7–9, 14–20 are source-inspected against named tests that exist and
assert the stated thing; row 19 additionally carries F-2.

## 6. Decision

No Critical finding and no High finding against what AT-03 promises (charter §9): nothing writes into a target root, no
process is spawned from a hostile name, no evidence comes from a foreign SHA, and no refusal returns `0`. The stage is
accepted. The two carried items are obligations of the MVP gate, where they are blocking, and they are named here so that no
later green run is mistaken for them.
