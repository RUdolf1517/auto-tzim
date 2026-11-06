# Command-line interface

> **Status: specification, no implementation.** The commands come from `SPEC.md` §8, the exit-code contract from charter
> v0.2 §16 P-03, and the JSON field names from decision D-16. A coder implements this page as written or brings a
> decision — fields and codes are not invented in code.
> Related: [config](config.md) · [safety](safety.md) · [state](state.md) · [conflicts between the two specifications](spec/CONFLICTS.md)

```text
auto-tzim <PATH> [<PATH>…] [OPTIONS]   # the normal use: one or more folders, one run
auto-tzim <COMMAND> [OPTIONS]          # management verbs: folder, config, status, history, logs, pause, resume, doctor
auto-tzim --help
auto-tzim --version
```

**The path comes first, and there may be several.** The ordinary invocation names folders and nothing else:

```sh
auto-tzim /home/user/downloads --time 24 --auto
auto-tzim ~/downloads ~/Pictures/raw ~/tmp/incoming --time 24 --auto
```

That is a complete instruction: *compress this folder at most once every 24 hours, and do not ask me anything.* No config
entry, no subcommand, no second step. `folder add` exists so that a folder can be remembered and repeated, not so that it
can be used at all ([D-46](manager/DECISIONS.md)).

Disambiguation is mechanical, because a path and a verb start the same way: if the first argument is one of the fixed
subcommand words — `folder`, `config`, `status`, `history`, `logs`, `pause`, `resume`, `doctor`, `scan`, `run` — it is a
subcommand; otherwise it is a path, and it must be a directory that exists (anything else is exit `2`).

## 0. The one-shot form

| Invocation | What the user sees | What it does |
| --- | --- | --- |
| `auto-tzim PATH… --dry-run` | the plan of each folder | nothing else: no `tzim` is started on a file, no folder is written, no cooldown is consumed |
| `auto-tzim PATH…` | the plans, then **one** question | installs only on `y`; anything else — including an empty line, EOF or no terminal — installs nothing, anywhere |
| `auto-tzim PATH… --auto` | **nothing** — no plan, no question, one summary line | installs every file the plans selected |
| `auto-tzim PATH… --auto --time 24` | nothing until a summary line | as above, but only for folders whose last compression was **24 hours ago or more**; the others are skipped and named ([D-50](manager/DECISIONS.md)) |

| Option | Default | Meaning |
| --- | --- | --- |
| `--auto` | off | `auto` mode ([D-29](manager/DECISIONS.md), [D-48](manager/DECISIONS.md)): the plan is **not shown** and nothing is asked. Without it the run is `confirm`: the plan is shown and one answer is required |
| `--time <HOURS>` | `0` — every invocation may compress | the **cooldown** ([D-47](manager/DECISIONS.md)): compress this folder at most once in this many hours. `0` disables it. Integer, 0–100 000 |
| `--force` | off | ignore the cooldown for this run; the cooldown is then restarted from now |
| `--dry-run` | off | plan only. This is **our** flag; `tzim --dry-run` stays forbidden (D-04) |
| `--name`, `--recursive`, `--formats`, `--exclude` | the profile's values | per-run overrides; they are used for this run and **not** written back to the config |
| `--plan <ID>` | — | bind this run to an earlier plan record; if the tree no longer matches it, exit `6` before anything is installed |

### 0.2 Several paths are one batch

`auto-tzim a b c --auto` is a **batch** ([D-49](manager/DECISIONS.md)), not three commands typed at once:

| Rule | Detail |
| --- | --- |
| Validation | every path first, before any file is touched. One refusal ⇒ exit `2`, the reason names the path, **nothing is processed anywhere** |
| Overlap | a path inside another path of the same batch ⇒ exit `2` `overlapping_roots`, both named; a file in both would be planned twice |
| Duplicates | the same canonical root twice is de-duplicated, recorded once |
| Order | the order typed on the command line |
| Lock | one run lock for the whole batch |
| State | one run record per folder, sharing a `batch_id` ([state](state.md) §3.1) |
| `confirm` | one proposal listing every folder, one question; declining installs nothing anywhere and exits `0` |
| A verification failure | aborts the batch: the remaining folders are `interrupted` and are not started |

