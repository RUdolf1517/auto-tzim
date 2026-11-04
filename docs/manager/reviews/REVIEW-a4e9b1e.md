# Review — a4e9b1e (AT-02, draft PR #34)

Reviewer: manager (this session) · Date: 2026-11-04
Scope: **AT-02 (#2)** — config model, folder CRUD, atomic writes, kernel lock.
Head: `a4e9b1e`, 11 files, +2470 −29. Draft; the coder has declared it not ready, and this review agrees.

**Verdict: NOT READY — not because the code is bad (it is the strongest work in this repository so far), but because
five of the seven decisions in [D-54](../DECISIONS.md) are not implemented in it yet, and one of them is a defect.**

## 1. What I verified myself

| Item | Result | Evidence |
| --- | --- | --- |
| CI run at this head | **green, 5 of 5 jobs**, `pull_request`, both platforms | run **41900805601**, head `a4e9b1e` |
| macOS acquisition after D-51 | **4 s** from run creation to job start (6 s including dispatch) | job 126822386689 timings |
| evidence is machine-readable, not prose | **pass** — annotations on both `test` jobs | 126822386689, 126822386939 |
| the eight commands leave a corrupt config byte-identical | **pass** — one SHA before and after: `12f3de65…`, plus a per-platform SHA from the unit test | `Corrupt configuration` annotation |
| rejected edit leaves the file and its `stat` untouched | **pass** — identical SHA, identical size/mode/mtime | `Edit evidence` annotation |
| refusal matrix | **pass** — `root_missing`, `root_is_symlink`, `root_not_a_directory`, `forbidden_root`, `root_not_absolute`, `state_inside_root`, `cloud_sync_root` each with its own message | `Refusal *` annotations |
| identity: duplicates, case-insensitive lookup, rename, no old-name residue | **pass** | `Identity evidence` annotation |
| F-19 (`concurrency` cancelling push runs) | **fixed** — `cancel-in-progress: ${{ github.event_name == 'pull_request' }}` | `ci.yml` head |
| F-20 (failures hidden in pipes) | **fixed** — `set +e` + `PIPESTATUS[0]` + an `::error` annotation carrying the last 3000 bytes | `ci.yml` clippy/test/deny steps |
| no `unsafe`, no `libc`, no FFI | **pass** — `grep` over `src/` ends with nothing | source-inspected |
| the only `std::process` use is `id()` | **pass** — `config.rs:1105`; no spawn, no `Command` in `src/` | source-inspected |
| lock is the kernel lock, not a file convention | **pass** — `try_lock()` on `$AUTO_TZIM_HOME/locks/config.lock`, `WouldBlock` → holder diagnostics → exit `8` | `config.rs:1076` |
| write is load → validate → round-trip → `create_new` tmp `0600` → `sync_all` → rename → dir fsync | **pass**, order read line by line | `save()` |
| the lock file is never deleted | **pass** — `remove_stale_temps` matches only our tmp pattern | `config.rs` |
| `0600`/`0700` do not depend on the umask | **pass** — `permissions_are_private_under_umask_zero` exists and ran on both platforms | `tests/config_cli.rs:127` |
| 36 test functions, including SIGKILL release and symlinked lock path | **pass** | `tests/config_cli.rs` |

That is a real green, and I am not going to pretend otherwise: the six controls the coder was asked for are there, the
evidence is in the annotations where a machine can read it, and the two CI defects are gone.

## 2. What blocks it

**F-21 (Medium, fixed by decision, not yet in code).** The lock is taken in `execute()` before `load()`, for **every**
command, and taking it creates `$AUTO_TZIM_HOME/locks/`. So `auto-tzim config validate` — the one command `cli.md` §2.2
promises will work "without touching state" — creates a directory. It is unnecessary as well: config writes are atomic
renames, so a reader sees a whole file without any lock. `config validate`, `folder list` and `folder show` must take
**no** lock; the mutating commands keep it. This is the only finding here that is a broken promise rather than a missing
feature.

**Four decisions not implemented yet** ([D-54](../DECISIONS.md)):

1. an explicit `mode = "auto"` is **valid** — the code rejects it, and the message says "default must be confirm", which is
   a different rule than the one in `config.md`;
2. a non-empty `tzim_path` must exist, be a regular file and be executable — `config validate` does not check;
3. `--allow-cloud-sync` / `--no-allow-cloud-sync` do not exist on `folder add` / `folder edit`;
4. the permissions control is accepted **plus** a repair test (a config already at `0666` ends `0600` after a mutating
   command) — the control exists, the repair test does not.

**One naming defect:** the lock file writes `start_time` as the process entry time. An OS-reported creation time is not
available without a dependency, and a field whose name says "creation" while its value says "entry" is exactly the kind of
small lie `state.md` exists to prevent. Rename to `started_at`, or drop the field; do not add a dependency for it. `host`
is `HOSTNAME` when set and **omitted** when not — never the literal `"unknown"`.

## 3. What is still open, and is not a finding

- **R-27 stays open.** This run acquired its macOS runner; the run at `41898092925` between the coder's two fast ones did
  not, for 15 minutes. One green run proves the change works when capacity exists, not that capacity always exists. The
  three consecutive fast acquisitions the coder reports are **coder-reported** — my own API check saw `41900805601` and
  saw `41898092925` fail, and did not see the other two. [D-55](../DECISIONS.md) makes `macos-15-intel` a measurement, not
  a switch.
- **The PR body is stale.** It still carries a "Lock implementation decision needed" section that D-53 answered and D-54
  refined. A body that contradicts the decisions is worse than no body, because the next reader trusts it.
- **No merge.** PR #34 is a draft on a branch that changes the product; it needs the green run, the fixes above, and the
  owner's word.

## 4. The rule this review is applying

Green CI is not proof, and the coder's summary is not evidence: everything in §1 marked `pass` was read in the source or
read back from the API. Everything in §2 marked as missing was checked by looking for it and not finding it. `not-run` is
not `passed`, and a cancelled run is not a failed one.
