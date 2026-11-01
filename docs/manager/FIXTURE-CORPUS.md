# Pre-registered fixture corpus

> **Upstream contract moved (2026-11-01).** `tzim` v0.16.0 shipped `--capabilities`, a stable `reason_code` and
> JSON result schema 2; v0.16.1 is the current release and its exit-code precedence is now documented and
> enforced. Statements below written against `tzim` 0.14 that assume no capability document, an untyped
> refusal or an older pin are stale: see the [2026-11-01 audit](../audits/2026-11-01-tzim-v0.16.1.md).

Frozen **2026-10-29**, before any implementation exists. Charter §7.3 forbids removing an unfavourable fixture or
changing an expected result after the outcome is known; this page exists so that both are decided now, by the reviewer,
not later by the author.

Rules that make the corpus binding:

1. **IDs are stable.** A case is referred to by its ID in the handoff (`N-03`, `S-02`, …) and in the raw evidence rows.
2. **Every case is built by code, not hand-made**, so it is reproducible on both platforms; the two committed binary
   fixtures (a real PNG and a real gzip) carry their SHA-256 in the test that uses them.
3. **No case may be dropped.** A case that cannot run on a platform is reported as `skipped` **with the reason and the
   platform**, and that row is a finding for the reviewer, not a pass.
4. **Expected outcomes below are the specification's, not a prediction of what the code will do.** If an implementation
   disagrees, either the implementation or this page is wrong, and the disagreement is resolved in a commit *before* the
   result is accepted.
5. Additions are allowed (a new case may be appended with a new ID at any time); removals and expectation changes are
   not.

Target: ~40 cases, each mapping to a gate row in [GATES.md](GATES.md).

---

## 1. Names — nothing is ever interpreted (`N-*`, INV-4, AC-1, AC-4)

All cases live in one directory inside a temporary root. Each file contains one byte; none is a supported format, so the
expected outcome is either a policy/unsupported skip or, for the last case, an *attempted* validation that must not
create anything.

| ID | Name (as constructed) | Expected |
| --- | --- | --- |
| N-01 | `plain.png` | planned; `unsupported` or refusal after apply, bytes unchanged |
| N-02 | `with space.png` | same as N-01 |
| N-03 | `two  spaces .png` | same |
| N-04 | `semi;colon.png` | same |
| N-05 | `dollar$(touch /tmp/ats_pwned).png` | same, **and `/tmp/ats_pwned` must not exist afterwards** |
| N-06 | `` back`touch /tmp/ats_pwned2`.png `` | same, and `/tmp/ats_pwned2` must not exist |
| N-07 | `quote'single.png` | same |
| N-08 | `quote"double.png` | same |
| N-09 | `newline\nname.png` | same; the JSON `path` escapes it, `path_raw_b64` optional |
| N-10 | `tab\tname.png` | same |
| N-11 | `-leading-dash.png` | same; argv must carry `--` before it |
| N-12 | `--json` | same; must not be parsed as a flag |
| N-13 | `--` | same |
| N-14 | `-` | same; **not** stdin |
| N-15 | `emoji-🙂.png`, `кириллица.png`, `日本語.png` | same |
| N-16 | `combining-e\u0301.png` | same |
| N-17 | non-UTF-8 bytes on Unix (`b"bad\xff.png"`), skipped on the platforms that cannot create it | same; `path` lossy, `path_raw_b64` present |

## 2. Kinds — what discovery must refuse (`K-*`, INV-5, INV-6, AC-4)

| ID | Object | Expected status | Exit |
| --- | --- | --- | --- |
| K-01 | symlink → regular file **inside** the root | `skipped_policy` (no-follow), target unchanged | `0` |
| K-02 | symlink → file **outside** the root | `skipped_policy`, target unchanged | `0` |
| K-03 | symlink → directory inside the root | `skipped_policy`, its contents not planned | `0` |
| K-04 | broken symlink | `skipped_policy` | `0` |
| K-05 | symlink loop (`a → b → a`) | `skipped_policy`, no hang, completes < 5 s | `0` |
| K-06 | FIFO | `skipped_policy`, no read attempted (no hang) | `0` |
| K-07 | Unix socket | `skipped_policy` | `0` |
| K-08 | device node (skipped when the platform forbids creating one) | `skipped_policy` or case reported `skipped` + reason | `0` |
| K-09 | hard-linked pair, 2 links | both `hardlink_skip`, both unchanged | `0` |
| K-10 | hidden file `.hidden.png`, `include_hidden = false` | not planned at all (absent from the plan) | — |
| K-11 | hidden file, `include_hidden = true` | planned | — |
| K-12 | orphan `.tzim-123-456-0.tmp`, `include_hidden = true` | **never planned**, diagnostic emitted | `0` |
| K-13 | file with `allow_hard_links = false` (same as K-09, asserted via plan) | one `hardlink_skip` row per link | `0` |
| K-14 | 65-level-deep directory | refused at the depth limit, no stack overflow, reason recorded | `0` |

