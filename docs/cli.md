# Command-line interface

> **Status: specification, no implementation.** The commands come from `SPEC.md` §8, the exit-code contract from charter
> v0.2 §16 P-03, and the JSON field names from decision D-16. A coder implements this page as written or brings a
> decision — fields and codes are not invented in code.
> Related: [config](config.md) · [safety](safety.md) · [state](state.md) · [conflicts between the two specifications](spec/CONFLICTS.md)

```text
auto-tzim [OPTIONS] <COMMAND> ...
auto-tzim --help
auto-tzim --version
```

`auto-tzim` is a subcommand tool, unlike `tzim`: an orchestrator has several verbs, and a path is an argument to one of
them. Everything that decides what happens to a file lives in the config or in an explicit flag.

## 1. Global options

| Option | Default | Meaning |
| --- | --- | --- |
| `--json` | off | typed NDJSON on stdout (§3); implies non-interactive behaviour and no progress line |
| `-q, --quiet` | off | suppress human output; errors still go to stderr |
| `--no-color` / `--color <WHEN>` | `auto` | `auto` honours `NO_COLOR`, `CLICOLOR`, `CLICOLOR_FORCE`, `TERM=dumb` and the terminal test; `--json` is never coloured |
| `--no-progress` | off | never draw the progress line |
| `-h, --help`, `-V, --version` | — | stdout, exit `0` |

There is no `--config PATH`: the location is fixed ([config](config.md) §1) and relocated only by `AUTO_TZIM_HOME`, so
two invocations cannot disagree about which file they describe.

**No command ever prompts, and no PTY is allocated.** `stdin` is never read; a child process gets `stdin=DEVNULL`
(D-21). A command that needs a decision requires a flag. `--json`, a pipe and a scheduler behave exactly like a terminal,
minus the progress line.

## 2. Commands

### 2.1 `doctor`

Prints what a bug report needs: configured `tzim` path, `realpath`, size, SHA-256, `tzim --version`, the adapter entry,
the platform, the config location and schema, and the state database status. It runs **one** command against the binary —
`tzim --version` — and never a file-processing invocation. Exit `0` consistent, `3` `tzim` missing/unreadable/outside the
adapter contract, `2` invalid config, `7` unreadable state.

### 2.2 `config validate`

Parses and validates without touching state. Exit `0` valid, `2` invalid with **every** error listed (not just the
first), each naming the field, the rule and a suggested correction.

### 2.3 `folder add PATH [--name NAME] [--recursive] [--formats LIST] [--exclude GLOB]...`

Validates and canonicalises `PATH`, creates a profile, processes nothing, prints the resulting profile including the
canonical root. Exit `2` for a forbidden, relative, missing or symlinked root, a duplicate root, a path inside
`AUTO_TZIM_HOME`, or a non-empty `transforms` requirement it cannot satisfy.

### 2.4 `folder list`, `folder show ID`, `folder edit ID ...`, `folder enable ID`, `folder disable ID`, `folder remove ID`

`show` prints the profile plus its last run and counts. `edit` validates the whole result before writing, so a rejected
edit leaves the file byte-identical. `remove` deletes the profile and its cached eligibility records — never a user file
— and prints what was removed.

### 2.5 `scan [--folder ID]` — the read-only candidate plan

Builds the plan and **starts no file-processing `tzim`**. Specifically:

- runs discovery, stability and policy, and calls `tzim --version` once per run;
- never passes a target path to `tzim`, never passes `--dry-run`, never passes `--formats` when `formats = []`;
- writes nothing inside the target root and does not need write access there;
- emits one `planned` row per candidate with path, size, identity, the policy decision and the argv a future apply would
  use;
- does **not** claim eligibility, predicted savings, or "would optimize" — those numbers exist only after `tzim` has run
  (D-23, charter §8.4).

Each scan stores a plan record with an id and a hash over its ordered rows (D-20). That record is what authorises an
apply.