```text
3 folders, 212 files, 1.9 GiB → 1.5 GiB (21.4% smaller)   [mode: confirm]

  downloads        /home/user/downloads            96 files   1.1 GiB → 0.9 GiB
  raw              /home/user/Pictures/raw         88 files   712 MiB → 548 MiB
  incoming         /tmp/incoming                   28 files    94 MiB → 77 MiB
  … two files and a total per folder, then one line per file

Originals will be deleted after each replacement is verified. There is no undo.
Continue? [y/N]
```

**A path that is not in the config is allowed and normal.** It is validated exactly as `folder add` would validate it
(forbidden root, symlink, cloud sync, `AUTO_TZIM_HOME` containment — `config.md` §5), and it is recorded in history
under its basename. `folder add` is how you stop typing the path, not a prerequisite.

**The plan is always built.** Even with `--auto` there is a plan: a record with an id and a hash over its ordered rows,
written before the first file is touched. What `--auto` removes is the *conversation* — the plan is not printed and
nothing is asked — never the plan itself, which is what makes "the file changed between planning and install"
detectable ([AC-16](manager/GATES.md)).

### 0.1 The cooldown is not a scheduler

`--time 24` does not make anything happen at 3 a.m.; nothing in `auto-tzim` runs by itself, ever ([D-47](manager/DECISIONS.md)).
It is a **cooldown**: the tool refuses to compress the same folder twice inside the window, so an outside timer can call it
as often as it likes without consequence.

```sh
# cron: try every hour; the cooldown decides whether anything happens
0 * * * *  auto-tzim /home/user/downloads --time 24 --auto
```

```xml
<!-- launchd, ~/Library/LaunchAgents/auto-tzim.downloads.plist, start interval 3600 -->
<key>ProgramArguments</key>
<array>
  <string>/usr/local/bin/auto-tzim</string>
  <string>/home/user/downloads</string>
  <string>--time</string><string>24</string>
  <string>--auto</string>
</array>
```

| Situation | Result |
| --- | --- |
| never compressed, or compressed ≥ 24 h ago | the run proceeds; the window restarts now |
| compressed 3 hours ago — **the only folder** | exit `6`, reason `too_soon`, the next eligible time printed; **nothing is read for processing** |
| compressed 3 hours ago — **one of several** | that folder is skipped and named in the summary, the rest run, exit `0` ([D-50](manager/DECISIONS.md)) |
| `--force` | proceeds regardless, and the window restarts from now |
| `--dry-run` | proceeds and prints the plan; the window is **not** consumed, because nothing was compressed |
| no `--time` | no cooldown at all: every invocation is allowed to compress |

The window is consumed **when processing starts**, not when it ends: a run that crashes half-way still used its window,
so a cron job cannot loop on a folder that keeps failing. Recovering from that is `--force`, which is a human decision.

The window is keyed by the folder's **canonical root**, not by its name, so renaming a folder does not hand it a second
compression, and a path typed on the command line shares the window with the same folder added to the config.

**Not the same thing as the age filter.** `min_file_age_seconds` in the config ([config](config.md) §4, default 60) skips
files that were modified too recently — it protects a download that is still being written. `--time` throttles whole runs.
Both exist; they are unrelated, and neither implies the other.

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

**No command ever prompts, and no PTY is allocated.** `stdin` is never read; a child process gets `/dev/null` on stdin
(`Stdio::null()`) (D-21). A command that needs a decision requires a flag. `--json`, a pipe and a scheduler behave exactly like a terminal,
minus the progress line.

## 2. Commands

### 2.1 `doctor`

Prints what a bug report needs: configured `tzim` path, `realpath`, size, SHA-256, `tzim --version`, the adapter entry,
the platform, the config location and schema, and the state directory status (schema, runs readable, tail recovery). It runs **one** command against the binary —
`tzim --version` — and never a file-processing invocation. Exit `0` consistent, `3` `tzim` missing/unreadable/outside the
adapter contract, `2` invalid config, `7` unreadable state.

### 2.2 `config validate`

This is the command that checks **reality**, not only the document: it reports a root that no longer exists, is now a
symlink or a file, and a `tzim_path` that is missing or not executable ([D-56](manager/DECISIONS.md)) — the read-only
folder commands do not, so an unplugged disk never makes a profile unrepairable.

Parses and validates **without touching state**: it takes no lock and creates no file or directory — a config write is an
atomic rename, so a reader needs no lock to see a whole file ([D-54](manager/DECISIONS.md), F-21). Exit `0` valid, `2` invalid with **every** error listed (not just the
first), each naming the field, the rule and a suggested correction.

