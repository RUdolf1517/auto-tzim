# State, history and locks

> **Status: specification, no implementation.** The layout below is a plan; the first implementation must match it or
> bring a decision.
> Related: [architecture](architecture.md) · [cli](cli.md) · [recovery](recovery.md) · [privacy](privacy.md)

Everything `auto-tzim` remembers lives in **plain files under `AUTO_TZIM_HOME`**: no database engine, no C dependency, no
format that needs a library to read. A user can inspect every byte with `cat`, and a corrupt file can be diagnosed by eye
instead of by tooling ([D-35](manager/DECISIONS.md)).

The trade is deliberate: we give up SQL queries and transactions, and we take on three rules that replace them — **one
writer at a time, append-only history, atomic replacement of every small file.** Those three rules are testable; a
database's guarantees are assumed.

## 1. Layout

```text
$AUTO_TZIM_HOME/
  config.toml                    # 0600, atomic replacement
  state/
    schema.json                  # {"schema":1,"created":"…"} written once
    runs/
      <run_id>/
        run.json                 # header at start, finalised at end (atomic replace)
        files.jsonl              # append-only: one row per file outcome
        journal.jsonl            # append-only: the install journal (apply only)
        proposal.txt             # the exact text the user approved (confirm mode)
    plans/
      <plan_id>.json             # immutable once written
      log.jsonl                  # append-only: plan lifecycle (used / superseded)
    eligibility.jsonl            # append-only cache with periodic compaction
    last-compression.json        # atomic replace: canonical root → when it was last compressed
    locks/
      run.lock                   # created exclusively; holds pid + start-time discriminator
  logs/
    auto-tzim.log                # JSONL, rotated by size
```

Every file above is `0600`, every directory `0700`, where the platform supports it.

## 2. The three primitives

Everything else in this document is a consequence of these.

### 2.1 `atomic_write(path, bytes)`

```text
temp = path.with_extension(".tmp-<pid>-<counter>")
create temp (0600) → write → fsync(temp) → rename(temp, path) → fsync(parent directory)
```

A reader sees either the old file or the new one, never a half-written one. Used for `config.toml`, `schema.json`,
`run.json`, `plan.json`, `proposal.txt` and the lock file. **Directory fsync is not optional**: without it the rename can
be lost while the data survives, which is the failure nobody tests for.

### 2.2 `append_line(path, record)`

```text
open(path, append | create, 0600) → write one JSON line → fsync → close
```

Used for `files.jsonl`, `journal.jsonl`, `log.jsonl` and `eligibility.jsonl`. Append-only is not a style choice: it means a
crash can damage **at most the last line**, and a reader can detect that by one rule — a line that is not valid JSON is
treated as an incomplete trailing write, reported in `--json` as `tail_recovered: true`, and never rewritten.

### 2.3 `create_exclusive(path, bytes)`

`OpenOptions::new().create_new(true)` — the kernel guarantees that exactly one process succeeds. Used **only** for the run
lock. The consequence, stated plainly: a lock left by a killed process stays on disk. That is intentional — the
specification forbids automatic stealing (see §5) — so a stale lock is *reported*, with the pid and start time in it, and
a human decides.

## 3. What each file contains

### 3.1 `runs/<run_id>/run.json`

Written at the start with `state: "running"`, replaced atomically at the end with the final record. Fields:

| Field | Notes |
| --- | --- |
| `schema`, `run_id`, `started_at`, `finished_at` | UTC, ISO 8601 with microseconds |
| `state` | `running` \| `completed` \| `aborted` \| `interrupted` |
| `mode` | `confirm` \| `auto` (D-29); `declined` when the proposal was refused |
| `folder`, `plan_id`, `plan_hash` | the binding this run acted on; `folder` is the folder's **name as it was when the run started** ([D-43](manager/DECISIONS.md)) |
| `batch_id` | present when this run was one folder of a multi-path invocation ([D-49](manager/DECISIONS.md)); every record from that command shares it, and a single-path run has none |
| `tzim` | `path`, `sha256`, `version`, `capabilities_schema`, `result_schema`, `adapter` |
| `config_hash` | SHA-256 of the config file as read |
| `counts`, `exit_code` | derived from `files.jsonl` at finalisation, then frozen |

A run that never finished keeps `state: "running"` on disk — that is the signal the next start looks for (§6).

### 3.2 `runs/<run_id>/files.jsonl`

