# MVP acceptance matrix (pre-registered)

Written **before** any implementation exists, so that the criteria cannot be shaped by the first results. Every row
names the specification clause, the behavioural evidence required, and the negative control that proves the
evidence can fail. A row passes only with raw artifacts at the reviewed head SHA.

Pre-registration rules (charter §7.3):

1. This file is frozen at the first implementation commit. Changing a row needs a written reason, a new commit
   *before* the result is seen, and the owner's notification — never an edit after a failing result.
2. A row is never satisfied by a green badge, by a screenshot, or by "it works locally".
3. Fixtures are fixed before the run. A fixture that behaves unfavourably is not removed; if it was wrong, that is
   a documented correction with a pre-registered reason.
4. Anything the coder could not run is reported as not-run, not as passed.

---

## 1. Specification acceptance criteria (§15)

| ID | Clause | Required behavioural evidence | Negative control | Verified by |
| --- | --- | --- | --- | --- |
| AC-1 | Narrow folder + deterministic dry-run plan | `scan --json` twice on the hostile-name corpus; the two plan bodies are byte-identical; every path is inside the canonical root | A file created between the two runs must appear in the second plan and be absent from the first — proving the comparison is sensitive | manager, independently |
| AC-2 | First run without dry run changes nothing | Pre/post recursive checksums and inode list of the root after `run --once` on an unpreviewed folder; the command refuses per D-03 | Same command with the gate removed in a test double must process the folder — proving the gate is what stopped it | manager, independently |
| AC-3 | `run --once` calls official `tzim` only with computed allowed parameters | Golden argv per profile + the real-binary transcript; the binary path is the configured one, its SHA-256 recorded | A fake `tzim` that records argv must reject any token not in the allowlist, and a test proves the check fires on a tampered argv | manager, tool-executed |
| AC-4 | No unknown/unsupported/refused/failed/unstable/hard-linked/symlink file is modified | Per-case before/after SHA-256 for every non-optimizing outcome; asserted equal, including `mtime` | A test that flips one expectation must fail the suite (mutation of the expectation, not of the code) | manager, independently |
| AC-5 | No shell, no prompts, no root, no hidden background activity | Code inspection of every spawn site; `stdin=DEVNULL`; a fake `tzim` that blocks on stdin must be killed by the timeout; a process-table check that no daemon survives the command | The blocked-stdin fixture is the control: a hang would fail the job by timeout | manager, both |
| AC-6 | Two runs do not create a retry loop | Second run on an unchanged corpus produces zero `tzim` invocations (asserted by the fake runner's invocation log) and identical statuses | Changing one byte of a previously-`failed` file must make it eligible again — proving the invalidation key is live | manager, tool-executed |
| AC-7 | Config/history/log recovery after cancellation and crash | Injected `SIGKILL` mid-run: next start reports the run `interrupted`, the DB is readable, no row stays `running` | A crash injected after the commit point must leave the run `completed`, not `interrupted` | manager, independently |
| AC-8 | JSON and exit codes match documentation | Table of every command × every exit code, each produced by a real invocation; `--json` stdout parsed by a strict validator in the same test | One row deliberately breaks an expectation to show the table is executed, not transcribed | manager, tool-executed |
| AC-9 | Update check only notifies | Resolver disabled for all commands except `update-check`; mocked responses for 200/403/429/500/malformed; nothing in `AUTO_TZIM_HOME` is replaced | An "up to date" answer from an error response must fail the suite | manager, tool-executed |
| AC-10 | Documentation matches behaviour | Reflection test: every documented default equals the schema default; every documented exit code appears in the test table; limitations list checked line by line against the binary | A documented default that the schema contradicts must fail the test | manager, source-inspected + tool-executed |

## 2. Safety invariants (charter §6)

| ID | Invariant | Evidence | Control that proves the evidence can fail |
| --- | --- | --- | --- |
| INV-1 | Nothing is processed before the user adds the folder | Empty config → `scan`/`run` find nothing and start no process | Config written only in the test's arrangement step; a run without it must not process | 
| INV-2 | First run of a new folder is dry-run only | Refusal row (D-03) + zero byte changes | See AC-2 |
| INV-3 | No root/sudo requirement, no hidden background process | Process table before/after; no `setuid`; no unit files installed in v0.1 | A test asserting the absence of any child process after exit |
| INV-4 | No shell; filenames stay argv data | Every spawn site inspected; hostile-name corpus (spaces, newline, `;`, `$(...)`, backtick, quotes, leading `-`, non-UTF-8) processed without interpretation | Filename `$(touch /tmp/pwned)` — the file must not appear |
| INV-5 | No symlink following, no special files | Symlink/socket/FIFO/device corpus: skipped with a reason, targets untouched | Symlink pointing *inside* the root must also be skipped, not followed |
| INV-6 | Hard-linked and unstable files skipped by default | `st_nlink > 1` rows; identity-change rows | A hard-linked pair where one is modified during the run must end `skipped_unstable` for both attempts, never `optimized` |
| INV-7 | Never outside the canonical root | Containment re-check before every spawn; a path swapped after planning must abort | A test that replaces a planned file with a symlink to `/etc/hostname` before the spawn must abort, not follow |
| INV-8 | No own codec, no own installer | Source review: no image/archive code, no `os.replace` of a candidate | `tzim` absent → exit `3`, nothing written |
| INV-9 | Eligibility/refusal/verification/install stay in `tzim` | Adapter table maps, never re-decides; a `tzim` refusal is recorded verbatim | A file `tzim` refuses but which our policy would allow must remain `skipped_refused` |
| INV-10 | No re-routing of unknown/unsupported/refused/failed | One handler, one path; no fallback branch exists | Code review + a test asserting only one spawn per file per run |
| INV-11 | Confirmation-required transformations need explicit opt-in | `transforms = []` allowlist; any value rejected (D-05) | A config with `fits-tile` must be rejected before any file is planned |
| INV-12 | Scheduler/non-TTY/JSON never wait for a prompt | No prompt code path exists; `stdin=DEVNULL`; timeout | Blocked-stdin fake (AC-5) |
| INV-13 | `failed` never retried automatically | Second run makes no invocation for a failed file | Manual retry succeeds after the file changes — proves it is a policy, not a permanent blacklist |
| INV-14 | MVP update check downloads and replaces nothing | No download code; mocked responses; filesystem diff | A mocked "new version" response must leave the binary path untouched and unchanged by hash |
| INV-15 | No telemetry, no undocumented requests | Resolver disabled outside `update-check`; log fields reviewed | Any socket attempt outside the allowed command fails the test |
| INV-16 | No false undo promise | Documentation review; no backup code; explicit statement in README and `--help` | Text search for "undo"/"restore" in user-facing output must find only the disclaimers |
| INV-17 | UI does not duplicate core logic | Not testable in v0.1 (no UI); recorded as a v0.2/v0.4 gate | Architecture review at that stage |

## 3. Evidence a handoff must contain

Per charter §7.1: final head SHA, issue/commit map, CI run IDs with their SHA, raw per-case and per-file rows,
independent recomputation of aggregates, artifact SHA-256 values, OS/arch/Python/`tzim` identity, the reproduction
commands, docs updated, and `remaining blocker: none` or an exact blocker in the charter §10 format.

The manager then marks each claim **independently verified / source-inspected / tool-executed / coder-reported** and
records the marks in the review document. A claim that stays `coder-reported` is a finding, not a pass.

## 4. Stop rules

- Any Critical finding (charter §9) stops the review and the merge recommendation, regardless of the remaining rows.
- A High finding stops acceptance; it may be listed as a follow-up only by the owner, never by the manager.
- A required job that is skipped, cancelled, or replaced by a placeholder is a **failed** row.
- Evidence from any SHA other than the final head is invalid for the final decision.
- A change after the review invalidates every row it could affect; the affected rows are re-run, not argued about.