## 3. Stability — races end as skips, never as success (`S-*`, INV-6, AC-4, R-03)

| ID | Injected situation | Expected |
| --- | --- | --- |
| S-01 | file modified between discovery and the pre-spawn check | `skipped_unstable`, bytes at the end are the *later* bytes, no `optimized` |
| S-02 | file deleted between discovery and the pre-spawn check | `skipped_unstable`, no exception traceback, run continues |
| S-03 | file younger than `min_file_age_seconds` at scan | `skipped_unstable`; after the clock advances (injected), it is planned |
| S-04 | file touched between `scan` and `run --once` | **exit `6`** (plan hash mismatch), nothing processed |
| S-05 | file touched during the apply of that file (fake `tzim` mutates, reports success) | `failed` (`internal`, identity changed), never `optimized` |
| S-06 | path swapped for a symlink to `/etc/hostname` between plan and spawn | abort: `skipped_policy` + diagnostic; `/etc/hostname` byte-identical |

## 4. Policy — our decisions, made before any process (`P-*`, INV-9, INV-11, AC-3)

| ID | Config | Expected |
| --- | --- | --- |
| P-01 | `formats = []` | argv contains **no** `--formats` |
| P-02 | `formats = ["png"]` | argv contains `--formats png`; a `.gz` in the root is `skipped_policy` |
| P-03 | `transforms = ["fits-tile"]` | config rejected (`transforms_forbidden`), exit `2`, nothing planned |
| P-04 | `follow_symlinks = true` | config rejected (`not_in_v01`), exit `2` |
| P-05 | `allow_hard_links = true` | config rejected (`not_in_v01`), exit `2` |
| P-06 | `jobs = 2` | config rejected (`not_in_v01`), exit `2` |
| P-07 | `[folder.schedule]` present | config rejected, exit `2` |
| P-08 | `exclude_globs = [".git/**"]` with a matching file | file absent from the plan |
| P-09 | root is `$HOME` | rejected `forbidden_root`, exit `2` |
| P-10 | root contains `AUTO_TZIM_HOME` | rejected `state_inside_root`, exit `2` |

## 5. Adapter — fail-closed, no prose (`A-*`, INV-18, AC-13)

| ID | Fake binary behaviour | Expected |
| --- | --- | --- |
| A-01 | `--version` → `tzim <PINNED_VERSION>` (shape `tzim <semver>`; the literal comes from the pin file, never from this table) | accepted |
| A-02 | `--version` → `tzim 0.15.0` | `incompatible_tzim`, exit `3`, nothing planned |
| A-03 | `--version` → `tzim 1.0.0` | `incompatible_tzim`, exit `3` |
| A-04 | `--version` → `garbage` | `incompatible_tzim`, exit `3` |
| A-05 | binary missing / not executable | exit `3` |
| A-06 | binary exits non-zero for `--version` | exit `3` |
| A-07 | apply: JSON without `schema` | `failed` (`internal`), abort, exit `1` |
| A-08 | apply: unknown `status` value | `failed` (`internal`), abort, exit `1` |
| A-09 | apply: extra optional field `"strip_metadata":true` | ignored, run succeeds, field recorded as ignored |
| A-10 | apply: `--info`/`--list-formats` **tripwire** — the fake exits `99` if invoked with either | never invoked in any test. `--capabilities` is *not* a tripwire: since D-26 it is the supported way to ask, and `A-11` covers it |
| A-11 | `--capabilities` at start: (a) a valid document, (b) `capabilities_schema_version: 2`, (c) `result_schema_version: 3`, (d) the flag rejected as unknown, (e) the document missing a required key | (a) planning proceeds and both schema versions are recorded; (b)–(e) `incompatible_tzim`, exit `3`, nothing planned, nothing staged |
| A-12 | `--json` row with `reason_code: "format_disabled"`, one with `reason_code: "something_new"`, one with `reason_code: null` | all three accepted: the first two are stored verbatim in `reason`, the third stays `null`; none is mapped to a code of ours and none changes a status (D-27) |

