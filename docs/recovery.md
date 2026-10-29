# Recovery

> **Status: specification, no implementation.**
> Related: [state](state.md) · [safety](safety.md) · [cli](cli.md)

This page is for the moment after something went wrong. It is written as a procedure, because the wrong instinct —
delete the database, force the lock, run it again — destroys the evidence that would explain what happened.

## 1. A run was interrupted (Ctrl-C, crash, power loss)

What you will see: `status` reports the last run as `interrupted`; history has rows up to the point of interruption,
the in-flight file is `interrupted`, and later files in the plan are `interrupted` too.

What to do: nothing urgent. Files that were already optimized keep their verified changes — `tzim` replaced them
atomically, and each row records before/after sizes and, within the documented cap, hashes. Run `status` and
`history --limit 20` to see where it stopped, then start a new run when convenient.

What **not** to do: delete the SQLite database "to clean up". It is the only record of what happened, and the next run
will re-plan every file from scratch, which is not dangerous but is slow and loses the explanation.

## 2. A stale lock

What you will see: `run --once` exits `4` and names a pid and a start time.

What to do: check whether that process exists (`ps -p <pid>`). If it does, a run is active — wait, or interrupt it. If
it does not, the lock is stale: `auto-tzim` reports that state and offers no automatic takeover, because two runs on
one folder is the failure mode this lock exists to prevent. Re-running after the process is confirmed gone clears it;
if the lock file itself is unreadable, the diagnostic says which file.

What **not** to do: delete the lock file while a run is active. Two processes writing history for the same file is
exactly the race the record exists to make impossible.

## 3. A file changed under a run

What you will see: a `skipped_unstable` row (the change was seen before the spawn) or a `failed` row from `tzim` with
a concurrent-modification reason (the change happened while `tzim` worked).

What it means: the file was being written by another program — an editor autosaving, a sync client, a build. Nothing
was lost: `tzim` refuses rather than races. The file becomes eligible again when its identity changes and it stays
still for `min_file_age_seconds`.

What to do: if a sync client is the cause, exclude the folder or set `allow_cloud_sync` deliberately; if it is a
build, exclude the output directory.

## 4. An orphaned `.tzim-*.tmp` file

What you will see: a diagnostic naming `.tzim-<pid>-<nanoseconds>-<attempt>.tmp`; it is never planned for processing.

What it means: a `tzim` process died hard (kill -9, panic, power loss) while it held a candidate. `auto-tzim` does not
delete it — it is not our file, and deleting unknown files is the behaviour this whole project exists to avoid.

What to do: once you are sure no `tzim` process is running and no `auto-tzim` run is active, you may delete it
yourself. If it keeps happening, note the pid and the size: it usually means either a crash in `tzim` or a full disk.

## 5. Corrupt config

What you will see: every command exits `2` with the file and the parse error.

What to do: open the file, fix the syntax. The program never rewrites a config it could not parse, so the file is
exactly as you left it. If you cannot fix it, move it aside and re-add folders one at a time; history is in the
database and is not affected by a config rewrite.

## 6. Corrupt state database

What you will see: `doctor` reports the database as not OK, or a command exits `2`/`1` while opening it.

What to do: copy the database aside first — it is the only record of past runs. `auto-tzim` will refuse to start
rather than migrate or truncate silently. Recovery options are: restore from a backup, or start a fresh state
directory and accept that history is lost while config and folders are unaffected. A user file is never at risk in
either case: the state directory contains no user files.

## 7. Disk full during a run

What you will see: `tzim` reports `failed` with an I/O reason for the file in flight; the run continues with the next
file, which will probably fail too.

What to do: free space and re-run. The `min_free_bytes` floor exists to refuse the run before it starts when the
volume is already low; it is a floor, not a forecast, and a dry run also needs candidate space
([P-02](manager/problems/P-02-dry-run-is-not-write-free.md)).

## 8. `tzim` was upgraded and the adapter refuses it

What you will see: exit `3` and a message naming the version and the supported range.

What it means: `auto-tzim` will not drive a binary whose behaviour it has not been reviewed against. That is the
fail-closed rule, not a bug.

What to do: upgrade `auto-tzim` to a version whose adapter covers the new `tzim`, or point `tzim_path` at the previous
binary until that exists. Do not edit the adapter's version table by hand to "make it work": the table is the reviewed
statement of what has been tested.

## 9. What recovery never does

- Never re-runs a `failed` file automatically.
- Never marks an interrupted file as `optimized`.
- Never deletes a user file, a temporary file it did not create, or a partial record.
- Never overwrites a config or database it could not read.