### 2.3 `folder add PATH [PATH…] [--name NAME] [--recursive] [--allow-cloud-sync] [--formats LIST] [--exclude GLOB]...`

**Optional.** Remembering a folder is a convenience, not a prerequisite: `auto-tzim PATH` works on a folder the config has
never seen ([D-46](manager/DECISIONS.md)). What `add` buys is a stored name and stored defaults, so the path stops being
retyped.

Validates and canonicalises every `PATH`, creates one profile each, processes nothing, prints the resulting profiles
including the canonical roots. **All or nothing**: if one path is invalid, none is added ([D-49](manager/DECISIONS.md)).
`--name` applies only to a single path — with several, each takes its own basename. `NAME` defaults to the basename of the canonical root; it is the folder's identifier from then on
([config](config.md) §4). Exit `2` for a forbidden, relative, missing or symlinked root, a duplicate root or name, a
path inside `AUTO_TZIM_HOME`, or a non-empty `transforms` requirement it cannot satisfy.

### 2.4 `folder list`, `folder show NAME`, `folder edit NAME ...`, `folder enable NAME`, `folder disable NAME`, `folder remove NAME`

`edit` also takes `--allow-cloud-sync` / `--no-allow-cloud-sync`, the D-44 escape hatch ([D-54](manager/DECISIONS.md)). Every
subcommand takes the folder's **name**, matched case-insensitively and quoted if it contains spaces — there is no
opaque id to copy out of a file ([D-43](manager/DECISIONS.md)). `edit` also takes `--name`, which renames in place and
refuses a name another folder already holds. `show` prints the profile plus its last run and counts. `edit` validates
the whole result before writing, so a rejected edit leaves the file byte-identical. `remove` deletes the profile and its cached eligibility records — never a user file
— and prints what was removed.

### 2.5 `scan [PATH… | --folder NAME]…` — the read-only candidate plan

`auto-tzim PATH --dry-run` is the same thing with the path first; `scan` stays for scripts that prefer a verb.

**Several paths are one batch** (§0.2, [D-57](manager/DECISIONS.md)): every path is validated before a directory is read, one
refusal is exit `2` with nothing scanned **and no plan file written anywhere**, a duplicate canonical root is planned once, an
overlapping root is `overlapping_roots`, and output follows the order typed. Each folder gets **its own plan** — its own id,
its own hash, its own `plans/<plan_id>.json` naming its canonical root — because `--plan ID` binds one folder (§2.6) and the
hash covers one tree's rows ([D-52](manager/DECISIONS.md)). A batch is the same walk repeated per root, with validation done
once, up front. `--folder` may be repeated.

Builds the plan and **starts no file-processing `tzim`**. Specifically:

- runs discovery, stability and policy, and calls `tzim --version` once per run — in v0.1, once the adapter exists (AT-04);
  until then a scan spawns **nothing at all**, which the AT-03 brief makes acceptance point 5;
- never passes a target path to `tzim`, never passes `--dry-run`, never passes `--formats` when `formats = []`;
- writes nothing inside the target root and does not need write access there;
- emits one `planned` row per candidate with path, size, identity, the policy decision and the argv a future apply would
  use;
- does **not** claim eligibility, predicted savings, or "would optimize" — those numbers exist only after `tzim` has run
  (D-23, charter §8.4).

Each scan stores a plan record with an id and a hash over its ordered rows (D-20). That record is what authorises an
apply.

### 2.6 `run [PATH… | --folder NAME] [--auto | --mode auto|confirm] [--time HOURS] [--force] [--plan ID] [--dry-run]`

Several paths make a batch (§0.2). `--plan` binds **one** folder, so it is refused together with more than one path (`2`).

The processing entry point as a verb; `auto-tzim PATH` is the same run with the path first (§0). `--dry-run` is an alias
for `scan`: it produces the read-only plan and processes nothing (D-04). `--mode auto` and `--auto` mean the same thing;
the long form is kept because the mode also appears in the config.

**Two modes, one install path (D-29).** The mode gates exactly one thing: whether a proposal is shown and accepted.

