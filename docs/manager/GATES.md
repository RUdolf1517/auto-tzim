# MVP acceptance matrix (pre-registered)

> **Upstream contract moved (2026-11-01).** `tzim` v0.16.0 shipped `--capabilities`, a stable `reason_code` and
> JSON result schema 2; v0.16.1 is the current release and its exit-code precedence is now documented and
> enforced. Statements below written against `tzim` 0.14 that assume no capability document, an untyped
> refusal or an older pin are stale: see the [2026-11-01 audit](../audits/2026-11-01-tzim-v0.16.1.md).

Written **before** any implementation exists, so that the criteria cannot be shaped by the first results. Every row
names the clause it enforces, the behavioural evidence required, and the negative control that proves the evidence can
fail. Revised on 2026-10-29 for charter v0.2 (§6 invariants 18–19, §8.4, §16): the read-only candidate plan, the
generic `skipped_by_tzim`, the exit codes `6/7/8` and the precedence are now first-class acceptance items.

**Revised 2026-11-02 (D-46).** The one-shot form `auto-tzim PATH [--time HOURS] [--auto]` is the normal invocation, so
AC-11's first case reads "the plan no longer matches" rather than "no plan exists": every run builds its plan, and the
two-step `scan`-then-`run` choreography is no longer required. Semantics unchanged; only the spelling of the invocation.

**Revised again on 2026-11-01 (D-26…D-29).** AC-14…AC-17 and INV-20…INV-23 are new and were written *before* any of this
code exists: one install path for two modes, a crash-safe swap, an identity re-check per file, and a confirmation gate that
fails closed. AC-9 changed meaning — from "the one request is honest" to "there is no request" — because `update-check`
left the MVP (D-28). Rows are never deleted here: a superseded row keeps its number and says what superseded it, so a later
reader cannot mistake a weaker past criterion for a stronger present one.

Pre-registration rules:

1. Each row changes only by a commit written **before** the result that motivated it is seen, with the reason in the
   commit message. An expectation edited after a failing run is a finding, not a fix.
2. A row is never satisfied by a green badge, a screenshot, or "it works locally".
3. Fixtures are fixed in advance and listed by ID in [FIXTURE-CORPUS.md](FIXTURE-CORPUS.md) (frozen 2026-10-29, before
   any code). An unfavourable fixture is not removed, and an expected outcome is not edited after a result is known.
4. Anything not run is reported as not-run, never as passed.

---

## 1. Specification acceptance criteria

