# Risk register

The charter's baseline risks (R-01…R-12) are kept verbatim in meaning; R-13…R-21 come from the `tzim` study and
from what this project's own shape makes likely. A risk is closed only by an implemented control **and** a
behavioural test that would fail if the control were removed. "Unlikely" is not a control.

Status values: `open` (nothing implemented yet), `specified` (control described, not built), `controlled`
(built and evidenced — nothing is in this state today).

| ID | Risk | Level | Control | Verification | Status |
| --- | --- | --- | --- | --- | --- |
| R-01 | A root is too wide or wrong | Critical | Narrow explicit allowlist, forbidden-root list, first run dry-run only, canonical containment re-check before every spawn | Tests for `/`, `$HOME`, system dirs, `..` traversal, and a root that is a symlink; a plan row outside the root is a hard abort | specified |
| R-02 | Symlink / path traversal | Critical | `lstat`-based walk, no follow, canonical containment, symlinks skipped with a reason | Unit + integration tests with symlinked files, symlinked dirs, links out of the root, link loops | specified |
| R-03 | A file changes while it is processed | Critical | `min_file_age_seconds`, identity re-check between discovery and spawn, post-run identity re-read | Injected `stat` transitions at each stage; a modified file must end as `skipped_unstable`/`failed`, never `optimized` | specified |
| R-04 | Hard-link siblings | High/Critical | `st_nlink > 1` → `hardlink_skip` before any spawn | Fixture with two links; both files' bytes unchanged; no `tzim` process started (asserted via the fake runner) | specified |
| R-05 | Re-processing / retry storm | High | Invalidation key (D-06) for successful and refused outcomes; `failed` never retried automatically | Two consecutive runs: second run produces zero `tzim` invocations; a `failed` row stays failed without a manual retry | specified |
| R-06 | Incompatible `tzim` JSON/CLI | High | Version-keyed adapter (D-01), schema guard, fail-closed `incompatible_tzim` (exit `3`) | Fake `tzim` variants: wrong version, missing `schema`, unknown status, malformed JSON, extra fields | specified |
| R-07 | A prompt in a scheduler / non-TTY run | High | `stdin=DEVNULL`, no TTY assumption, timeout, and a pinned `tzim` range with no prompts (D-01, study G-5) | Fake `tzim` that reads stdin and blocks: run must terminate by timeout with a diagnostic, not hang | specified |
| R-08 | Silent or unsafe update | Critical | MVP is notify-only; single documented request; no download, no replacement (D-07) | Test that `run`/`scan` never open a socket (asserted by a blocked resolver in the test env); `update-check` mutation test: no file in `AUTO_TZIM_HOME` except the cache/state | specified |
| R-09 | Cloud-sync conflict | High | `allow_cloud_sync = false` default; known sync roots rejected unless explicitly allowed | Validation tests for the documented patterns; documentation states the check is best-effort | specified |
| R-10 | Path leakage through logs/telemetry | High | Local-only logs, retention (30 days / size cap), paths treated as sensitive, no telemetry | `--json`/log inspection test: no environment dump, no absolute path outside the root, no network fields | specified |
| R-11 | UI duplicates core logic | Medium | UI is a thin layer over one core API; architecture review at each stage | v0.2/v0.4 review gate: no discovery, subprocess or updater code outside `core/` | specified |
| R-12 | Supply-chain / dependency risk | High | Minimal pinned dependencies with hashes; `lockfile` CI job; no post-install scripts | Review of the pinned set (proposal D-15) and a CI job that fails on an unpinned dependency | specified |
| R-13 | "Dry run changes nothing" is stated too broadly | High | Document that `tzim -n` creates and removes a temporary candidate in the target directory (study B-1); invariant 2 phrased about bytes and permanent files | Documentation review + a test that the target directory contains no extra entry after a dry run | specified |
| R-14 | Orphaned `.tzim-*.tmp` files enter discovery after a crash | High | Pattern excluded from discovery even with `include_hidden = true`; reported as a diagnostic; never deleted by us | Fixture with orphan temp files, including one named like an image | specified |
| R-15 | The adapter silently drifts from `tzim` behaviour | High | Pinned artifact by SHA-256 (D-13), T1–T10 behavioural assertions at CI time, spec-version recorded per run | CI job comparing observed JSON/exit codes against the pre-registered table; a mismatch fails the job instead of editing the expectation | specified |
| R-16 | `update-check` breaks or leaks under rate limiting | Medium | Timeout, cache, explicit error mapping to exit `5`, documentation of the single request | Tests for 403/429, timeout, malformed JSON, and an offline environment | specified |
| R-17 | Stability tests are slow or flaky | Medium | Injected clock (D-14); one integration test with a real 1–2 s age and a documented marker | Test-suite duration budget in CI; a rerun that shows the same result twice | specified |
| R-18 | State directory inside a managed root | Critical | Root validation rejects a root containing `AUTO_TZIM_HOME` (D-08); discovery excludes it too | Unit test for the rejection; integration test that the SQLite file is never planned | specified |
| R-19 | `tzim` exit `4` (verification failure) is mishandled | Critical | Abort semantics in D-02: `failed(verify)`, remaining plan `interrupted`, critical diagnostic, no retry | Fake `tzim` returning `4` mid-plan; the run must stop, not continue, and must not retry | specified |
| R-20 | macOS-only failures stay invisible (no real macOS evidence) | High | CI on macOS from AT-01 onwards; acceptance is per-platform and platform-named in every report | At least one macOS job per stage, with logs archived at the head SHA | specified |
| R-21 | Specification and implementation diverge | Medium | Provenance file pins the spec version; GATES matrix references specification clauses; any divergence becomes a decision, not a silent edit | Review at each stage: every documented behaviour traceable to a clause or a decision ID | specified |

## Rules the manager applies to this register

1. A Critical/High row is never marked `controlled` on the strength of a green CI badge; it needs the named
   behavioural test and the raw artifact.
2. A row can be **accepted** by the owner (documented residual risk) — it is never quietly dropped.
3. New rows are added by the manager when a review finds a failure mode with no row; a review finding without a
   risk row is incomplete.
4. R-19, R-18, R-14, R-13 and R-15 did not exist in the charter's baseline: they come from reading `tzim` and
   from the shape of this project. They are the reason the study was done before the code.