One line per file outcome, in plan order. Fields: `v`, `ts`, `folder`, `relative_path`, `status`, `reason`
(`tzim`'s stable code, D-27), `before`, `after`, `saved`, `identity_before`, `identity_after`, `sha256_before`,
`sha256_after`, `tzim_status`, `tzim_note`, `tzim_error_kind`, `tzim_error_message`, `duration_ms`, `argv`.

Rules:

- **Append-only, never edited.** A re-run is a new run directory; history is the set of directories, not a mutable table.
- `status` uses only the vocabulary of D-02: `planned`, `optimized`, `already_optimal`, `skipped_unsupported`,
  `skipped_by_tzim`, `skipped_policy`, `skipped_unstable`, `skipped_by_user`, `skipped_changed`, `hardlink_skip`,
  `failed`, `interrupted`, `incompatible_tzim`. The raw `tzim` status, note and error live in their own fields and are
  never folded into `status`.
- `argv` is stored because it is the only way to explain a result later. It contains an absolute path, which is why
  history is treated as sensitive ([privacy](privacy.md)).

### 3.3 `runs/<run_id>/journal.jsonl`

Written only by an apply (D-29). One line per install step, so a crash can be replayed:

```json
{"v":1,"ts":"…","file":"a.png","step":"stat","identity":{"dev":1,"ino":42,"size":1258291,"mtime_ns":…}}
{"v":1,"ts":"…","file":"a.png","step":"staged","staged":"runs/<id>/stage/a.png","sha256":"…"}
{"v":1,"ts":"…","file":"a.png","step":"replaced","old":".auto-tzim-old-<run>-07","sha256":"…"}
{"v":1,"ts":"…","file":"a.png","step":"committed"}
```

The `replaced` line is written **before** the rename-away and the `committed` line after the unlink, so a crash between them
is resolvable by re-reading the filesystem — which is what [recovery](recovery.md) §1a does.

### 3.4 `plans/<plan_id>.json` and `plans/log.jsonl`

One file **per folder**, written once and **never modified**; its lifecycle is a row in `log.jsonl` (`used`,
`superseded`). A batch of three folders produces three plan files, three ids and three hashes, each recording its own
canonical root ([D-57](manager/DECISIONS.md)) — never one blob covering several trees.

**Publication is `hard_link`, not `rename`** ([D-60](manager/DECISIONS.md)): the temp file is written, `fs::hard_link(tmp, final)`
creates the final name **only if it is free** (`EEXIST` means the id is taken ⇒ `plan_id_collision`, exit `7`), then the temp is
unlinked and the directory fsynced. `rename` would silently overwrite a plan that already existed, which "written once and never
modified" forbids; §2's "atomic replacement" therefore applies to the **mutable** files only (`run.json`,
`last-compression.json`), and the immutable ones are *published*.

This is what makes D-20's "any difference is exit `6`, never close enough" auditable after the fact: the plan the user
approved is still on disk, byte for byte, next to the hash it was compared against.

### 3.5 `last-compression.json` — the cooldown

One small object, atomically replaced, keyed by **canonical root** so that a rename does not hand a folder a second
compression ([D-47](manager/DECISIONS.md)):

```json
{"schema": 1,
 "roots": {
   "/home/user/downloads": {"at": "2026-11-02T03:00:04.120391Z", "run_id": "01J…", "hours": 24}
 }}
```

- **read** during the preflight, before anything is planned;
- **written** once, right before the first file is staged — a run that crashes still consumed its window, so an hourly
  cron cannot loop on a folder that keeps failing;
- **not written** by `--dry-run` (nothing was compressed) and **not written** by a run refused with `6` (nothing started);
- **`--force`** bypasses the comparison and then rewrites the entry from now.

The file is a cache of a fact that is also derivable from `runs/*/run.json`; if it is missing or unreadable, the tool
treats the folder as never compressed (a run may happen) and logs the fact, because refusing to work is a worse failure
than doing the work once more. `hours` records the window the entry was written under, so lowering `--time` from 24 to 6
takes effect immediately rather than after the old window expires.

### 3.6 `eligibility.jsonl`

The "do not re-examine" cache (D-06), as an append-only log of (identity, `tzim_sha256`, `adapter`, `policy_hash`, verdict,
`decided_at`). A lookup matches on **all** of those; a stale line simply never matches, which is a cheaper and safer
invalidator than deleting anything. When the file exceeds the documented size it is compacted: latest entry per
`(folder, relative_path)` into a temp file, `fsync`, rename. Compaction is the only place that rewrites it, and it
rewrites nothing that a reader could still match.

## 4. Replacing what a database gave us

| Database property | How it is provided here | What we accept |
| --- | --- | --- |
| Transactions | one writer at a time (the run lock) + single-file atomic replacement | no multi-file rollback: a crash can leave a run directory that needs recovery, never a half-written file |
| Isolation | nothing reads a run directory under `running` except recovery | `history` shows a run only after its `run.json` says so |
| Durability | `fsync` on every append and every rename, directory fsync included | slower per row; the load is one row per file, so it does not matter |
| Indexes | files are named by id; `history` scans and pages with `--limit` | a scan over years of history is linear — documented, not hidden |
| Constraints | validation on write + a schema version in every line | a hand-edited file can violate the schema; we report it (`7`) instead of repairing it |

**The single-writer rule is what makes this safe**, and it is why the lock is taken before discovery: two writers would be
the only way to interleave a `files.jsonl` append with a run-directory rewrite. The scheduler (v0.2) inherits the same
lock, so it cannot create a second writer either.

## 5. Locks

