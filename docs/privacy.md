# Privacy and network surface

> **Status: specification, no implementation.**
> Related: [update](update.md) · [state](state.md) · [limitations](limitations.md)

The short version, for v0.1: **there is no network request at all.** Not one, not on start-up, not on a timer, not in a
dependency. This page exists because a tool that reads folders of personal files must be able to state that as a fact and
show how it is tested — and because v0.2 will add the first request, and the statement has to change in public when it
does (D-28).

## 1. The complete list of network activity

| Command | Request | When | Data sent |
| --- | --- | --- | --- |
| — | **none** | never | — |

There is no `update-check` command in v0.1 (D-28): it moves to v0.2, and when it lands it will be one `GET` to the GitHub
releases API, run only when the user asks, sending nothing but the request itself. The section you are reading is the
place where that sentence will be written, with the same table and the same "verified by" line.

Nothing else. `run`, `scan`, `status`, `history`, `logs`, `folder *`, `doctor` and `config validate` open no socket —
and in v0.1 that claim has no exception clause, which is the point.
`doctor` reports the `tzim` binary's SHA-256 **locally**. A scheduled check does not exist in v0.1; when it does
(v0.2), it will be opt-in and listed here before it ships.

There is no telemetry, no crash reporting, no analytics, no "check for updates" on start-up, and no undeclared
request hidden inside a dependency. Dependency review is part of acceptance: a library that phones home is refused
regardless of convenience.

## 2. What is stored locally, and how sensitive it is

| Data | Where | Sensitivity |
| --- | --- | --- |
| Folder roots (absolute paths) | config | high — a path can reveal a name, a project, a person |
| File names | history, logs | high — same reason |
| File sizes, timestamps, inode numbers | history | medium — reveals activity patterns |
| SHA-256 of rewritten files (within the documented cap) | history | medium — a hash of a known file is an identifier |
| The exact `argv` used per file | history | high — contains an absolute path |
| `tzim` binary path, hash, version | history, logs | low |

Consequences, all deliberate:

- **Permissions:** `AUTO_TZIM_HOME` and its state directory are `0700` where the platform supports it; the config,
  database and logs are `0600`.
- **Logs are local files**, structured JSONL, rotated by size and age (30 days by default). They are never uploaded
  and never printed to stdout unless the user asks with `logs`/`--json`.
- **Exports are the user's responsibility.** `history --json` prints absolute paths to the terminal; redirecting that
  into a bug report is a disclosure, and the documentation says so at the point of use.
- **No environment dump.** Bug-report output (`doctor`) lists versions and hashes, never environment variables, never
  `$HOME`, never the working directory.

## 3. What the operating system and other software will still do

`auto-tzim` cannot control, and does not claim to control:

- **Cloud sync clients** watching a folder (iCloud Drive, Dropbox, OneDrive, Google Drive, Nextcloud): they will see
  files change — and will see `tzim`'s temporary candidate during a dry run. That is one of the reasons
  `allow_cloud_sync = false` is the default.
- **Backup software** (Time Machine and similar): it will capture the pre- and post-optimisation versions according to
  its own schedule. That is the closest thing to an undo the product has, and it is not a guarantee we make.
- **Spotlight, Windows Search, antivirus**: files are read by indexers; nothing we can change or promise.
- **The `tzim` binary itself**: it opens no socket and checks nothing on the network (`tzim` documents this). We rely
  on that statement and record the binary hash so a different binary is visible.

## 4. Zero network activity is not a setting in v0.1 — it is the product

- There is nothing to switch off: the resolver is disabled in the test suite for every command, and a socket opened
  anywhere fails the run rather than being reported.
- Exit code `5` remains reserved for the v0.2 `update-check` failure and is not produced in v0.1.
- A user who wants a guarantee at the network layer can run the program with no network at all; nothing degrades, because
  nothing reaches for it.

## 5. What would require a new privacy review

Any of these changes the statement on this page, so it is a decision, not an implementation detail:

1. Any second endpoint (a mirror, a version manifest in the repository, a Homebrew API call).
2. Any automatic check (on start, on a schedule, after a run).
3. Any data in a request beyond the standard HTTP request itself — including a User-Agent that identifies the version,
   if a reviewer decides even that is too much.
4. Any telemetry, even "anonymous counts".
5. Any update mechanism that downloads an artifact (v0.3), which adds a download URL, a checksum source and a
   signature story to this page.
