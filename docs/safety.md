# Safety model

> **Status: specification, no implementation.** Each property below names the clause it comes from and the test that
> must exist before acceptance; none of those tests exists yet.
> Related: [architecture](architecture.md) · [guarantees](guarantees.md) · [recovery](recovery.md) ·
> [limitations](limitations.md)

`auto-tzim` does not modify files. Its safety work is to decide **which files may reach `tzim`**, to prove that a file
did not change behind its back, and to keep an honest record. The single most dangerous thing this program could do
is hand the wrong path to a program that rewrites files in place, so every control below exists to prevent that.

## 1. The seventeen invariants and where they are enforced

| # | Invariant | Enforced by | Test that must exist |
| --- | --- | --- | --- |
| 1 | Nothing is processed before the user adds the folder | empty config ⇒ no plan, no process | INV-1 |
| 2 | The first run of a new folder is dry-run only | state gate + CLI refusal | AC-2 |
| 3 | No root/sudo, no hidden background activity | no privileged call anywhere; the process ends when the command ends | INV-3 |
| 4 | No shell; filenames stay argv data | argv arrays only, `shell=False` | INV-4 |
| 5 | No symlink following, no special files | `lstat`, mode checks, canonical containment | INV-5 |
| 6 | Hard-linked and unstable files skipped by default | `st_nlink > 1`; age and identity checks | INV-6 |
| 7 | Never outside the canonical approved root | validation + containment re-check before each spawn | INV-7 |
| 8 | No codecs, no own installer | architectural: nothing in the tree can write a user file | INV-8 |
| 9 | Eligibility, refusal, verification, installation stay in `tzim` | adapter maps, never decides | INV-9 |
| 10 | No re-routing of unknown/unsupported/refused/failed | one path, no fallback branch | INV-10 |
| 11 | Confirmation-required transformations need explicit opt-in | empty `transforms` allowlist in v0.1 | INV-11 |
| 12 | Scheduler/non-TTY/JSON never wait for a prompt | `stdin=DEVNULL`, no prompt code, timeout | INV-12 |
| 13 | `failed` is never retried automatically | retry policy keyed on the invalidation key | INV-13 |
| 14 | `update-check` never downloads or replaces a binary | no download code exists in v0.1 | INV-14 |
| 15 | No telemetry, no undocumented requests | `updater` is the only network client | INV-15 |
| 16 | No promise of undo | documentation and `--help`; no backup code | INV-16 |
| 17 | The UI never duplicates core logic | review gate for v0.2/v0.4 | INV-17 |

Invariants 1–16 are the charter's; changing one is an owner decision, not a code review remark.

## 2. Layers, in the order a path passes through them

1. **Config validation.** The root is absolute, existing, a directory, not a symlink, not forbidden, not inside
   `AUTO_TZIM_HOME`, canonicalised and stored in canonical form. A rejected root never reaches discovery.
2. **Discovery.** `lstat` only. Entries sorted by relative POSIX path. Hidden files per profile, mount boundaries per
   profile, `exclude_globs`, dot-prefixed `tzim` temporary files always excluded. Regular files only: FIFOs, sockets,
   device nodes, symlinks and directories are skipped with a reason, not an error.
3. **Stability.** Identity `(st_dev, st_ino, size, mtime_ns)` captured; `st_nlink > 1` ⇒ `hardlink_skip`; age below
   `min_file_age_seconds` ⇒ `skipped_unstable`. The same identity is re-read immediately before the spawn.
4. **Policy.** Format allowlist, `transforms` (empty in v0.1), limits, and the opt-in gates. A file excluded here is
   `skipped_policy` and never spawns a process.
5. **Runner.** `argv` array, minimal environment, `stdin=DEVNULL`, stdout/stderr captured with a size cap, a per-file
   timeout, `TERM` then `KILL`. After exit, the file's identity is read once more.
6. **State.** The outcome is written in one transaction with the identity, the argv, the `tzim` binary identity and
   the raw result. A run interrupted by a crash is recovered as `interrupted`, never as `optimized`.

A file that fails any layer is finished: no layer below is tried, and no layer above is re-entered.

## 3. The dangerous windows, named

| Window | What could go wrong | Control |
| --- | --- | --- |
| Between discovery and spawn | the path is replaced by a symlink or moved out of the root | identity re-check + canonical containment re-check immediately before the spawn; on any difference, skip |
| Between spawn and exit | another program modifies the file while `tzim` works | `tzim` itself detects concurrent modification and refuses; we re-read identity after exit and record any difference |
| Between exit and state write | our process is killed; the run looks unfinished | the run is opened with a heartbeat and closed in one transaction; a stale `running` row becomes `interrupted` |
| During a dry run | a temporary candidate is written into the user's folder | documented honestly; we never create one ourselves; orphans are reported, never deleted ([P-02](manager/problems/P-02-dry-run-is-not-write-free.md)) |
| During `update-check` | a network request discloses the client | one request, one command, documented ([privacy](privacy.md)) |
| At config write | a crash mid-write truncates the config | temp file + `fsync` + rename + directory `fsync`; the previous file survives until the rename |

## 4. Failure-mode table (the target, to be proven row by row)

| Condition | Our status | Exit | User file touched? |
| --- | --- | --- | --- |
| File is a symlink | `skipped_policy` | `0` | no |
| File is a FIFO, socket or device | `skipped_policy` | `0` | no |
| File has `st_nlink > 1` | `hardlink_skip` | `0` | no |
| File changed since discovery | `skipped_unstable` | `0` | no |
| File deleted since discovery | `skipped_unstable` | `0` | no |
| Path resolves outside the root before the spawn | `skipped_policy` + diagnostic | `0` | no |
| `tzim` refuses the content | `skipped_refused` | `0` | no |
| `tzim` reports the format unsupported | `skipped_unsupported` | `0` | no |
| `tzim` exits `1` for the file | `failed` (`io`/`parse`) | `1` | unchanged (tzim guarantees) |
| `tzim` exits `4` (verification failure) | `failed` (`verify`), plan aborted, rest `interrupted` | `1` | unchanged for that file |
| `tzim` hangs | `failed` (`timeout`) | `1` | unchanged |
| `tzim` missing or unsupported version | no file status; run does not start | `3` | no |
| Another run is active | — | `4` | no |
| Free space below `min_free_bytes` | no file status; run refuses | `6` (proposed) | no |
| `SIGINT` during the run | in-flight and remaining files `interrupted` | `130` | earlier files keep verified changes |
| Our own crash | stale run recovered as `interrupted` | — | per-file, as above |

## 5. What we explicitly refuse to add for convenience

- `--force`, `--yes`, "ignore errors", "continue on verification failure": each one converts a refusal into an
  action, and the refusal is the product.
- Automatic retry of `failed`, retry after a timeout in the same run, or a backoff loop: invariant 13.
- Following symlinks, or "resolving" a hard-linked file by choosing one sibling.
- A background helper, a menu-bar item, a tray icon, a watch mode: invariant 3, and the scheduler stage is a separate
  reviewed design.
- Deleting `.tzim-*.tmp` orphans: they are not our files; we report them.
- Widening a root automatically when the user "obviously meant the parent folder".

## 6. Reporting a safety problem

A file changed outside the plan, or a change during a dry run, or a prompt appearing in a non-interactive run is a
**Critical** finding (charter §9): it stops the merge and the release until it is reproduced and fixed. The report
needs the `auto-tzim` version, the `tzim` binary identity from `doctor`, the run id, the affected path, and the raw
rows — which is exactly what the history keeps, so a user can produce it without re-running anything.
