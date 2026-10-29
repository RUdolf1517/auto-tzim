# Architecture

> **Status: specification, no implementation.** `auto-tzim` v0.1 has not been started; no code exists in this
> repository. This document is the contracted shape a reviewer will check against the first pull request.
> Related: [cli](cli.md) · [config](config.md) · [safety](safety.md) · [state](state.md)

## 1. What this program is, and is not

`auto-tzim` is an orchestrator. It decides **which files may be handed to `tzim`**, photographs their identity before
and after, and records what happened. It contains:

- no codec, no parser for any file format, no repacker;
- no verification of any candidate;
- no replacement of any user file, not even "just this once" for a special case.

Those four things happen inside `tzim`, which is invoked as an external process with an argv array. If `tzim`
refuses a file, the refusal is recorded and the file is finished with; there is no fallback handler, no second
attempt through another path, and no "force" flag anywhere in the code base.

The consequence is a hard boundary worth stating in one line: **`auto-tzim` can only ever prevent work; it can never
do work `tzim` would not have done.**

The same boundary fixes **when** the binary may be called at all: outside an apply, the only invocation is
`tzim --version`, for identity. A scan never passes a path, never passes `--dry-run`, and never writes in a target root
(charter §6-2).

## 2. Modules

The layout is fixed by `SPEC.md` §4.2 and is a review item, not a suggestion:

```text
auto_tzim/
  core/
    config          # read/validate TOML, atomic write, folder CRUD
    capabilities    # version adapter for tzim: what it may do, what it emits
    discovery       # walk one approved root, filter, produce candidates
    stability       # identity capture, age and change checks
    planner         # deterministic plan + exact argv per file
    runner          # subprocess, NDJSON parse, timeout, signals
    policy          # formats, transforms, limits, refusals decided by us
    state           # SQLite: runs, files, locks, history
    scheduler       # run-once now; OS adapters from v0.2
    updater         # update check now; controlled update from v0.3
    logging         # structured JSONL + retention
  cli/              # argument parsing, human and JSON rendering, exit codes
```

`tui/` and `qt/` do not exist in v0.1. When they exist they call the same `core` API — the review gate for v0.2 and
v0.4 is that no discovery, subprocess or updater code appears outside `core/` (charter invariant 17).

## 3. Data flow of one run

```text
config ──► folders ──► discovery ──► stability ──► policy ──► plan ──► apply ──► runner ──► state
                          │             │            │          │        │        │        │
                          │             │            │          │        │        │        └─ history, counts, locks
                          │             │            │          │        │        └─ one tzim process per file, NDJSON
                          │             │            │          │        └─ plan id + hash; mismatch ⇒ exit 6
                          │             │            │          └─ ordered candidates + argv, hashed into the plan
                          │             │            └─ formats / transforms / limits / opt-in gates
                          │             └─ (dev, ino, size, mtime_ns), age, no-follow, nlink
                          └─ lstat walk, hidden files, mounts, excludes, depth limit
```

`scan` stops at **plan** and writes nothing in the target root; `apply` (`run --once`) is the only path that reaches
`runner`, and only for a plan whose hash still matches the tree (D-20).

Properties that follow from this shape and are individually tested:

- **Deterministic.** Discovery order is the sorted relative path; the plan is a list, not a set. Two runs on an
  unchanged tree produce the same plan, including argv (AC-1).
- **Sequential in v0.1.** One file at a time, one process at a time, `jobs = 1`. Parallelism inside a file belongs to
  `tzim`; parallel files are a later, separately reviewed change.
- **Re-checked before the spawn.** The identity captured at discovery is compared again immediately before `tzim` is
  started; a difference is a skip, never a "probably fine" (invariant 6, R-03).
- **Recorded after the process.** The file's identity is read once more after `tzim` exits, so a file that changed
  between the spawn and the exit is visible even if `tzim` reported success (the runner trusts the exit code and the
  JSON, never the absence of an error).

## 4. The boundary with `tzim`, in six rules

1. **Pass files, not directories.** Never `-r`, never a directory argument: our plan is the only input (D-12).
   Outside an apply the only invocation is `tzim --version`: no path, no `--dry-run`, no flag that could read a file
   (charter §6-2).
2. **No shell.** `subprocess.run(argv, shell=False)` with a minimal environment, `stdin=DEVNULL`, captured stdout and
   stderr with size limits.
3. **One file per process, no PTY.** Simplifies failure attribution, cancellation and timeouts; costs process startup
   per file, which is irrelevant next to compression. `stdin=DEVNULL`, no `pty` module, no TTY assumption (D-21).
4. **Parse only the documented JSON.** Keys we need are validated; unknown optional keys are ignored and recorded;
   malformed output is a failure, never a success (study G-4). Prose (`--info`, `--list-formats`, `note`) is never parsed
   (charter §6-18).
5. **Never translate a child's exit code into ours.** The mapping table of D-02 is the only place the two vocabularies
   meet, and our own contract (D-18/D-19) is fixed: `0/1/2/3/5/6/7/8/130/143`, with `4` deliberately unused.
6. **Never repair `tzim`.** Missing interfaces become issues there, not heuristics here.

## 5. Where the safety properties live

| Property | Enforced in | Proof |
| --- | --- | --- |
| Only inside the canonical approved root | `config` (validation) + `discovery` (containment) + `runner` (re-check before spawn) | INV-7 |
| No symlinks, no special files | `discovery` (lstat, mode bits) | INV-5 |
| Hard links and unstable files skipped | `stability` | INV-6, R-04 |
| First apply of a folder needs a matching read-only plan; scan never runs file-processing `tzim` | `planner` cannot reach `runner`; `state` stores the plan id/hash; `cli` refuses with exit `6` | INV-2, AC-2, AC-11, AC-12 |
| No prompts, no PTY | `runner` (`stdin=DEVNULL`, no `pty` import) + no prompt code + timeout | INV-12, AC-5 |
| No re-routing of a refused file | absence of any fallback branch, reviewed | INV-10 |
| No automatic retry of `failed` | `state` (retry policy keyed on the invalidation key) | INV-13 |
| No network except `update-check` | `updater` is the only module that imports a network client | INV-15 |
| Nothing written outside `AUTO_TZIM_HOME` | every write site enumerated in review; `tzim` does the rest | INV-8 |

## 6. Extension points reserved now, implemented later

- `scheduler`: `run --once` is the only entry point the scheduler will call in v0.2. No scheduler-specific
  eligibility code will exist, so the scheduler cannot diverge from the CLI.
- `updater`: v0.1 implements "notify" only. A controlled update (v0.3) must reuse the same version comparison and the
  same lock, and must refuse while a run is active.
- `tui`/`qt`: read from the same state API and start the same `run --once`. Any duplicate logic is a review failure.

## 7. What the architecture deliberately refuses

- A daemon, a service, or anything that keeps running after the command returns (invariant 3).
- A second configuration format, environment-variable overrides of policy, or hidden defaults that tighten
  `tzim`'s own limits (D-11).
- Storing file contents: only identity, hashes within the documented cap, statuses, and the argv that was used.
- An "undo" store. A successful optimization is a new file on disk; pretending otherwise is forbidden by invariant 16.