### 2.6 `run --once [--plan ID] [--folder ID] [--dry-run]`

The only processing entry point in v0.1. It requires a **matching plan**: same folder, complete, and the same plan hash as
the tree it is about to process. Without one it exits `6` and does nothing (§4, D-03, D-20). `--dry-run` is an alias for
`scan`: it produces the same read-only plan and processes nothing (D-04).

An apply invokes `tzim` once per planned file with an argv array, `stdin=DEVNULL`, no PTY, and a per-file timeout. Whether
`tzim` changes a file is entirely its decision; our job is to record the verdict and to prove that every non-candidate
file is byte-identical afterwards.

### 2.7 `status`

Paused state, whether a run is active (and since when), the last run per folder, the counts of that run, and whether a
plan is pending (id, age, number of rows).

### 2.8 `history [--folder ID] [--status STATUS] [--limit N]`

Recorded outcomes, newest first, `--limit` default 50. A row shows the relative path, the normalized status, sizes when
known, the `tzim` version and binary hash, and — for `skipped_by_tzim` and `failed` — the raw `tzim` status, `note` and
`error` verbatim. An unknown `--status` value is exit `2`.

### 2.9 `logs [--folder ID] [--limit N]`

The structured local log, rendered for humans; with `--json`, the raw JSONL objects.

### 2.10 `pause`, `resume`

`pause` sets the flag that makes `run --once` exit `6`. It does not abort an active run — stopping one means sending
`SIGINT`, which the run records as `interrupted`.

### 2.11 `update-check`

One documented request ([update](update.md)); prints current and available versions or the incompatible-major state.
Never downloads, never replaces anything. Exit `5` on failure.

## 3. `--json` output

Typed NDJSON, one value per line, on stdout, nothing else, `"schema": 1` in every object (D-16).

| Command | Lines |
| --- | --- |
| `doctor`, `config validate`, `status`, `pause`, `resume`, `update-check`, `folder *` | one `{"type":"result", …}` |
| `scan`, `run --dry-run` | one `{"type":"file", …}` per candidate, then one `{"type":"plan", …}` |
| `run --once` (apply) | one `{"type":"file", …}` per processed file, then one `{"type":"run", …}` |
| `history` | one `{"type":"history","rows":[…]}` |
| `logs` | one `{"type":"log","lines":[…]}` |

```json
{"schema":1,"type":"result","command":"doctor","tzim":{"path":"/usr/local/bin/tzim","realpath":"/usr/local/bin/tzim","sha256":"…","size":1048576,"version":"0.14.0","adapter":"tzim-0.14","version_status":"supported"},"platform":{"os":"linux","arch":"x86_64","python":"3.12.7"},"config":{"path":"…","schema":1,"folders":1},"state":{"path":"…","ok":true}}

{"schema":1,"type":"file","folder":"01J…","plan_id":"…","path":"/Users/example/Pictures/project/a.png","relative_path":"a.png","status":"planned","policy":{"formats":"default","transforms":[],"reason":null},"identity":{"dev":1,"ino":42,"size":1258291,"mtime_ns":1690000000000000000},"argv":["tzim","--json","/Users/example/Pictures/project/a.png"]}

{"schema":1,"type":"plan","plan_id":"…","plan_hash":"sha256:…","folder":"01J…","created":"2026-10-29T19:00:00Z","candidates":9,"excluded":4,"tzim":{"version":"0.14.0","sha256":"…"}}

{"schema":1,"type":"run","run_id":"…","plan_id":"…","started":"2026-10-29T19:00:00Z","finished":"2026-10-29T19:02:11Z","counts":{"planned":9,"optimized":7,"already_optimal":1,"skipped_unsupported":1,"skipped_by_tzim":0,"skipped_policy":0,"skipped_unstable":0,"hardlink_skip":0,"failed":0,"interrupted":0},"binary":{"version":"0.14.0","sha256":"…"},"exit_code":0}
```

