# Command-line interface

> **Status: specification, no implementation.** The commands and exit codes come from `SPEC.md` §8; the JSON field
> names are a **proposed contract (proposal D-16)** that the coder implements as written or challenges with a
> decision — they must not be invented ad hoc in code.
> Related: [config](config.md) · [architecture](architecture.md) · [safety](safety.md) · [state](state.md)

```text
auto-tzim [OPTIONS] <COMMAND> ...
auto-tzim --help
auto-tzim --version
```

`auto-tzim` is a subcommand tool, unlike `tzim`: `tzim`'s verb is a path, but an orchestrator has several verbs and a
path is one of their arguments. Everything that decides what happens to a file lives in the config or in an explicit
flag; nothing is inferred from the environment.

## 1. Global options

| Option | Default | Meaning |
| --- | --- | --- |
| `--json` | off | newline-delimited JSON on stdout (§3); implies `--no-progress` and non-interactive behaviour |
| `-q, --quiet` | off | suppress human output; errors still go to stderr |
| `--no-color` / `--color <WHEN>` | `auto` | `auto` honours `NO_COLOR`, `CLICOLOR`, `CLICOLOR_FORCE`, `TERM=dumb` and the terminal test; `--json` is never coloured |
| `--no-progress` | off | never draw the progress line |
| `-h, --help`, `-V, --version` | — | stdout, exit `0` |

There is no `--config PATH`: the location is fixed (§ [config](config.md) §1) and relocated only by `AUTO_TZIM_HOME`,
so that two invocations cannot disagree about which file they describe.

**No command ever prompts.** `stdin` is never read; a command that needs a decision requires a flag. In a
non-interactive session (`--json`, a pipe, a scheduler) the behaviour is identical to a terminal, minus progress.

## 2. Commands

### 2.1 `doctor`

Checks the installation and prints the facts a bug report needs: the configured `tzim` path, its `realpath`, size and
SHA-256, `tzim --version`, the JSON `schema` observed, the adapter entry in use, whether the optional capability probe
succeeded, the platform, the config location and schema, and the state database status. Exit `0` when consistent,
`3` when `tzim` is missing or incompatible, `2` when the config is invalid.

### 2.2 `config validate`

Parses and validates the config without touching state. Exit `0` valid, `2` invalid with every error listed (not just
the first), each naming the field, the rule and a suggested correction.

### 2.3 `folder add PATH [--name NAME] [--recursive] [--formats LIST] [--exclude GLOB]...`

Validates and canonicalises `PATH`, creates a profile, and **processes nothing**. Prints the resulting profile,
including the canonical root, so the user can see what was actually accepted. Exit `2` for a forbidden/relative/
missing root, a duplicate root, or a path inside `AUTO_TZIM_HOME`.

### 2.4 `folder list`, `folder show ID`, `folder edit ID ...`, `folder enable ID`, `folder disable ID`, `folder remove ID`

`show` prints the profile plus its last run and counts. `edit` validates the whole resulting profile before writing,
so a rejected edit leaves the file byte-identical. `remove` deletes the profile and its cached eligibility records,
never a user file, and prints what was removed.

### 2.5 `scan [--folder ID]`

Builds the plan without starting `tzim`: which files, which status, which argv, which skip reason. This is the
preview the first-run gate accepts (§4). Exit `0`, or `2` for a bad folder id.

### 2.6 `run --once [--folder ID] [--dry-run]`

The only processing entry point in v0.1. `--dry-run` additionally asks `tzim -n --json` for exact predicted numbers
(proposal D-04, unapproved — see [P-02](manager/problems/P-02-dry-run-is-not-write-free.md)). The first run of a
folder requires a preceding `scan` or a `--dry-run` in the same invocation; otherwise the command refuses and changes
nothing.

### 2.7 `status`

Paused state, whether a run is active (and since when), the last run per folder, and the counts of the last run.

### 2.8 `history [--folder ID] [--status STATUS] [--limit N]`

Recorded outcomes, newest first, `--limit` defaulting to 50. A row shows the relative path, the normalized status, the
size before/after when known, the `tzim` version and binary hash, and the raw `tzim` status/note when one was
received.

### 2.9 `logs [--folder ID] [--limit N]`

The structured log, rendered for humans; with `--json` the raw JSONL objects. Logs are local only.

### 2.10 `pause`, `resume`

Set a global flag that makes `run --once` refuse. `pause` does not abort an active run — stopping one is `SIGINT` to
the process, which the run records as `interrupted`.

### 2.11 `update-check`

One documented request (§ [update](update.md)); prints current and available versions, or the incompatible-major
state. Never downloads, never replaces anything. Exit `5` on network or parse failure.

## 3. `--json` output

One JSON value per line, on stdout, nothing else. Diagnostics (warnings, refusals of a whole command) go to stderr as
human text, so `auto-tzim … --json > out.ndjson` always yields parseable data.