| ID | Clause | Required behavioural evidence | Negative control | Verified by |
| --- | --- | --- | --- | --- |
| AC-1 | Narrow folder, deterministic candidate plan (`SPEC` §15.1, D-04/D-23) | `scan --json` twice on the hostile-name corpus: the two plan bodies are byte-identical, every path is inside the canonical root, every row's status is `planned` | A file created between the two runs appears in the second plan and is absent from the first | manager, independently |
| AC-2 | Scan/dry-run is read-only (charter §6-2, §8.4, D-04) | **Invocation counter**, counted in two buckets that are never mixed: **flag-only probes** (`--version`, `--capabilities` — no path, no file read) and **path invocations** (a file or directory argument). A scan/dry-run records exactly two probes, `--version` then `--capabilities` (D-26), and **zero** path invocations; the target root's recursive listing, inode list and every file's SHA-256 are identical before and after; the run succeeds with the target root mounted read-only (or made non-writable) | The same corpus under `run --once` (apply) must change at least one file and must show path invocations naming **staged copies** — proving the counter can observe invocations at all; a path invocation naming a file inside the root fails INV-20 | manager, independently |
| AC-3 | Only computed, allowed argv (`SPEC` §15.3) | Golden argv per profile; the real-binary transcript; the binary path is the configured one and its SHA-256 is recorded; no `--dry-run`, no `--formats` when `formats = []`, no `-r`, no directory argument | A fake `tzim` that rejects any token outside an allowlist must fail the suite when an extra token is injected in a test double | manager, tool-executed |
| AC-4 | No non-candidate file ever modified (`SPEC` §15.4) | Per-case before/after SHA-256 for every `skipped_*`, `hardlink_skip`, `failed` and `interrupted` outcome, asserted equal | One expectation deliberately flipped must fail the suite | manager, independently |
| AC-5 | No shell, no prompt, no PTY, no root, no background process (charter §6-3/4/12) | Every spawn site inspected: `std::process::Command` with one `.arg()` per item, no `sh -c` and no string command line anywhere, `.env_clear()` then an explicit minimum, `.stdin(Stdio::null())`; a fake `tzim` that blocks on stdin is killed by the timeout; the child sees a non-terminal on all three streams and `/dev/null` as fd 0; no `pty`, `portable-pty` or `libc` in `Cargo.lock`; process table after exit shows no surviving child | The blocked-stdin fixture is the control — a hang fails the job by timeout, it does not pass quietly | manager, both |
| AC-6 | No retry loop (`SPEC` §15.6) | Second run on an unchanged corpus: zero **per-file** spawns (the two flag-only probes are expected and counted separately), identical statuses | Changing one byte of a previously `failed` file makes it eligible again — proving the invalidation key is live | manager, tool-executed |
| AC-7 | Recovery after cancellation and crash (`SPEC` §15.7) | Injected `SIGKILL` mid-run: the next start reports the run `interrupted`, every state file still parses, a damaged last line is reported as `tail_recovered` rather than repaired, no row stays `running`, no row became `optimized` | A crash after the commit point leaves the run `completed`, not `interrupted` | manager, independently |
| AC-8 | Exit codes are ours, complete and ordered (charter §8.4, §16 P-03, D-18/D-19) | A table executes every command and produces `0/1/2/3/5/6/7/8/130/143` by a **real invocation**, and asserts `4` is never produced; a precedence table asserts every ordering pair of D-19 with a constructed conflict (e.g. a usage error while a lock is held ⇒ `2`); the raw `tzim` code appears only in history, never in `$?` | One row deliberately breaks an expectation to show the table is executed; a fake `tzim` returning `4` must make `auto-tzim` exit `1`, not `4` | manager, tool-executed |
| AC-9 | **No command opens a socket** (`SPEC` §15.9 as overridden by D-28; structural under D-33) | `cargo tree` contains no crate capable of a socket; `cargo deny` bans the known HTTP crates by name; every command and both modes run in a network namespace with no route and no DNS; `update-check` is absent from `--help` and exits `2` if invoked (corpus `W-02`); a `connect` that somehow occurs is caught by the no-route run, not by trust | A "silently skipped" network test, or a swallowed `Result` around a socket call, must fail the suite — proven by planting one and watching it go red | manager, tool-executed + source-inspected |
| AC-10 | Documentation matches behaviour (`SPEC` §15.10) | Reflection test: every documented default equals the schema default; every documented exit code appears in the AC-8 table; the limitations list is checked line by line | A documented default the schema contradicts must fail | manager, source-inspected + tool-executed |
| AC-11 | Preflight refusals return `6`, never `0` (charter §8.4, D-03) | Four real invocations: a second run inside a `--time` window (`too_soon`, [D-47](DECISIONS.md)), which the same command with `--force` must turn into `0`; apply whose plan no longer matches the tree (`--plan <ID>` of a stale record, [D-46](DECISIONS.md)); apply while `pause` is active; apply with free space below `min_free_bytes`. Each exits `6`, prints a reason, processes nothing, starts no `tzim` | The same folder after a successful `scan` exits `0` — proving `6` is the gate and not a broken run | manager, tool-executed |
| AC-12 | Plan binding (D-20) | Apply with a fabricated plan id, a plan whose hash no longer matches the tree, or a plan from another folder ⇒ `6`, nothing processed | Apply with the matching plan ⇒ `0` and the expected outcomes | manager, tool-executed |
| AC-14 | **One install path, two modes** (D-29) | The same corpus run in `confirm` and in `auto`: identical per-file outcomes, identical install sequence, identical history apart from the mode and the approval row; a source-level check that `mode` is read in exactly one branch (the proposal gate) | A second code path for `auto` — an early return, a duplicated install block, a `if auto: install()` — must fail the source check | manager, tool-executed + source-inspected |
| AC-15 | **Crash-safe install**: no file is ever lost (D-29) | `SIGKILL` injected at every step of the journal (before rename-away, between the renames, after rename-in, before unlink, after unlink); after each, the next start restores or completes, and the file exists and is readable in every state | A window in which neither the original nor a restorable copy exists fails the test — including the one between `rename(original → old)` and `fsync` | manager, tool-executed |
| AC-16 | **Changed files are never installed over, never deleted** (D-29) | Identity mutation between plan and install (`dev`, `ino`, `size`, `mtime_ns`, content) for each field separately: the file is `skipped_changed`, its bytes are identical after the run, and the staged copy is deleted | An install that proceeds on a stale stat fails; so does a run that reports `skipped_changed` and installs anyway | manager, tool-executed |
| AC-17 | **`confirm` never hangs and never assumes yes** (D-29) | No TTY, `--json`, EOF, empty line, `no`, a closed stdin, and a read error each ⇒ exit `6` or `0` as documented, nothing installed, nothing written in the folder; measured with a 2 s timeout so a hang is a red test, not a stall | A single case that waits for input without a terminal, or that continues on EOF, fails | manager, tool-executed |
| AC-13 | Fail-closed adapter, no invented capabilities (charter §6-18, §8.4) | Fake and real binaries: unparseable `--version`, version outside the range, absent binary, unknown JSON `schema`, unknown `status`, extra optional field. Unsupported cases ⇒ `3` before anything is planned; an extra field is ignored and recorded, not fatal. A source-level check asserts no module spawns `tzim` with `--info`, `--list-formats` or `--check-deps`, and that no code path reads a `note` to classify | A test double that starts parsing `--list-formats` output must fail the source check | manager, both |