| | `confirm` (default) | `auto` (`--auto`) |
| --- | --- | --- |
| Plan | built by this run, then **printed**; the run installs exactly what it printed | built inside the run, never printed ([D-48](manager/DECISIONS.md)) |
| `--plan ID` given | the tree must still match that plan, else exit `6` | same |
| Approval | proposal on stderr, one line on stdin | none |
| Output | proposal, progress on a terminal, summary | **one** summary line, and the exit code |
| Without a terminal, or with `--json` | **exit `6`**, nothing processed, message points at `--auto` | works |
| Cooldown (`--time`) | checked in both modes | checked in both modes |

The proposal is the last moment at which refusing costs nothing, so it is written to be read:

```text
Project images   /Users/example/Pictures/project   9 files   41.2 MiB → 33.8 MiB  (18.0% smaller)
  a.png      1.20 MiB → 0.98 MiB   -18%
  b.png      3.10 MiB → 2.55 MiB   -18%
  …
Originals will be deleted after each replacement is verified. There is no undo.
Continue? [y/N]
```

Anything other than `y` or `yes` — including an empty line, EOF and a closed stdin — is a refusal: exit `0`, nothing
installed, every planned file reported `skipped_by_user`, one history row marked `declined`.

**The work itself is identical in both modes**, and it never touches the user's folder until the last step:

1. re-stat the file against the plan; a changed file is `skipped_changed` and keeps its bytes;
2. copy it into `AUTO_TZIM_HOME/staging/<run_id>/` and `fsync`;
3. invoke `tzim` on **the copy** — argv array, stdin on `/dev/null`, no PTY, per-file timeout;
4. accept the result only on exit `0`, `status == optimized` and a smaller size; record the SHA-256;
5. install: `rename(original → .auto-tzim-old-<run>-<n>)`, `rename(staged → original)`, `fsync(directory)`,
   `unlink(.auto-tzim-old-…)` — journaled before each step.

Whether `tzim` changes a file is entirely its decision; our job is to record the verdict, delete an original only after
its verified replacement is in place, and prove that every non-candidate file is byte-identical afterwards.

### 2.7 `status`

Paused state, whether a run is active (and since when), the last run per folder, the counts of that run, and whether a
plan is pending (id, age, number of rows).

### 2.8 `history [--folder NAME] [--status STATUS] [--limit N]`

Recorded outcomes, newest first, `--limit` default 50. A row shows the relative path, the normalized status, sizes when
known, the `tzim` version and binary hash, and — for `skipped_by_tzim` and `failed` — the raw `tzim` status, `note` and
`error` verbatim, plus the stable `reason` code in its own field (D-27). An unknown `--status` value is exit `2`.

### 2.9 `logs [--folder NAME] [--limit N]`

The structured local log, rendered for humans; with `--json`, the raw JSONL objects.

### 2.10 `pause`, `resume`

`pause` sets the flag that makes `run --once` exit `6`. It does not abort an active run — stopping one means sending
`SIGINT`, which the run records as `interrupted`.

### 2.11 `update-check` — not in v0.1

The command does not exist in this release (D-28): v0.1 makes no outbound request of any kind, and there is no network
code in the binary to disable. Typing it is an unknown command, so it exits `2` — which is what AC-9 and corpus case
`W-02` assert. It returns in v0.2 as one documented `GET` ([update](update.md)) that never downloads and never replaces
anything; exit `5` stays reserved for it until then.

## 3. `--json` output

Typed NDJSON, one value per line, on stdout, nothing else, `"schema": 1` in every object (D-16).

| Command | Lines |
| --- | --- |
| `doctor`, `config validate`, `status`, `pause`, `resume`, `folder *` | one `{"type":"result", …}` |
| `scan`, `run --dry-run` | one `{"type":"file", …}` per candidate, then one `{"type":"plan", …}` |
| `run --once` (apply) | one `{"type":"file", …}` per processed file, then one `{"type":"run", …}` |
| `history` | one `{"type":"history","rows":[…]}` |
| `logs` | one `{"type":"log","lines":[…]}` |