| Command | Lines |
| --- | --- |
| `doctor`, `config validate`, `status`, `pause`, `resume`, `update-check`, `folder *` | exactly one `{"type":"result", …}` |
| `scan` | one `{"type":"file", …}` per planned file, then one `{"type":"plan", …}` |
| `run --once` | one `{"type":"file", …}` per processed file, then one `{"type":"run", …}` |
| `history` | one `{"type":"history", "rows":[…]}` |
| `logs` | one `{"type":"log", "lines":[…]}` |

Every object carries `"schema": 1`. Field sketches (proposal D-16):

```json
{"schema":1,"type":"result","command":"doctor","tzim":{"path":"/usr/local/bin/tzim","realpath":"/usr/local/bin/tzim","sha256":"…","size":1048576,"version":"0.14.0","json_schema":1,"adapter":"tzim-0.14","probe":"ok"},"platform":{"os":"linux","arch":"x86_64","python":"3.12.7"},"config":{"path":"…","schema":1,"folders":1},"state":{"path":"…","ok":true}}

{"schema":1,"type":"file","folder":"01J…","path":"/Users/example/Pictures/project/a.png","relative_path":"a.png","status":"optimized","before":1258291,"after":1065000,"saved":193291,"tzim_status":"optimized","tzim_note":null,"tzim_error":null,"identity":{"dev":1,"ino":42,"size_before":1258291,"mtime_ns_before":1690000000000000000,"size_after":1065000,"mtime_ns_after":1690000000123400000},"dry_run":false}

{"schema":1,"type":"run","run_id":"…","dry_run":false,"started":"2026-10-29T19:00:00Z","finished":"2026-10-29T19:02:11Z","counts":{"planned":12,"optimized":9,"already_optimal":3,"skipped_refused":0,"skipped_unsupported":0,"skipped_policy":0,"skipped_unstable":0,"hardlink_skip":0,"failed":0,"interrupted":0},"binary":{"path":"…","sha256":"…","version":"0.14.0"},"exit_code":0}
```

Rules: `null` means "not computed or not applicable", never "unknown but probably fine"; every `failed` row carries
`tzim_error` with `{"kind","message"}` when `tzim` supplied one; `argv` is recorded in history, not on stdout, because
it contains the absolute path (still local data, still not exported).

## 4. Exit codes

| Code | Meaning |
| --- | --- |
| `0` | the command did what it says; skips, refusals and `already_optimal` included |
| `1` | at least one file ended `failed` |
| `2` | usage or configuration error; nothing was processed |
| `3` | `tzim` missing, unreadable, or outside the adapter's supported versions |
| `4` | another run holds the lock, or the lock cannot be taken |
| `5` | `update-check` (or a later update) failed |
| `6` | **proposed, not adopted**: the run was refused before processing (first-run gate, `pause`, `min_free_bytes`) — see [P-03](manager/problems/P-03-exit-codes.md) and decision D-03 |
| `130` | interrupted by `SIGINT`; in-flight file and remaining plan recorded `interrupted` |

Precedence when several apply is **not yet fixed** — the `tzim`-style ordered rule (`usage → lock → compatibility →
failure → success`) is the shape proposed in P-03 and needs the same owner decision as `6`. Until then a coder must
not invent one; the tests that assert combinations are blocked with the code.

`auto-tzim` never forwards a child's exit code as its own. A `tzim` exit `4` (verification failure) is recorded as a
`failed` row with `kind=verify`, aborts the plan (the remaining files become `interrupted`), and the command exits `1`.

## 5. Human output

```
auto-tzim scan --folder 01J…

Project images   /Users/example/Pictures/project
planned  9   already optimal  2   unsupported  1   skipped  3
a.png            plan  optimize
b.png            skip  hardlink (2 links)
notes.txt        skip  unsupported format
```

- one line per file, then a summary line last;
- `--quiet` prints only the summary;
- colour marks the outcome only (green rewrite, dim nothing-to-do, yellow skip, red failure), plain text through a
  pipe, never in `--json`;
- the progress line goes to stderr and appears only on a terminal, only without `--quiet`/`--json`/`--no-progress`;
- a file name is printed at most once per line, so a very deep tree cannot turn the report into noise.

## 6. Conformance notes

Deliberate choices, recorded so they are not "fixed" later:

- **No `--force`.** A refusal is information; the flag that changes eligibility is the folder's policy, edited
  deliberately.
- **No interactive anything.** Not `--yes`, not a confirmation prompt, not a pager. Dangerous changes are config
  fields with defaults that are safe (§ [config](config.md) §4).
- **No `--jobs` in v0.1.** It exists in the config for v0.2 and is rejected today.
- **`-` is not stdin.** The program acts on files it must be able to re-read afterwards.
- **No glob patterns on the command line.** Shell globbing exists; `exclude_globs` in the config is the only pattern
  language, and it is a small one.
