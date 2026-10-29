# Privacy and network surface

> **Status: specification, no implementation.**
> Related: [update](update.md) · [state](state.md) · [limitations](limitations.md)

The short version: `auto-tzim` has **one** network request, it belongs to one command, and everything else stays on
the machine. This page exists because a tool that watches folders of personal files must be able to say exactly what
it sends, to whom, and when.

## 1. The complete list of network activity

| Command | Request | When | Data sent |
| --- | --- | --- | --- |
| `update-check` | `GET https://api.github.com/repos/RUdolf1517/tzim/releases/latest` | only when the user runs the command (D-07, manager default) | An HTTP request: the host, the path, standard headers from the HTTP client, and the client's IP address. No paths, no file names, no configuration, no identifiers beyond what any HTTP request carries |

Nothing else. `run`, `scan`, `status`, `history`, `logs`, `folder *`, `doctor` and `config validate` open no socket.
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

## 4. If the user wants zero network activity

- Do not run `update-check`; `update_policy = "manual"` documents the intent, and no command checks implicitly.
- Block `api.github.com` at the firewall; `update-check` then fails with exit `5` and a message saying so, and nothing
  else in the product notices.
- Verify with the test in AT-07/AC-9: with the resolver disabled, `run` and `scan` behave identically.

## 5. What would require a new privacy review

Any of these changes the statement on this page, so it is a decision, not an implementation detail:

1. Any second endpoint (a mirror, a version manifest in the repository, a Homebrew API call).
2. Any automatic check (on start, on a schedule, after a run).
3. Any data in a request beyond the standard HTTP request itself — including a User-Agent that identifies the version,
   if a reviewer decides even that is too much.
4. Any telemetry, even "anonymous counts".
5. Any update mechanism that downloads an artifact (v0.3), which adds a download URL, a checksum source and a
   signature story to this page.