## 6. Runner — process discipline (`R-*`, INV-12, INV-13, AC-5, AC-6)

| ID | Fake binary behaviour | Expected |
| --- | --- | --- |
| R-01 | prompts on stdin and blocks (the `tzim` FITS confirmation shape) while our child stdin is `/dev/null` | killed by the per-file timeout, `failed` (`timeout`), total run time bounded, nothing left running |
| R-02 | asserts `is_terminal()` on all three streams and exits `98` if any is a TTY | never exits `98` |
| R-03 | prints `argv` and the environment | argv is a list, no shell metacharacter interpretation, env is the minimal documented set |
| R-04 | ignores `SIGTERM` | killed by `SIGKILL` after the grace period, `failed` |
| R-05 | exits `0`, prints truncated JSON | `failed` (`internal`), never `optimized` |
| R-06 | exits `0`, prints two summaries | `failed` (`internal`) |
| R-07 | exits `0`, prints no summary | `failed` (`internal`) |
| R-08 | exits `0`, prints 50 MiB of stdout | output cap enforced, `failed` with a reason, memory bounded |
| R-09 | exits `0`, prints 10 MiB of stderr | stderr stored truncated with a marker, not in stdout |
| R-10 | exits `4` (verification failure) on the third of five files | `failed` (`verify`) on that file, remaining two `interrupted`, plan aborted, **our exit `1`, never `4`** |
| R-11 | exits `2` on the second file | `failed` (`internal`), run aborted, exit `1` |
| R-12 | exits `1` on the second file | `failed`, run **continues**, exit `1` |
| R-13 | dies by signal (`SIGSEGV`) | `failed` (`internal`), exit `1`, no crash of our process |
| R-14 | never invoked: second run over the unchanged corpus | zero invocations, identical statuses, exit `0` |
| R-15 | a `failed` file, unchanged, second run | no invocation (invariant 13) |

## 7. State — locks and recovery (`L-*`, AC-7, INV-19)

| ID | Injected situation | Expected |
| --- | --- | --- |
| L-01 | second `run --once` while the first holds the lock | exit **`8`**, nothing processed, no second plan |
| L-02 | stale lock (pid gone) | reported with pid and start time; re-run after confirming the process is gone succeeds |
| L-03 | `SIGKILL` of our process mid-run, then a new start | previous run `interrupted`, no row `running`, no row `optimized` |
| L-04 | malformed line in the middle of a `files.jsonl` | exit **`7`**, the file and line named, nothing rewritten |
| L-05 | corrupt config | exit `2`, config byte-identical afterwards |
| L-06 | unwritable state directory | exit `7` with a reason, no partial file |

## 8. CLI — the exit-code contract (`X-*`, AC-8, AC-11, AC-12)

| ID | Command | Expected |
| --- | --- | --- |
| X-01 | `doctor` with a good installation | `0` |
| X-02 | unknown flag | `2` |
| X-03 | `history --status nonsense` | `2` |
| X-04 | `run --once` with **no plan** | `6`, nothing processed |
| X-05 | `run --once` with `pause` active | `6` |
| X-06 | `run --once` with `min_free_bytes` above free space | `6` |
| X-07 | `run --once` on a disabled folder | `6` |
| X-08 | `run --once --plan <fabricated id>` | `6` |
| X-09 | `run --once` with a stale plan hash (S-04) | `6` |
| X-10 | `run --once` with a valid plan | `0` |
| X-11 | usage error **while a lock is held** | `2` (precedence: `2` > `8`) |
| X-12 | incompatible binary **and** paused | `3` (precedence: `3` > `6`) |
| X-13 | corrupt state **and** bad flag | `7` — the deliberate tie-breaker case: precedence says `7 > 2`, and the implementation must state that order in `--help` and in `cli.md` rather than leave it to chance |
| X-14 | `SIGINT` during a run with one earlier `failed` file | `130`, JSON still contains the `failed` row |
| X-15 | `scan` output parsed by a strict JSON validator, with a warning on stderr | every line parses, warning only on stderr |
| X-16 | `4` produced by any command | **impossible** — asserted by grepping the whole matrix for `4` |
| X-17 | stdout closed early (`EPIPE`) during a run that has already earned `1` | exit `1`, not `3` and not `0`: a write error never replaces a code the run already earned (DR-07) |