| Lock | Scope | Taken by | Released |
| --- | --- | --- | --- |
| global run lock | one run at a time per `AUTO_TZIM_HOME` **including every folder of a batch** ([D-49](manager/DECISIONS.md)) | `run`, and later the scheduler | on exit, including on signal |
| update lock | no run during an update and no update during a run (v0.3) | `update` | on exit |

Two lock files, one mechanism ([D-53](manager/DECISIONS.md)):

| File | Guards |
| --- | --- |
| `locks/config.lock` | the read-modify-write of `config.toml` (AT-02) |
| `locks/run.lock` | one run at a time, whole batch included (AT-05) |

The lock is a **kernel lock** taken with `File::try_lock()` (std, stable since Rust 1.89). The file is created `0600` and
**never deleted** — deleting a lock file another process still holds is the classic race — and its contents are written
after acquisition for diagnostics only:

```json
{"pid": 48213, "started_at": "…", "host": "…"}
```

`started_at` is the process **entry time**, not an OS-reported creation time, and `host` is omitted when `HOSTNAME` is unset —
these fields are diagnostics for the human reading exit `8`, nothing depends on them ([D-54](manager/DECISIONS.md)).
`config validate`, `folder list` and `folder show` take **no** lock: config writes are atomic renames, so a reader always sees a
whole file.

- Contention exits **`8`** and processes nothing; the message names the holder from the file, when it can be read.
- **There are no stale locks.** A holder that dies takes its lock with it, so the old "report it stale, never remove it"
  ritual is retired: it existed only because a lock *file* could outlive its owner. A file left by a dead process is not an
  obstacle and is reused.
- The config lock is taken **before** the config is read, and released after the atomic write. The run lock is taken
  **before** discovery, so two runs cannot both build a plan.

## 6. Crash behaviour

1. A run writes `run.json` with `state: "running"` and takes the lock before the first file.
2. Each file outcome is appended and `fsync`ed immediately, so a crash loses at most the file in flight.
3. Finalisation replaces `run.json` atomically with counts, `finished_at`, `exit_code` and `state`.
4. On the next start, any run directory whose `run.json` says `running` **and** whose lock is stale is resolved: the
   in-flight file is appended as `interrupted`, the run becomes `interrupted`, and the journal is replayed (§1a of
   [recovery](recovery.md)).
5. Recovery never upgrades an outcome: `interrupted` cannot become `optimized`, and a file is never retried
   automatically (D-02, INV-13).

## 7. Retention

| Artefact | Default retention | Configurable |
| --- | --- | --- |
| `runs/<run_id>/` | 90 days | yes |
| `logs/` | 30 days and a size cap | yes |
| `plans/` | 90 days, or until superseded | yes |
| `eligibility.jsonl` | compacted at a documented size, entries expire with the cache rules | yes |

Retention deletes records, never user files, and it runs **after** the lock is taken, never concurrently with a write.

## 8. Schema versioning

`schema = 1` in the config and `"schema": 1` in `state/schema.json` are separate on purpose: the config can evolve without
touching state and vice versa. Every record carries `"v": 1` so a mixed directory is detectable.

- An **older program** meeting a newer `schema.json` refuses with exit `2` and says which version it found. Silently
  misreading history is worse than refusing to start.
- A **newer program** meeting an older one migrates by writing a new file and renaming it, leaving the original until the
  migration has been verified; a failed migration leaves everything as it was and reports what was attempted.
- There is no down-migration. A user who downgrades keeps their old state on disk; the older program refuses to read the
  newer records rather than guessing at them.

## 9. What the state does not contain

- file contents, thumbnails, or hashes above the documented size cap (D-09 records the omission explicitly rather than
  pretending);
- environment variables, shell history, or anything read from the network;
- any path outside the configured roots;
- credentials of any kind.

## 10. Inspecting and exporting

`status` and `history` are the supported ways to read it; both have `--json`. The files are plain JSONL and a user may read
them with anything — `jq -r 'select(.status=="failed") | .relative_path' runs/*/files.jsonl` is a legitimate way to use
this program, and the documentation says so instead of pretending otherwise.

There is no `export` command and no second format: the on-disk format **is** the export format, which is one of the reasons
it is line-oriented JSON rather than a database.

## 11. The limits we accept, written down

1. **Volume.** A folder with hundreds of thousands of files produces a `files.jsonl` that `history` reads linearly. Fine at
   the scale this tool is for; measured before v0.2, and a documented `--limit` is the mitigation until then.
2. **No transactions across files.** A crash leaves a run needing recovery rather than a database needing repair. The
   recovery path is the price, and it is tested (`L-01`…`L-06`).
3. **Hand editing is possible.** Nothing stops a user from editing a line, and we will not refuse to run because of it —
   we report the malformed record and the path. A tool that treats its own files as sacred tends to hide its mistakes.
4. **A C dependency would have bought us indexes and atomicity.** D-35 says no, because a static binary with no C in the
   build is worth more here than a query language nobody asked for.