## 2. Safety invariants (charter §6, v0.2)

| ID | Invariant | Evidence | Control that proves the evidence can fail |
| --- | --- | --- | --- |
| INV-1 | Nothing processed before the user adds the folder | empty config ⇒ no plan, no process | a run with no config processes nothing |
| INV-2 | First apply of a new folder needs a read-only plan; dry-run invokes no file-processing `tzim` | AC-2, AC-11, AC-12 | the apply path must produce invocations (AC-2 control) |
| INV-3 | No root/sudo, no hidden background process | process table; no privileged call; no unit files in v0.1 | absence of any surviving child after exit |
| INV-4 | No shell; filenames are argv data | hostile-name corpus: `$(touch /tmp/pwned)` must not create the file; newline, `;`, quotes, leading `-`, non-UTF-8 | the same name passed through a deliberately shell-based test double must create the file |
| INV-5 | No symlink following, no special files | symlink/socket/FIFO/device corpus skipped with a reason; targets untouched | a symlink pointing *inside* the root must also be skipped |
| INV-6 | Hard-linked and unstable files skipped by default | `st_nlink > 1` rows; age and identity-change rows | a pair where one link is modified mid-run ends `skipped_unstable`, never `optimized` |
| INV-7 | Never outside the canonical root | containment re-check before every spawn; a path swapped for a symlink to `/etc/hostname` must abort | that swap must be attempted in the test, and the abort asserted |
| INV-8 | No own codec, no own installer | source review: nothing in the tree writes, renames or unlinks a user file; every `fs::rename` in the tree is either the journaled staging swap or a state file | `tzim` absent ⇒ `3`, nothing written |
| INV-9 | Eligibility/refusal/verification/install stay in `tzim` | one mapping table (D-02); the verdict is recorded, never re-decided | a file `tzim` refuses but our policy allows stays `skipped_by_tzim` |
| INV-10 | No re-routing of refused/unsupported/failed | no fallback branch; one spawn per file per run | code review plus an invocation-count assertion |
| INV-11 | `transforms` empty in the MVP | `transforms = ["fits-tile"]` ⇒ validation error before any planning | the same config with `[]` must be accepted |
| INV-12 | No prompt, no PTY, in any mode | AC-5 | blocked-stdin fixture |
| INV-13 | `failed` never retried automatically | second run: no invocation for a failed file | manual retry after the file changes succeeds |
| INV-14 | v0.1 contains no download, no installer and no network code at all (D-28) | `cargo tree` shows no crate capable of a socket; `cargo deny` bans the known ones by name; no `update-check` command exists | a planted `std::net::TcpStream::connect` call, or one `ureq`/`reqwest` line in a manifest, must fail the source check |
| INV-15 | No telemetry, no undocumented request — and in v0.1 no documented one either | `cargo tree` has no crate that can open a socket; a source check forbids `std::net` outside one `#[cfg(test)]` guard; the acceptance run happens in a network namespace with no route | a test that forgets the namespace, or a `std::net` use left unguarded, fails the check |
| INV-16 | No false undo promise | documentation review; no backup code; only disclaimers mention undo | text search for "undo" finds only disclaimers |
| INV-17 | UI does not duplicate core logic | not applicable in v0.1 (no UI); v0.2/v0.4 gate | — |
| INV-18 | No prose, `note`, extension or guess for capabilities and reasons | AC-13 source check; the generic `skipped_by_tzim` with the verbatim note | a test double that classifies from a note must fail the check |
| INV-19 | Own exit codes, `6` for preflight, no raw passthrough, fixed order and precedence | AC-8, AC-11 | a fake `tzim` exit `4`/`8` must not appear as our `$?` |
| INV-20 | The user's folder is not written before the install step of a file: all processing happens on a staged copy (D-29) | AC-14, AC-15 | the invocation log shows a path invocation naming a file inside the folder | a run whose `tzim` argv contains any path inside the root fails the test |
| INV-21 | An original is deleted only after its verified replacement is in place and its directory is `fsync`ed — never in a batch, never for a file that was not replaced | AC-14, AC-15 | journal rows + a `SIGKILL` at each step | an install that unlinks first, or that deletes a file whose replacement failed |
| INV-22 | Identity is re-checked immediately before the work on each file; a changed file keeps its bytes | AC-16 | `M-09`…`M-10` | an install that proceeds on a stale stat |
| INV-23 | `confirm` never hangs and never assumes yes; a declined proposal is exit `0` with zero writes | AC-17 | `M-02`…`M-06` | a run that continues on EOF, or that exits non-zero because the user said no |

