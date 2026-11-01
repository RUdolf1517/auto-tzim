# Recovery

> **Status: specification, no implementation.**
> Related: [state](state.md) · [safety](safety.md) · [cli](cli.md)

This page is for the moment after something went wrong. It is written as a procedure, because the wrong instinct —
delete the state files, force the lock, run it again — destroys the evidence that would explain what happened.

## 1. A run was interrupted (Ctrl-C, crash, power loss)

What you will see: `status` reports the last run as `interrupted`; history has rows up to the point of interruption,
the in-flight file is `interrupted`, and later files in the plan are `interrupted` too.

What to do: nothing urgent. Files that were already optimized keep their verified changes — `tzim` replaced them
atomically, and each row records before/after sizes and, within the documented cap, hashes. Run `status` and
`history --limit 20` to see where it stopped, then start a new run when convenient.

What **not** to do: delete the state directory "to clean up". It is the only record of what happened, and the next run
will re-plan every file from scratch, which is not dangerous but is slow and loses the explanation.

## 1a. A `.auto-tzim-old-…` file appeared next to your file

**What you will see:** a file named `photo.png` and, beside it, `photo.png.auto-tzim-old-3f2a91-07`. Nothing is broken and
nothing is half-written: that name is the crash-safe half of an install (D-29).

**What happened:** replacing a file is three steps — move the original aside, move the verified replacement into place,
delete the one moved aside. A crash between any two leaves a complete, readable file under a temporary name. That is the
design, not a leak: at every instant at least one full copy exists that can be restored.

**What to do:** run `auto-tzim status`. The next start finds the journal and finishes the job — either restoring the
original (the replacement never landed) or removing the leftover (the replacement did land). It prints one line per file
it resolved, so you know which happened. Do not delete these files by hand; if the program cannot resolve one, it says so
and leaves it, because guessing is worse than leaving evidence.

**What never happens:** a file is never lost by an interruption. That is the whole reason the swap looks like this
instead of `unlink` + `write`.

## 1b. `confirm` mode refused with `6` and asked for no confirmation

**What you will see:** exit `6`, nothing processed, a message naming `--mode auto`.

**What happened:** the proposal must be read by a person. There was no terminal (a pipe, a cron job, `--json`, or a closed
stdin), so the program refused instead of assuming "yes" — and instead of hanging until a timeout nobody is watching.

**What to do:** run it from a terminal, or use `--mode auto` deliberately. There is no `--yes`, by design: a silenced
prompt and a different mode are not the same thing, and only one of them can be audited later.

## 2. A stale lock

What you will see: `run --once` exits **`8`** and names a pid and a start time. (`4` is never produced by `auto-tzim`:
it belongs to `tzim`, where it means a verification failure, and we keep it out of our own contract so the two can never
be confused.)

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
exactly as you left it. If you cannot fix it, move it aside and re-add folders one at a time; history lives in
`state/runs/` and is not touched by a config rewrite.

## 6. A corrupt state file

There is no database to corrupt; there are JSONL files and small JSON files (D-35), so "corrupt" has exactly three shapes
and each has an answer.

**A truncated last line in a `.jsonl` file.** This is the expected damage from a kill during a write. `history` reads the
file, reports `tail_recovered: true` for that run, and continues. **Nothing to do** — the record was never completed, and
inventing it would be worse than reporting its absence.

**A malformed line in the middle of a `.jsonl` file.** That means the file was edited by hand or damaged on disk. `doctor`
names the file, the line number and the run id; commands that only read other runs keep working, and a command that needs
that run exits `7`. Nothing is rewritten. Copy the file aside, then either delete the bad line or move the run directory
out of `state/runs/` and accept that run's loss — config, folders and other runs are unaffected.

**A `run.json` that will not parse.** The run is unusable as a record, but the files it touched are still described in
`files.jsonl` next to it. Rename the directory out of `state/runs/`; `status` will report the run as unknown rather than
guess.

**What never happens:** `auto-tzim` will not migrate, truncate or "repair" a state file it could not read. A user file is
never at risk in any of these cases, because the state directory contains no user files and no path into a managed root.

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

## 8a. A run refused before it started (`6`)

What you will see: exit **`6`** with a reason: no matching plan ("preview this folder first"), `pause` is active, free
space is below `min_free_bytes`, or the folder is disabled.

What it means: nothing was read for processing and nothing changed. Not an error and not a success — a scheduler must
read it as "not now", never as "done".

What to do: run `auto-tzim scan` and then apply; `resume` after a pause; free space; enable a disabled folder.

## 9. What recovery never does

- Never re-runs a `failed` file automatically.
- Never marks an interrupted file as `optimized`.
- Never deletes a user file, a temporary file it did not create, or a partial record.
- Never overwrites a config or state file it could not read.
