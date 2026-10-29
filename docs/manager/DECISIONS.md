# Decision log

States: **adopted** (binding on the implementation, from the owner's charter v0.2 of 2026-10-29 or from a manager
decision the owner has not overridden), **proposed** (a default written down so that the first coder does not have to
invent it; the owner may change it without cost), **superseded**.

| ID | Decision | State | Source | Affects |
| --- | --- | --- | --- | --- |
| D-01 | Restricted adapter: `tzim --version` only, no prose parsing, fail-closed on an unknown contract | adopted | charter §6-18, §16 P-01 | AT-04 |
| D-02 | Status vocabulary and the one mapping table from `tzim` results to ours | adopted | charter §16 P-01 | AT-04, AT-05 |
| D-03 | Exit `6` for a preflight/policy refusal | adopted | charter §16 P-03 | AT-06 |
| D-04 | Scan/dry-run is a local read-only candidate planner; `tzim --dry-run` is forbidden | adopted | charter §6-2, §16 P-02 | AT-03, AT-04, AT-06 |
| D-05 | `transforms` must be empty in the MVP | adopted | charter §6-11, §16 P-01 | AT-02, AT-04 |
| D-06 | Invalidation key: identity + binary hash + adapter id + policy hash | adopted | manager | AT-05 |
| D-07 | `update-check` = one documented GitHub releases request, explicit command only | proposed (manager default) | manager | AT-07 |
| D-08 | A root may not contain `AUTO_TZIM_HOME` | adopted | manager | AT-02, AT-03 |
| D-09 | Hashes only for rewritten files, capped at 64 MiB, omission recorded | proposed (manager default) | manager | AT-05 |
| D-10 | Per-file timeout 900 s, `TERM` then `KILL` after 10 s | adopted | manager | AT-05 |
| D-11 | No hidden tightening: pass only what the profile sets, record the effective argv | adopted | manager | AT-04 |
| D-12 | `tzim` gets files, never directories, never `-r` | adopted | manager | AT-04, AT-05 |
| D-13 | Integration tests pin a release artifact by SHA-256; never `latest` | adopted | manager | AT-08 |
| D-14 | Stability tests inject the clock | adopted | manager | AT-03, AT-08 |
| D-15 | Dependencies pinned with hashes; `tomli-w` is the only write dependency | proposed (manager default) | manager | AT-01, AT-02 |
| D-16 | Our own JSON envelope: typed NDJSON, `schema: 1` | adopted | manager | AT-06 |
| D-17 | Local JSONL logs, 30 days, size cap, no environment dump | adopted | manager | AT-05, AT-09 |
| D-18 | Exit `7` internal, `8` lock, `4` unused, `130`/`143` signals; no raw `tzim` passthrough | adopted | charter §16 P-03 | AT-06 |
| D-19 | Precedence `130/143 > 7 > 2 > 8 > 3 > 6 > 5 > 1 > 0` and a fixed preflight order | adopted | charter §16 P-03 | AT-06 |
| D-20 | Apply requires a plan ID and plan hash produced by a scan of the same tree | adopted | charter §6-2, §8.4 | AT-03, AT-05, AT-06 |
| D-21 | No PTY is ever allocated; `stdin=DEVNULL`; no `pty` import | adopted | charter §6-12 | AT-05 |
| D-22 | One upstream issue for a capabilities/structured-reasons API, outside this repo's PRs — it is `tzim#85`; my duplicate `#86` is to be closed | adopted | charter §16 P-01 | none here |
| D-23 | A candidate plan is not a promise of eligibility or gain | adopted | charter §8.4 | AT-03, AT-04, AT-09 |

---

## D-01 · Restricted adapter

The adapter knows `tzim` by **one** call: `tzim --version`, whose output is `tzim <semver>` on stdout, exit `0`. From
that it decides whether this binary is in the supported range. Nothing else is read from the binary at scan time.

Consequences, accepted deliberately:

