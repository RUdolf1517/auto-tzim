# Pre-registered fixture corpus

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
| A-01 | `--version` → `tzim 0.14.0` | accepted |
| A-02 | `--version` → `tzim 0.15.0` | `incompatible_tzim`, exit `3`, nothing planned |
| A-03 | `--version` → `tzim 1.0.0` | `incompatible_tzim`, exit `3` |
| A-04 | `--version` → `garbage` | `incompatible_tzim`, exit `3` |
| A-05 | binary missing / not executable | exit `3` |
| A-06 | binary exits non-zero for `--version` | exit `3` |
| A-07 | apply: JSON without `schema` | `failed` (`internal`), abort, exit `1` |
| A-08 | apply: unknown `status` value | `failed` (`internal`), abort, exit `1` |
| A-09 | apply: extra optional field `"strip_metadata":true` | ignored, run succeeds, field recorded as ignored |
| A-10 | apply: `--info`/`--list-formats` **tripwire** — the fake exits `99` if invoked with either | never invoked in any test |

## 6. Runner — process discipline (`R-*`, INV-12, INV-13, AC-5, AC-6)

| ID | Fake binary behaviour | Expected |
| --- | --- | --- |
| R-01 | reads stdin and blocks | killed by the timeout, `failed` (`timeout`), total run time bounded |
| R-02 | asserts `isatty()` on all three streams and exits `98` if any is a TTY | never exits `98` |
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
| L-04 | corrupt SQLite file | exit **`7`**, no traceback dump, database left untouched |
| L-05 | corrupt config | exit `2`, config byte-identical afterwards |
| L-06 | unwritable state directory | exit `7` with a reason, no partial database |

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
| X-13 | corrupt state **and** bad flag | `7`? — **deliberate tie-breaker case**: expected `7` per precedence `7 > 2`; the reviewer checks the implementation documents this order explicitly |
| X-14 | `SIGINT` during a run with one earlier `failed` file | `130`, JSON still contains the `failed` row |
| X-15 | `scan` output parsed by a strict JSON validator, with a warning on stderr | every line parses, warning only on stderr |
| X-16 | `4` produced by any command | **impossible** — asserted by grepping the whole matrix for `4` |

## 9. Network and documentation (`W-*`, `D-*`, AC-9, AC-10)

| ID | Case | Expected |
| --- | --- | --- |
| W-01 | resolver disabled; `scan`, `run`, `status`, `history`, `logs`, `folder`, `doctor`, `config validate` | all succeed, zero socket attempts |
| W-02 | `update-check` with mocked 200 (newer) | `0`, "update available", nothing written outside cache |
| W-03 | mocked 403 / 429 / 500 / malformed / HTML / non-semver | `5` each, never "up to date" |
| W-04 | mocked 200 with a newer **major** | "incompatible", exit `0`, nothing acted on |
| W-05 | binary path hash before/after every W-case | identical |
| D-01 | every documented default vs the schema | equal (reflection) |
| D-02 | every documented exit code vs the X-matrix | present and equal |
| D-03 | "undo" in user-facing text | only disclaimers |
| D-04 | any document mentioning `tzim --dry-run` as our behaviour | none |

---

## How the reviewer uses this page

1. The handoff must reference every ID above, with a raw row (or an explicit `not run: <reason>`).
2. The reviewer picks **five cases at random** and re-runs them independently; a mismatch is a finding.
3. A case reported `skipped` on a platform without a platform-specific reason is a finding.
4. The `N-05`, `N-06`, `R-10`, `S-06` and `X-16` rows are the ones a "passing" implementation most often fakes; they are
   checked first, against raw bytes, not against the summary.
