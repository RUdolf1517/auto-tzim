# State, history and locks

> **Status: specification, no implementation.** The schema below is a plan; the first implementation must match it or
> bring a decision.
> Related: [architecture](architecture.md) · [cli](cli.md) · [recovery](recovery.md) · [privacy](privacy.md)

Everything `auto-tzim` remembers lives in one SQLite database and one log directory under `AUTO_TZIM_HOME`. There is
no cloud state, no cache keyed on the network, and no file inside the user's folders.

## 1. Tables

| Table | Purpose | Key columns |
| --- | --- | --- |
| `runs` | one row per `run --once` (and per refused run) | `id`, `started_at`, `finished_at`, `state` (`running`\|`completed`\|`aborted`\|`interrupted`), `dry_run`, `tzim_path`, `tzim_sha256`, `tzim_version`, `adapter`, `config_hash`, `exit_code` |
| `files` | one row per file per run | `run_id`, `folder_id`, `relative_path`, `status`, `reason_code`, `before`, `after`, `saved`, `dev`, `ino`, `size_before`, `mtime_ns_before`, `size_after`, `mtime_ns_after`, `sha256_before`, `sha256_after`, `tzim_status`, `tzim_note`, `tzim_error_kind`, `tzim_error_message`, `duration_ms`, `argv_json` |
| `eligibility` | the "do not re-examine" cache | `folder_id`, `relative_path`, `dev`, `ino`, `size`, `mtime_ns`, `tzim_sha256`, `adapter`, `policy_hash`, `status`, `reason_code`, `decided_at` |
| `plans` | one row per `scan` / `run --dry-run` (D-20) | `plan_id`, `folder_id`, `created_at`, `rows`, `plan_hash`, `policy_hash`, `tzim_sha256`, `adapter`, `state` (`complete`\|`superseded`\|`used`) |
| `locks` | active run and per-file guards | `run_id`, `pid`, `host`, `started_at`, `heartbeat_at` |
| `meta` | schema version and installation facts | `key`, `value` |

Design rules:

- `files` is append-only history: a row is never edited after its run finished. A re-run creates new rows.
- `eligibility` is a cache, and it is invalidated by identity **or** by any of `tzim_sha256`, `adapter`, `policy_hash`
  changing (D-06). A cache entry never claims a file is "safe to skip" on the strength of an old binary.
- `argv_json` is stored because it is the only way to explain a result later. It contains an absolute path, which is
  why logs and exports are treated as sensitive (§ [privacy](privacy.md)).
- `plans.plan_hash` is a SHA-256 over the ordered tuples `(relative_path, size, mtime_ns, dev, ino, policy_hash,
  tzim_sha256)`. An apply compares it with the tree it is about to process; any difference is exit `6`, never "close
  enough" (D-20). A plan is `used` once an apply starts and `superseded` by a newer scan of the same folder.
- The normalized status vocabulary is fixed by D-02: `planned`, `optimized`, `already_optimal`, `skipped_unsupported`,
  `skipped_by_tzim`, `skipped_policy`, `skipped_unstable`, `hardlink_skip`, `failed`, `interrupted`, `incompatible_tzim`.
  The raw `tzim` status, note and exit code live in their own columns, never in `status`.
- Timestamps are stored in UTC, ISO 8601 with microseconds; `mtime_ns` is stored as an integer, never as a float.

## 2. Transactions and crash behaviour

- A run is opened with `state = 'running'` and a heartbeat, before the first file. Every file outcome is committed
  individually, so a crash loses at most one in-flight file's record.
- The transition to `completed`/`aborted` and the final counts are one transaction.
- On startup, any `running` row whose process is gone (checked by pid plus a start-time discriminator, not pid alone —
  pids are reused) becomes `interrupted`. A row can therefore never remain `running` forever, and can never be
  upgraded to `optimized` by recovery.
- A file that was in flight when the process died is recorded as `interrupted` with the identity captured before the
  spawn. It is not retried automatically.
- `PRAGMA journal_mode = WAL` (or the platform default) and `synchronous = FULL` are chosen for durability over speed;
  the load is one row per file.

## 3. Locks

| Lock | Scope | Taken by | Released |
| --- | --- | --- | --- |
| global run lock | one `run --once` at a time per `AUTO_TZIM_HOME` | `run`, and later the scheduler | on exit, including on signal |
| folder guard | one folder at a time inside a run | the runner | when the folder's run ends |
| per-file guard | a file may not be in two runs | the runner, keyed on `(folder_id, relative_path)` | when the file's row commits |

Implementation notes that matter for correctness:

- the lock is an OS-level lock file (`flock`/`LockFileEx`) with the pid and start time recorded, so a stale lock from
  a crashed process is detectable rather than permanent;
- `run --once` on a locked installation exits `4` and processes nothing;
- the lock is taken **before** discovery, not after, so two concurrent runs cannot both build a plan;
- a lock is never "stolen" automatically. If it is stale, the program says so and tells the user which process id to
  check; automatic stealing is how two runs end up on one file.

## 4. Retention

| Artefact | Default retention | Configurable |
| --- | --- | --- |
| `files` rows | 90 days | yes, and `history` is paged |
| `runs` rows | 90 days | yes |
| log files | 30 days and a size cap (documented per platform) | yes |
| update-check cache | 24 hours | no need |

Retention deletes records, never user files. Deleting history is a user action (`history prune`, later stage); the MVP
applies retention on write with a documented limit.

## 5. Schema migrations

`schema = 1` in the config and `meta.schema = 1` in the database are separate on purpose: the config can evolve
without touching the database and vice versa. A newer database opened by an older program is a hard error (`2`), not a
best-effort read: silently misreading a history table is worse than refusing to start. A migration that cannot be
completed leaves the original file untouched and reports what was attempted.

## 6. What the state does not contain

- file contents, thumbnails, or hashes above the documented size cap (D-09 records the omission explicitly
  rather than pretending);
- environment variables, shell history, or anything read from the network;
- any path outside the configured roots;
- credentials of any kind.

## 7. Inspecting and exporting

`status` and `history` are the supported ways to read it; both have `--json`. Direct SQLite access is allowed but unsupported: the schema is versioned, and a query written against `schema = 1` may not survive `schema = 2`. An
`export` command (JSONL of runs and files) is a candidate for a later stage, listed here so that nobody adds a second
half-export in `history`.