## 3. Evidence a handoff must contain

Per charter §7.1: final head SHA; issue/commit map; CI run IDs with their SHA; raw per-case and per-file rows; the
invocation counter for every scan/dry-run case; independently recomputable aggregates; artifact SHA-256 values;
OS/arch/toolchain/`tzim` identity; reproduction commands; documentation updated; and `remaining blocker: none` or an exact
blocker per charter §10.

The manager marks each claim **independently verified / source-inspected / tool-executed / coder-reported** in the
review document. A claim that stays `coder-reported` is a finding, not a pass.

## 4. How the rows are checked

Each row above is verified with the commands in [EVIDENCE-PROTOCOL.md](EVIDENCE-PROTOCOL.md): SHA binding first, then CI
runs (never `skipped`), then row reconciliation by `jq`, then byte evidence, then the read-only-plan proof, then the
executed exit-code matrix, then six negative controls the reviewer performs personally. A row that cannot be checked that
way is a finding against AT-08, not a pass.

## 5. Stop rules

- Any Critical finding (charter §9) stops the review and the merge recommendation, regardless of the remaining rows —
  including a single write into a target root during scan/dry-run.
- A High finding stops acceptance; only the owner may list it as a follow-up.
- A required job that is skipped, cancelled, or replaced by a placeholder is a **failed** row.
- Evidence from any SHA other than the final head is invalid for the decision.
- A preflight refusal that returns `0`, a passed-through `tzim` code, or an undefined precedence is a **High** finding
  by charter §9, not a documentation gap.