- the default-enabled format set is **not** displayed or reconstructed; `formats = []` means "do not pass `--formats`
  and let `tzim` decide" (D-11), so the set is whatever the binary has, which is exactly the delegation §6-9 asks for;
- `--info`, `--list-formats`, `--check-deps` output is never parsed, and neither is the per-file `note`
  (charter invariant 18);
- an unknown version, an unparseable `--version`, a missing binary, or a JSON contract the adapter does not know
  produces `incompatible_tzim` and exit `3` **before** any file is planned;
- the binary identity (path, realpath, size, SHA-256, version) is recorded per run, so a changed binary is visible
  (D-06).

*Test obligation:* a source-level assertion that no module reads `--info`/`--list-formats` output, plus behavioural
tests for unknown version, garbage version, schema mismatch and a refused probe.

## D-02 · Status vocabulary and the single mapping table

Normalized statuses, and who can produce each:

| Status | Produced by | Meaning |
| --- | --- | --- |
| `planned` | us | in the candidate plan, not yet decided |
| `optimized` | `tzim` | the file was rewritten and verified by `tzim` |
| `already_optimal` | `tzim` | nothing to gain; bytes unchanged |
| `skipped_unsupported` | `tzim` | the content is not a supported format or construct |
| `skipped_by_tzim` | `tzim` | **generic**: `tzim` returned `skipped`, whatever the reason. The raw `note` is stored verbatim and never interpreted |
| `skipped_policy` | us | our policy excluded it (format not allowed, hidden file, mount boundary, path outside policy) |
| `skipped_unstable` | us | identity changed, file too young, or vanished between stages |
| `hardlink_skip` | us | `st_nlink > 1` |
| `failed` | both | `tzim` reported a failure, or our timeout/parse guard fired; `error.kind` distinguishes |
| `interrupted` | us | the run stopped before this file finished, or a crash orphaned it |
| `incompatible_tzim` | us | the binary is absent, unreadable or outside the adapter's contract |

Note the change from earlier drafts: there is no `skipped_refused`. A refusal by `tzim` and a limit skip by `tzim` are
indistinguishable from outside (P-01 §5), so pretending to separate them would be inventing a capability (charter
invariant 18). One honest generic bucket plus the verbatim note is the correct answer.

Mapping from `tzim` results to run effects — the only place the two vocabularies meet:

| `tzim` result | Our status | Run effect | Our exit |
| --- | --- | --- | --- |
| `0`, `optimized` | `optimized` | continue | `0` |
| `0`, `already_optimal` | `already_optimal` | continue, cache | `0` |
| `0`, `unsupported` | `skipped_unsupported` | continue, cache | `0` |
| `0`, `skipped` | `skipped_by_tzim` + raw note | continue, cache | `0` |
| `1`, `failed` | `failed` + `kind`/`message` | continue, **no automatic retry** | `1` |
| `2` | `failed` (`internal`) + run abort | stop | `1` |
| `3` | `failed` (`io`/`internal`) + run abort | stop | `1` |
| `4` (verification failure) | `failed` (`verify`), rest `interrupted`, critical diagnostic | stop | `1` |
| `130`/`143`/`137` | `interrupted` | stop | `130`/`143` |
| not run: incompatible binary | no file status | stop before planning | `3` |
| malformed/partial JSON | `failed` (`internal`) | stop | `1` |

The raw `tzim` exit code and status are stored in history; they are **never** our exit code (charter invariant 19).

## D-03 · Exit `6` — refused before processing

`6` means: the run was refused before any file was read for processing and before anything changed. It covers the
first-run gate (no matching plan), `pause`, `min_free_bytes`, a disabled folder, and a policy refusal of the whole run.
It is not an error and not a success: a script must be able to tell it from both.

## D-04 · Scan is a local read-only candidate planner

`auto-tzim scan` and `auto-tzim run --dry-run`:

- run discovery, stability, policy and the adapter's `--version` check **only**;
- never invoke `tzim` with a target path, never pass `--dry-run`, never `--formats`, never a directory;
- never create, rename or delete anything inside a target root; they do not need write access there, and `mkstemp`-style
  calls outside `AUTO_TZIM_HOME` are forbidden;
- report a **candidate** list: path, size, identity, the policy decision, and the argv that a future apply would use.
  No predicted savings, no eligibility claim, no "would optimize" status (`planned` only) — the numbers are unknown
  until `tzim` actually runs (D-23).

The alternative — invoking `tzim` for exact numbers — was rejected by the owner because `tzim --dry-run` writes a real
candidate into the destination directory ([P-02](problems/P-02-dry-run-is-not-write-free.md)).

## D-05 · Empty `transforms`

`transforms` exists in the schema so that the opt-in has a home before it is usable (invariant 11), and it must be
`[]`. Any value is a validation error naming the reason and the missing preconditions: a machine-readable contract, an
installed build that has the handler, and a real-binary test. `formats` is not consent to a structural transformation.

## D-06 · Invalidation key

A cached decision (`already_optimal`, `skipped_unsupported`, `skipped_by_tzim`, `skipped_policy`) is reused only while
all of these are unchanged: `(st_dev, st_ino)`, size, `mtime_ns`, the `tzim` binary SHA-256, the adapter id, and a
canonical hash of the policy fields that produced the argv. A version string is not enough: two builds of one version
differ (FITS, `jpeg-v1`).

## D-07 · One request, one command · *proposed*

`update-check` performs `GET https://api.github.com/repos/RUdolf1517/tzim/releases/latest` with no credentials, a 10 s
timeout, and a 24 h cache. `scan`/`run` never touch the network. Rate limits and errors are exit `5`, never "up to
date". A major version outside `allowed_major_versions` is reported as incompatible and never acted on. Documented in
[privacy](../privacy.md). If the owner prefers another official source, it is one function and one documentation page.

## D-08 · `AUTO_TZIM_HOME` is never inside a managed root

Otherwise a run could plan its own database, and retention could delete a file a plan references. Validation rejects
such a root; discovery excludes the path as a second line of defence.

## D-09 · Hash policy · *proposed*

SHA-256 is computed for the before/after bytes of files that were actually rewritten, up to 64 MiB; above the cap the
column is `null`, explicitly. `SPEC.md` §7.4 says "if computed", so this stays inside the contract.

## D-10 · Timeouts

Per file: 900 s default, 1 s–6 h configurable. On expiry `SIGTERM`, wait 10 s, `SIGKILL`; the file is
`failed` (`internal`, `timeout`) and is not retried automatically. Value and signal path recorded in the run row.

## D-11 · No hidden tightening, and no invented default set

Only what the profile sets is passed. `formats = []` means "omit `--formats`" (D-01), not "pass the list we believe is
default". Anything we do not set stays `tzim`'s default, and the effective argv plus the resolved limits are stored so a
user can reproduce one file by hand.

## D-12 · Files, not directories

`tzim` is invoked as `tzim … <one file>`. Never `-r`, never a directory. Our approved plan is the only input, so
`tzim`'s own traversal can never see a path our policy did not approve.

## D-13 · Pinning

CI downloads a named release artifact of a named version, checks it against the release `SHA256SUMS`, and tests that
binary. No `latest`, no floating tag, no source build in the MVP job.

## D-14 · Injected clock

`min_file_age_seconds` is tested through an injected clock and an injected `stat`. One integration test uses a real
1-second age and is labelled.

## D-15 · Dependencies · *proposed*

`tomllib` (stdlib) reads TOML; `tomli-w` (pure Python, no dependencies) writes it; `sqlite3` and `urllib.request` come
from the standard library; no HTTP client library, no image or archive library, no `shell=True`. Test-only dependencies
are pinned separately. Every pin carries a hash and a CI job fails on an unpinned dependency.

## D-16 · Our own JSON envelope