```json
{"schema":1,"type":"result","command":"doctor","tzim":{"path":"/usr/local/bin/tzim","realpath":"/usr/local/bin/tzim","sha256":"…","size":1048576,"version":"0.16.1","adapter":"tzim-0.16","capabilities_schema":1,"result_schema":2,"version_status":"supported"},"platform":{"os":"linux","arch":"x86_64","toolchain":"rustc 1.xx (pinned)"},"config":{"path":"…","schema":1,"folders":1},"state":{"path":"…","ok":true}}

{"schema":1,"type":"file","folder":"01J…","plan_id":"…","path":"/Users/example/Pictures/project/a.png","relative_path":"a.png","status":"planned","policy":{"formats":"default","transforms":[],"reason":null},"identity":{"dev":1,"ino":42,"size":1258291,"mtime_ns":1690000000000000000},"argv":["tzim","--json","/Users/example/Pictures/project/a.png"]}

{"schema":1,"type":"plan","plan_id":"…","plan_hash":"sha256:…","folder":"01J…","created":"2026-10-29T19:00:00Z","candidates":9,"excluded":4,"tzim":{"version":"0.16.1","sha256":"…"}}

{"schema":1,"type":"run","run_id":"…","plan_id":"…","started":"2026-10-29T19:00:00Z","finished":"2026-10-29T19:02:11Z","counts":{"planned":9,"optimized":7,"already_optimal":1,"skipped_unsupported":1,"skipped_by_tzim":0,"skipped_policy":0,"skipped_unstable":0,"hardlink_skip":0,"failed":0,"interrupted":0},"binary":{"version":"0.16.1","sha256":"…"},"exit_code":0}
```

Rules:

- `null` means "not computed or not applicable", never "unknown but probably fine";
- `failed` rows carry `tzim_error` with `{"kind","message"}` when `tzim` supplied one;
- `skipped_by_tzim` rows carry `tzim_note` verbatim — it is data for a human, never a classification key; the stable
  `reason` (D-27) is copied from `tzim`'s `reason_code` and is never interpreted either, only displayed and counted;
- `reason` is `null` when the row optimized, and an unknown code is passed through as an opaque string, never mapped to
  one of ours;
- diagnostics (warnings, whole-command refusals) go to stderr as human text, so `--json > out.ndjson` always parses;
- `argv` is printed in the plan (it is what the user is being asked to approve) and stored in history for the apply.

## 4. Exit codes

| Code | Meaning | Typical source |
| --- | --- | --- |
| `0` | the command did what it says; documented skips included, and a **declined proposal** is a completed outcome, not a failure | any successful command |
| `1` | at least one file ended `failed` | apply |
| `2` | usage or configuration error; nothing processed | bad flag, invalid config, unknown `--status` |
| `3` | `tzim` absent, unreadable, or outside the adapter contract (`incompatible_tzim`) | `doctor`, `scan`, `run` |
| `4` | **never produced by `auto-tzim`.** Reserved out of every path so a `tzim` verification failure can never be confused with our lock — the lock is `8` | — |
| `5` | **reserved, not produced in v0.1** (D-28): it belongs to `update-check`, which ships in v0.2 | — |
| `6` | refused before processing: **every** folder still inside its cooldown, a stale `--plan`, `pause`, `min_free_bytes`, disabled folder, a `confirm` run with no terminal, whole-run policy refusal | `run --once` |
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

- **`--force` overrides exactly one thing: the clock.** It does not override `pause`, a forbidden root, the lock, a
  missing `tzim` or any refusal that protects a file — those are not schedules, they are safety. There is still no
  "ignore errors": a refusal is information, and the way to change eligibility is the folder policy.
- **One prompt, and only one.** The `confirm` approval gate (D-29) is the product's single interaction: one line on
  stderr, one line on stdin, `n` by default, refused when stdin is not a terminal. There is no `--yes` to skip it — the way
  to run unattended is `--auto`, which is a different mode with the same install path, not a silenced prompt. No pager, no
  TUI, no PTY anywhere.
- **`--auto` says nothing.** No plan, no per-file lines, no question: one summary line and the exit code
  ([D-48](manager/DECISIONS.md)). A user who wants to see what happened reads `history`, or omits `--auto` and answers the
  question. `--quiet` suppresses even the summary.
- **No `--jobs` in v0.1**: it exists in the config for v0.2 and is rejected today.
- **`-` is not stdin**: the program acts on files it must be able to re-read.
- **No glob patterns on the command line**: `exclude_globs` in the config is the only pattern language.
- **`run --dry-run` is an alias of `scan`**, not a `tzim` dry run. The names collide because `tzim` has a `--dry-run` of its own, which does the work and deletes a temporary candidate; ours does no work at all — see [CONFLICTS §C-1](spec/CONFLICTS.md).
