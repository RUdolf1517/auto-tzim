# Safety model

> **Status: specification, no implementation.** Each property names the clause it comes from and the test that must exist
> before acceptance. The operative text is charter v0.2 §6 (nineteen invariants); the technical specification's older
> invariants 2 and 11 are superseded — see [CONFLICTS](spec/CONFLICTS.md).
> Related: [architecture](architecture.md) · [guarantees](guarantees.md) · [recovery](recovery.md) · [limitations](limitations.md)

`auto-tzim` does not modify files. Its safety work is to decide **which files may reach `tzim`**, to prove that a file did
not change behind its back, and to keep an honest record. The dangerous thing this program could do is hand the wrong
path — or the right path at the wrong time — to a program that rewrites files in place.

## 1. The nineteen invariants and where each is enforced

| # | Invariant (charter §6, v0.2) | Enforced by | Test |
| --- | --- | --- | --- |
| 1 | Nothing processed before the user adds the folder | config gate; empty config ⇒ empty plan | INV-1 |
| 2 | Before a folder's first apply there must be a **read-only candidate plan**; scan/dry-run runs no file-processing `tzim`, passes no `--dry-run`, writes nothing in the target root; only `tzim --version` is allowed | planner never spawns the runner; runner is reachable only from apply | AC-2, AC-11, AC-12 |
| 3 | No root/sudo, no hidden background activity | no privileged call; the process ends with the command | INV-3 |
| 4 | No shell; filenames stay argv data | `shell=False`, argv arrays, `--` for leading dashes | INV-4 |
| 5 | No symlink following, no special files | `lstat`, mode checks, canonical containment | INV-5 |
| 6 | Hard-linked and unstable files skipped by default | `st_nlink > 1`; age and identity checks at three stages | INV-6 |
| 7 | Never outside the canonical approved root | validation + containment re-check before every spawn | INV-7 |
| 8 | No codecs, no own installer | architectural: nothing in the tree writes a user file | INV-8 |
| 9 | Eligibility, refusal, verification, installation stay in `tzim` | one mapping table; verdict recorded, never re-decided | INV-9 |
| 10 | No re-routing of unknown/unsupported/refused/failed | one path, no fallback branch | INV-10 |
| 11 | `transforms` must be empty in the MVP; confirmation-required transformations do not run | validation rejects any value (D-05) | INV-11 |
| 12 | Scheduler/non-TTY/JSON never wait for a prompt and never allocate a PTY | `stdin=DEVNULL`, no `pty` import, timeout | INV-12 |
| 13 | `failed` never retried in an automatic loop | retry policy keyed on the invalidation key | INV-13 |
| 14 | Update-check downloads and replaces nothing | no download code exists in v0.1 | INV-14 |
| 15 | No telemetry, no undocumented requests | `updater` is the only network client | INV-15 |
| 16 | No promise of undo | documentation and `--help`; no backup code | INV-16 |
| 17 | The UI never duplicates core logic | review gate for v0.2/v0.4 | INV-17 |
| 18 | Default-enabled set and refusal reasons are never reconstructed from human output, extensions or the free-text `note`; an unknown contract is fail-closed `incompatible_tzim` | restricted adapter (D-01): `tzim --version` only; generic `skipped_by_tzim` (D-02) | AC-13 |
| 19 | No passthrough of a raw `tzim` exit code; preflight/pause/resource refusals have their own code `6`; internal errors and signals have documented codes and precedence | own exit contract (D-18/D-19) | AC-8, AC-11 |

Invariants 2, 11, 12, 18 and 19 were **tightened** by charter v0.2; the older technical specification allowed a `tzim`
dry run during a preview and did not fix an exit code for refusals. Where the two disagree, the charter wins.

## 2. Layers, in the order a path passes through them

1. **Config validation.** Root absolute, existing, a directory, not a symlink, not forbidden, not inside
   `AUTO_TZIM_HOME` (D-08), canonicalised. A rejected root never reaches discovery.
2. **Discovery.** `lstat` only. Entries sorted by relative POSIX path. Hidden files, mount boundaries and
   `exclude_globs` per profile; `.tzim-*.tmp` always excluded. Regular files only; FIFOs, sockets, device nodes,
   symlinks and directories are skipped with a reason, never errors.
3. **Stability.** Identity `(st_dev, st_ino, size, mtime_ns)`; `st_nlink > 1` ⇒ `hardlink_skip`; younger than
   `min_file_age_seconds` ⇒ `skipped_unstable`; identity re-read immediately before the spawn; identity re-read again
   after the process exits.
4. **Policy.** Format allowlist, `transforms` (must be empty), limits and opt-in gates. Excluded here ⇒ `skipped_policy`
   and no process.