Typed NDJSON on stdout, `"schema": 1` everywhere: one `{"type":"result", …}` for single-answer commands, one
`{"type":"plan", …}` after `scan`, one `{"type":"run", …}` after `run`, one `{"type":"file", …}` per file. `null` means
"not computed", never "probably fine". Diagnostics never on stdout. Field sketches are in [cli](../cli.md) §3; a field
added in code without that page changing is a review finding.

## D-17 · Logs

Local JSONL under `AUTO_TZIM_HOME`: timestamp, level, event, run id, folder id, relative path, binary identity, status.
No environment variables, no absolute path outside the root, nothing uploaded. Retention 30 days and a size cap.

## D-18 · The exit-code contract

| Code | Meaning |
| --- | --- |
| `0` | the command did what it says; documented skips allowed |
| `1` | at least one file `failed` |
| `2` | usage or configuration error; nothing processed |
| `3` | `tzim` absent, unreadable, or outside the adapter contract (`incompatible_tzim`) |
| `4` | **unused by `auto-tzim`** — reserved for nothing, kept out of every path so that a `tzim` verification failure can never be mistaken for our lock (the lock is `8`) |
| `5` | `update-check` failure |
| `6` | refused before processing: first-run gate, `pause`, `min_free_bytes`, disabled folder, whole-run policy refusal |
| `7` | internal error: corrupt state, unexpected exception, malformed child output that our guard cannot attribute to a file |
| `8` | another run holds the lock, or the lock cannot be taken |
| `130` / `143` | interrupted by `SIGINT` / `SIGTERM` |

The raw `tzim` exit code is stored in history and never becomes ours (invariant 19).

## D-19 · Precedence and the preflight order

Precedence, highest first:

```text
130/143  >  7  >  2  >  8  >  3  >  6  >  5  >  1  >  0
```

Reading it: a signal beats everything; an internal error beats a usage error (because it means our state is suspect); a
usage/config error beats a lock problem (nothing to lock for); a lock beats an incompatible binary (we cannot even look);
an incompatible binary beats a preflight refusal (the run cannot happen at all); a preflight refusal beats an update
failure; an update failure beats "a file failed"; and a file failure beats success.

The **fixed preflight order** a run must follow, each step able to return before the next:

1. parse arguments → `2`;
2. load config → `2`;
3. take the lock → `8`;
4. read the binary identity and the version → `3`;
5. pause flag → `6`;
6. folder enabled → `6`;
7. matching plan (first-run gate, plan hash) → `6`;
8. free space → `6`;
9. discovery, stability, policy, then per-file execution.

Putting the lock before the binary check is deliberate: two runs must not both pass a check and then queue up.

## D-20 · Plan binding

`scan` writes a plan record: `plan_id`, a SHA-256 over the ordered `(relative path, size, mtime_ns, dev, ino, policy
hash, binary hash)` tuples, the folder id, and the timestamp. `run --once` accepts `--plan ID`; without a plan that is
`complete`, belongs to the same folder, and has the same plan hash as the tree it is about to process, it exits `6` and
does nothing. A plan that changed even by one file is not "close enough": it is re-scanned.

## D-21 · No PTY, no prompts

No `pty` module import, no `openpty`, `stdin=DEVNULL` for every child, no TTY assumption anywhere in the runner, a
per-file timeout as the last line of defence. A child that blocks on stdin is killed and recorded `failed`.

## D-22 · Upstream issue for capabilities and reasons

One issue in `tzim` describes the future machine interface (a `--capabilities --json` with `version`, `schema`,
`formats`, `features`; and a structured reason code beside `note`). It is filed from this project and referenced here;
it is **not** part of any `tzim` pull request, and this project does not wait for it.

## D-23 · A candidate plan is not a promise

Scan output is a statement about which files our policy would hand over, not about whether `tzim` will change them or
by how much. Documentation must not call it eligibility, savings, or "would optimize"; the status of a planned file is
`planned`, and the outcome exists only after an apply.