## 9. Network and documentation (`W-*`, `D-*`, AC-9, AC-10)

| ID | Case | Expected |
| --- | --- | --- |
| W-01 | every command, both modes, run with no network namespace and with the DNS resolver disabled | all succeed, zero connection attempts |
| W-02 | `update-check` invoked as a command | exit `2` (unknown command in v0.1, D-28); absent from `--help` |
| W-03 | a planted `ureq`/`reqwest` line in `Cargo.toml` | `cargo deny` **fails** and `cargo tree` reports it — the control proves the check is live, not decorative |
| W-04 | the release binary run in a container with no network and no DNS | identical behaviour, identical exit codes |
| W-05 | binary path hash before/after every W-case | identical |
| D-01 | every documented default vs the schema | equal (reflection) |
| D-02 | every documented exit code vs the X-matrix | present and equal |
| D-03 | "undo" in user-facing text | only disclaimers |
| D-04 | any document mentioning `tzim --dry-run` as our behaviour | none |

## 10. Modes and install (`M-*`, D-29, AC-14…AC-17, INV-20…INV-23)

The point of this block is that **the mode is the only difference**. Every case runs twice where it makes sense, and the
two runs must agree on everything except the approval row and whether a proposal appeared.

| ID | Case | Expected |
| --- | --- | --- |
| M-01 | `confirm`, TTY, answer `y` | proposal on stderr, installs happen, originals replaced, exit `0` |
| M-02 | `confirm`, TTY, answer `n` / empty / `no` / `NO` / `yep` | exit `0`, **zero** writes in the folder, all rows `skipped_by_user`, history row `declined` |
| M-03 | `confirm`, stdin is `/dev/null` | exit `6`, nothing processed, message names `--mode auto`; measured with a timeout so a hang is red |
| M-04 | `confirm` with `--json` | exit `6`, nothing processed (a dataset cannot be asked a question) |
| M-05 | `auto` on the same tree as M-01 | same per-file outcomes and sizes, no proposal, exit `0` |
| M-06 | `confirm` then EOF on stdin mid-prompt | exit `0`, nothing installed, nothing half-installed |
| M-07 | `SIGKILL` between `rename(original → old)` and `rename(staged → original)` | next start restores the original from `.auto-tzim-old-*`; bytes identical; recovery line names the file |
| M-08 | `SIGKILL` after `rename(staged → original)`, before `unlink(old)` | next start removes the leftover and reports `installed`, not `restored` |
| M-09 | file edited between plan and install (content only) | `skipped_changed`, original keeps the **edited** bytes, staged copy deleted |
| M-10 | file edited between plan and install (`mtime_ns` only) | `skipped_changed` — identity includes time, not just size |
| M-11 | `tzim` returns `already_optimal` / `skipped` / `unsupported` / `failed` in `auto` | no install for that file, original byte-identical, row carries the `reason` code |
| M-12 | free space below staged size + headroom | exit `6` before the first file, nothing staged, nothing installed |
| M-13 | staging directory deleted between files by an external actor | that file `failed` with `kind=io`; the rest continue; originals untouched |
| M-14 | proposal text | contains the sentence about deletion and no undo; contains no path outside the folder |

---

## How the reviewer uses this page

1. The handoff must reference every ID above, with a raw row (or an explicit `not run: <reason>`).
2. The reviewer picks **five cases at random** and re-runs them independently; a mismatch is a finding.
3. A case reported `skipped` on a platform without a platform-specific reason is a finding.
4. The `N-05`, `N-06`, `R-10`, `S-06` and `X-16` rows are the ones a "passing" implementation most often fakes; they are
   checked first, against raw bytes, not against the summary.