5. **Plan.** A record with an id and a hash over the ordered rows (D-20). Nothing runs without a matching plan.
6. **Runner.** `argv`, `shell=False`, `stdin=DEVNULL`, no PTY, captured output with a size cap, per-file timeout,
   `TERM` then `KILL`.
7. **State.** One transaction per file with identity, argv, binary identity and the raw result. A crashed run is
   recovered as `interrupted`, never `optimized`.

A file that fails any layer is finished: no lower layer is tried, no upper layer re-entered.

## 3. The dangerous windows, named

| Window | What could go wrong | Control |
| --- | --- | --- |
| Between scan and apply | the tree changed; the user applies a stale plan | plan hash over ordered identities; any difference ⇒ exit `6` (D-20) |
| Between discovery and spawn | the path is replaced by a symlink or moved out of the root | identity + containment re-check immediately before the spawn |
| During the apply of one file | another program writes the file while `tzim` works | `tzim` refuses concurrent modification; we re-read identity after exit and record any difference |
| During an apply, in the directory | `tzim` creates `.tzim-<pid>-<ns>-<attempt>.tmp` and removes it; a hard kill leaves it | documented; never planned; never deleted by us (R-13) |
| Between exit and state write | our process dies; the run looks unfinished | heartbeat plus one closing transaction; a stale run becomes `interrupted` |
| During scan/dry-run | *nothing may happen* — this is the invariant that changed in v0.2 | planner cannot reach the runner; invocation counter asserts zero path invocations and an unchanged root (AC-2) |
| During `update-check` | one request discloses the client IP | one request, one command, documented ([privacy](privacy.md)) |
| At config write | a crash mid-write truncates the config | temp file + `fsync` + rename + directory `fsync` |

## 4. Failure-mode table (target; each row becomes a test)

| Condition | Status | Exit | User file touched? |
| --- | --- | --- | --- |
| Symlink, FIFO, socket, device | `skipped_policy` | `0` | no |
| `st_nlink > 1` | `hardlink_skip` | `0` | no |
| Changed since scan | `skipped_unstable` | `0` | no |
| Deleted since scan | `skipped_unstable` | `0` | no |
| Path resolves outside the root before the spawn | `skipped_policy` + diagnostic | `0` | no |
| `tzim` returned `skipped` (any reason) | `skipped_by_tzim` + verbatim note | `0` | no |
| `tzim` returned `unsupported` | `skipped_unsupported` | `0` | no |
| `tzim` exited `1` for the file | `failed` (`io`/`parse`) | `1` | no (tzim's guarantee) |
| `tzim` exited `4` (verification failure) | `failed` (`verify`), plan aborted, rest `interrupted` | `1` | no |
| `tzim` hung | `failed` (`internal`, `timeout`) | `1` | no |
| `tzim` absent / unreadable / out of contract | run does not start | `3` | no |
| Another run holds the lock | run does not start | `8` | no |
| No matching plan (first apply of the folder) | run does not start | `6` | no |
| `pause` active | run does not start | `6` | no |
| Free space below `min_free_bytes` | run does not start | `6` | no |
| Corrupt state database | run does not start | `7` | no |
| `SIGINT`/`SIGTERM` during a run | in-flight and remaining files `interrupted` | `130`/`143` | earlier files keep verified changes |

## 5. What we refuse to add for convenience

- `--force`, `--yes`, "ignore errors", "continue on verification failure": each converts a refusal into an action.
- An automatic retry of `failed`, a retry after a timeout inside the same run, or a backoff loop: invariant 13.
- Following symlinks, resolving a hard-linked file by picking one sibling, widening a root to "the obvious parent".
- A dry run that invokes `tzim` on a target path because "the numbers would be nicer": invariant 2, charter §16 P-02.
- Guessing a file's format from its extension, or a refusal reason from a `note`: invariant 18.
- A background helper, tray icon, watch mode or menu-bar item: invariant 3.
- Deleting `.tzim-*.tmp` orphans: they are not our files.
- An undo store, or any wording that suggests one.

## 6. Reporting a safety problem

Any of these is **Critical** (charter §9) and stops merge and release until reproduced and fixed:

- a write, candidate or temporary file appearing in a target root during `auto-tzim` scan/dry-run;
- a file outside the plan, or outside the canonical root, being touched;
- a change to an original during a refusal, a failure or a verification error;
- a refusal reported as `0`, or a raw `tzim` code appearing as `$?`;
- any evidence of prompt/PTY behaviour in a non-interactive run.

A report needs the `auto-tzim` version, the `tzim` identity from `doctor`, the run id, the plan id, the affected path and
the raw rows — which is exactly what history and the plan record keep.