Rules:

- `null` means "not computed or not applicable", never "unknown but probably fine";
- `failed` rows carry `tzim_error` with `{"kind","message"}` when `tzim` supplied one;
- `skipped_by_tzim` rows carry `tzim_note` verbatim — it is data for a human, never a classification key;
- diagnostics (warnings, whole-command refusals) go to stderr as human text, so `--json > out.ndjson` always parses;
- `argv` is printed in the plan (it is what the user is being asked to approve) and stored in history for the apply.

## 4. Exit codes

| Code | Meaning | Typical source |
| --- | --- | --- |
| `0` | the command did what it says; documented skips included | any successful command |
| `1` | at least one file ended `failed` | apply |
| `2` | usage or configuration error; nothing processed | bad flag, invalid config, unknown `--status` |
| `3` | `tzim` absent, unreadable, or outside the adapter contract (`incompatible_tzim`) | `doctor`, `scan`, `run` |
| `4` | **never produced by `auto-tzim`.** Reserved out of every path so a `tzim` verification failure can never be confused with our lock — the lock is `8` | — |
| `5` | `update-check` failure (network, rate limit, malformed answer) | `update-check` |
| `6` | refused before processing: no matching plan (first-run gate), `pause`, `min_free_bytes`, disabled folder, whole-run policy refusal | `run --once` |
| `7` | internal error: unreadable/corrupt state, unexpected exception, child output our guard cannot attribute to a file | any |
| `8` | another run holds the lock, or the lock cannot be taken | `run --once` |
| `130` / `143` | interrupted by `SIGINT` / `SIGTERM`; in-flight file and remaining plan recorded `interrupted` | `run --once` |

**Precedence**, highest first (charter §16 P-03):

```text
130/143  >  7  >  2  >  8  >  3  >  6  >  5  >  1  >  0
```

**Fixed preflight order** of a run — each step may return before the next:

1. parse arguments → `2`; 2. load config → `2`; 3. take the lock → `8`; 4. read the binary identity and version → `3`;
5. `pause` → `6`; 6. folder enabled → `6`; 7. matching plan → `6`; 8. free space → `6`; 9. discovery, then per-file work.

The raw `tzim` exit code is stored in history and is **never** our exit code (charter invariant 19). A `tzim` exit `4`
(verification failure) becomes `failed` with `kind=verify`, aborts the plan (remaining files `interrupted`, critical
diagnostic), and the command exits `1`.

## 5. Human output

```text
$ auto-tzim scan --folder 01J…
Project images   /Users/example/Pictures/project
candidates 9   excluded 4 (2 hidden, 1 hardlink, 1 outside policy)
a.png            plan  1.20 MiB  optimize
b.png            skip  hardlink (2 links)
notes.txt        skip  unsupported format        ← known only after an apply, not at scan time
```

- scan lines describe **policy decisions and file facts only**; whether a file will shrink is unknown until an apply;
- one row per file, summary last; `--quiet` prints only the summary;
- colour marks the outcome only (green rewrite, dim nothing to do, yellow skip, red failure), plain through a pipe,
  never in `--json`;
- the progress line goes to stderr, only on a terminal, never with `--quiet`/`--json`/`--no-progress`.

## 6. Conformance notes

Deliberate choices, recorded so they are not "fixed" later:

- **No `--force`** and no "ignore errors": a refusal is information; the way to change eligibility is the folder policy.
- **No interactive anything**: no `--yes`, no confirmation prompt, no pager, no PTY.
- **No `--jobs` in v0.1**: it exists in the config for v0.2 and is rejected today.
- **`-` is not stdin**: the program acts on files it must be able to re-read.
- **No glob patterns on the command line**: `exclude_globs` in the config is the only pattern language.
- **`run --dry-run` is an alias of `scan`**, not a `tzim` dry run — see [CONFLICTS §C-1](spec/CONFLICTS.md).
