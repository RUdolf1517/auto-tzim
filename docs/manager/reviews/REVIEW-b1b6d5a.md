# Review — b1b6d5a (AT-02, PR #34, no longer a draft)

Reviewer: manager (this session) · Date: 2026-11-04
Scope: **AT-02 (#2)** — config model, folder CRUD, atomic writes, kernel lock, D-54 implementation.
Head: `b1b6d5a626cc0dcf1f58e7903c3f5ed5fa241f68`, 11 files vs `main`, +2690 −28. Previous:
[REVIEW-a4e9b1e](REVIEW-a4e9b1e.md) — NOT READY.

**Verdict: NOT READY — one High finding, and it is a trap the tool must not ship with.**

Everything I asked for in [D-54](../DECISIONS.md) is here and works. What the implementation exposes is a *different*
problem, one my own answer created: a configuration that names a folder on an unplugged disk, or a `tzim_path` that is no
longer installed, cannot be read at all — so the tool cannot even remove the profile that is in the way. Recorded as
[D-56](../DECISIONS.md) and **F-22**.

## 1. F-22 (High) — an unplugged disk bricks the configuration

`load()` → `validate_config()` → `validate_root(&folder.root, …)`, and `validate_root` asks the filesystem. `load()` returns
`Err` on any error, so `main` prints and exits `2`. `save()` repeats the same check before writing.

The result, for a stored profile whose root is on an external disk that is not mounted:

| Command | Today | Should be |
| --- | --- | --- |
| `folder list` | exit `2` | exit `0`, prints the profile |
| `folder show trips` | exit `2` | exit `0` |
| `folder disable trips` | exit `2` | exit `0` |
| `folder remove trips` | exit `2` | exit `0` — the escape hatch |
| `config validate` | exit `2`, `root_missing` | exit `2`, `root_missing` — **correct as is** |

The only repair left to the user is hand-editing TOML, in a tool whose reason to exist is not hand-editing TOML. The same
shape applies to the `tzim_path` check I asked for in D-54 item 2: uninstall `tzim` and every command refuses.

The fix is a split, not a deletion ([D-56](../DECISIONS.md)):

- **structure** — schema, enums, names, duplicates, path shape, `forbidden_root`, `state_inside_root`, `cloud_sync_root`,
  reserved features, ranges, format names, `tzim_path` shape: checked on **every** load and save, exactly as now;
- **environment** — root exists / is a directory / is not a symlink / canonicalises; `tzim_path` exists, is a regular file,
  is executable: checked only by `config validate`, by `folder add`, and at run time.

Note what this does **not** change: a corrupt config is still never overwritten (corruption is a structure failure), `folder
add` still refuses a missing root, and `config validate` still reports reality. Nothing is weakened; one refusal is moved to
the command whose name says what it does.

**This is my error, not the coder's.** D-54 item 2 said *what* to check and not *where*, and the implementation put it
where the rest of validation lives — the natural place. The review at `a4e9b1e` missed it too; it was found by reading the
call chain in the new head, which is the argument for reading the code rather than the summary.

## 2. Three small findings

| # | Severity | Finding | Fix |
| --- | --- | --- | --- |
| F-23 | Low | `lock.as_ref().expect("mutating command holds the lock")` appears five times in `main.rs`. Unreachable today, because `mutating` is computed from the same `matches!` — but a panic in a tool whose contract is exit `0`/`2`/`8` is the wrong failure, and `expect` re-states an invariant instead of encoding it | build the lock once and pass `&ConfigLock` into a helper, or return a typed `ConfigError::Internal`; no panic |
| F-24 | Low | `validate_tzim_path` reports `tzim_path_not_executable` for *any* metadata error (e.g. `EACCES` on a parent) and for a directory. A message that names the wrong rule sends the user to fix the wrong thing | keep one code, append the OS error, and say "not a regular file" when `is_file()` is false |
| F-25 | Low | `use std::os::unix::fs::PermissionsExt` is un-gated, so the crate cannot build on Windows. That matches [limitations](../../limitations.md) ("macOS and Linux only"), but the constraint is now in the code without being stated there | one comment naming the scope; no `cfg` gymnastics for a platform we do not claim |

## 3. What is verified, and how

The coder's summary is not evidence, so everything below was read in the source or read back from the API.

| Item | Result | Evidence |
| --- | --- | --- |
| final head green on both platforms | **pass** — 5 of 5 jobs | run **41905359818** at `b1b6d5a`, macOS queued 20:18:34 → started 20:21:19 |
| F-21 fixed: `validate` / `list` / `show` take no lock | **pass** | `mutating = !matches!(read-only whitelist)` in `execute()` — the **safe** direction: a new command locks unless named |
| F-21 proven while a writer holds the lock | **pass** — this is the test I wanted and did not have to ask twice for | `lock_contention_…`: holder alive → `config validate`/`list`/`show` succeed, `folder disable` exits `8` naming the holder's pid; after `SIGKILL` the mutating command succeeds and `locks/config.lock` still exists |
| F-21 proven for a pristine home | **pass** | `read_only_commands_do_not_create_or_lock_state`: `AUTO_TZIM_HOME` absent, the directory is still absent after three commands |
| D-54 item 1 — `mode = "auto"` | **pass** | both `validate_document` and `validate_config` accept `confirm`/`auto`, message names both; test covers `auto`, `sometimes` |
| D-54 item 2 — `tzim_path` | **pass on the check, wrong on the placement** | test covers missing / directory / not executable / executable; see F-22 |
| D-54 item 3 — cloud flags | **pass** | `--allow-cloud-sync` / `--no-allow-cloud-sync` with `conflicts_with`; `validate_root` receives the flag, so add succeeds and a later `--no-allow-cloud-sync` edit is refused with the bytes unchanged |
| D-54 item 4 — permissions | **pass** | repair test: config forced to `0666`, one mutating command, `0600` after |
| D-54 item 5 — lock metadata | **pass** | `started_at`, `host` only when `HOSTNAME` is set, asserted absent and `"unknown"` asserted absent |
| D-54 item 6 — stale temps under the lock | **pass** | unchanged, now with the comment that says *why* it is legal |
| six negative controls are real | **pass** — verified, not accepted on trust | each red run's SHA is an ancestor of the revert commit listed: `6055b1d`, `5fed1c1`, `3954593`, `228b5bc`, `55923a2`, `613cf47`; each red run failed in **Cargo tests** on the claimed job |
| the sixth control is the D-53 one | **pass** | `07c44ad` replaces `try_lock()` with `create_new(true)` + `Drop` deleting the file; red run `41885760161` failed reacquisition after `SIGKILL` |
| the permissions control removes **both** layers | **pass** | `c3ce6cd` drops `.mode(0o600)`, `set_permissions`, and sets the directory mode to `0777` |
| no `unsafe`, no `libc`, no network, no spawn in product code | **pass** | `git show b1b6d5a:<each src file>` greps clean for `unsafe`, `libc`, `Command::new`, `std::net`, `thread::spawn`, `remove_dir_all` |
| F-19, F-20 stay fixed; no `continue-on-error`, no `\|\| true` | **pass** | `ci.yml` head |
| the PR body was rewritten, the stale lock section is gone | **pass** | body now carries head, runs, per-acceptance evidence, the six controls table, `cargo tree` |

## 4. On the merge order, and one thing not to repeat

The branch is based on `06ba2a6`, one commit behind `main` (`5a74208`, the STATUS-LOG row). That is why `git diff main…head`
*appears* to delete two log rows: they never existed in the base, so the merge will not delete them. Verified by reading the
merge base, not the diff — a diff against a newer base is not a merge simulation.

PR #37's own run `41905152395` shows as `failure` for the same reason a run is often called failed here: `test (macos-15)`
was **cancelled** after 23 minutes without ever starting. That is `not-run`, not `failed` ([D-51](../DECISIONS.md)).

## 5. What has to happen before AT-02 can be accepted

1. F-22 / [D-56](../DECISIONS.md): split structure from environment; `load()` and `save()` keep structure only.
2. Acceptance point `13` in [HANDOFF-AT-02](../HANDOFF-AT-02.md): prove it — a config with a deleted root and a deleted
   `tzim_path`, where the five read-only/mutating folder commands succeed and `config validate` exits `2`.
3. F-23, F-24, F-25 — small, and cheaper now than in AT-03.
4. A green run at the new head, then the owner's word. The coder does not merge, and neither do I without that word.
