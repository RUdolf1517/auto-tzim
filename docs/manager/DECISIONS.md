# Decision log

A decision is a requirement only in the state **adopted**. **Proposed** entries are the manager's recommended
default, written before any implementation exists so that the choice is not made by whoever writes the code
first. Entries marked **owner** change the product contract (`SPEC.md`) or a safety posture and cannot be adopted
by the manager (charter §2.2).

| ID | Decision | State | Affects |
| --- | --- | --- | --- |
| D-01 | Version-keyed adapter for `tzim 0.14.x` + `--version` + a dry-run JSON probe; no human-output parsing | proposed (owner notice) | AT-04, AT-08 |
| D-02 | One mapping table from `tzim` statuses and exit codes to our statuses and exit codes | proposed | AT-04, AT-05, AT-06 |
| D-03 | Pre-flight refusals get a new exit code `6` | **owner** | AT-06, SPEC §8.2 |
| D-04 | `scan` = plan only; `run --once --dry-run` = plan **and** `tzim -n --json` | proposed (owner notice) | AT-03, AT-04, AT-06 |
| D-05 | `transforms` allowlist is empty in v0.1; any value is a validation error | **owner** | AT-02, AT-04 |
| D-06 | Invalidation key is file identity + binary identity + adapter id + policy hash | adopted | AT-05 |
| D-07 | `update-check` = one documented GitHub API request, explicit command only | **owner** | AT-07, privacy |
| D-08 | A managed root may not contain `AUTO_TZIM_HOME` | adopted | AT-02, AT-03 |
| D-09 | File contents are hashed only within a documented size cap; the omission is recorded | proposed | AT-05 |
| D-10 | Per-file timeout default 900 s, `SIGTERM` then `SIGKILL` after 10 s; configurable downwards | adopted | AT-05 |
| D-11 | We never tighten `tzim`'s limits silently; the effective argv and limits are recorded | adopted | AT-04, AT-05 |
| D-12 | `tzim` is given files, never directories, and never `-r` | adopted | AT-04, AT-05 |
| D-13 | Integration tests pin a release artifact by SHA-256; "latest" is never used | adopted | AT-08, CI |
| D-14 | Stability tests inject the clock; no test sleeps for `min_file_age_seconds` | adopted | AT-03, AT-08 |
| D-15 | Runtime dependencies are pinned with hashes; the only write dependency is `tomli-w` | proposed | AT-01, AT-02 |

---

## D-01 · Version-keyed adapter, with a probe instead of prose

`tzim` exposes `--version` (`tzim <semver>`) and nothing machine-readable about its compiled formats
([study G-1](TZIM-STUDY-2026-10-29.md#g-1--no-machine-readable-capabilityformat-list)). `auto-tzim` therefore
ships an adapter table keyed by `(semver range, JSON schema)`, containing the exact argv it may build, the status
enum it may receive and the exit-code mapping. Unknown versions, an unreported `schema`, or a failed probe are
`incompatible_tzim` (exit `3`), never a best guess. `doctor` may additionally run `tzim -n --json` on a bundled
1×1 PNG inside a private temporary directory; that probe proves "the png handler works and emits schema 1" as a
fact, without reading a single line of prose.

*Overturned by:* an upstream `--capabilities --json`. That would replace the probe, not the version pinning.
*Evidence required:* an adapter test for at least one unsupported version, one malformed `--version` output and
one probe failure, each producing `incompatible_tzim` and exit `3`.

## D-02 · One mapping table

`tzim`'s six statuses and six exit codes are mapped in exactly one reviewed table
([study §1, §3 B-3, B-5](TZIM-STUDY-2026-10-29.md)). Rules:

| `tzim` result | Our status | Run effect |
| --- | --- | --- |
| exit `0`, `optimized` | `optimized` | continue |
| exit `0`, `would_optimize` (dry run) | `planned` → `would_optimize` recorded in history as dry-run only | continue |
| exit `0`, `already_optimal` | `already_optimal` | continue, no retry while identity unchanged |
| exit `0`, `unsupported` | `skipped_unsupported` | continue, no retry while identity unchanged |
| exit `0`, `skipped` | `skipped_refused`, raw `note` preserved | continue, no retry while identity unchanged |
| exit `1`, `failed` | `failed` + `error.kind` as reported | continue, **no automatic retry** |
| exit `2` (usage) | `failed` (`internal`) and a run-level abort | our bug; the run stops, exit `1` |
| exit `3` (fatal) | `failed` (`io`/`internal`) and a run-level abort | exit `1` |
| exit `4` (verification) | `failed` (`verify`), remaining plan `interrupted`, abort | exit `1` + critical diagnostic |
| exit `130`/`143` | `interrupted` for the file in flight, remaining `interrupted` | exit `1` |
| `tzim` missing, unreadable, wrong version, probe failure | no file status; run does not start | exit `3` |
| malformed/partial JSON, stdout not matching schema | `failed` (`internal`), abort after the schema guard | exit `1` |

Note the collision that must not be conflated: `tzim`'s exit `4` is a verification failure, while `auto-tzim`'s
exit `4` is "another run holds the lock". Both are documented, and the table above is the only place that maps
one to the other.

*Evidence required:* a fake `tzim` that can produce every row of this table, including exit `4` and a signal
exit, plus a test that the run-level abort marks the remaining plan `interrupted` and not `optimized`.

## D-03 · Pre-flight refusals need an exit code — **owner**

Three documented refusals happen before any file is touched: `run --once` on a folder whose first run has not
been previewed (§8.1), a run while `pause` is active, and a run below `min_free_bytes`. `SPEC.md` §8.2 has no code
for them: `1` means "some file failed" (false), `2` means usage/config error (arguable for the first-run gate,
wrong for low disk), `3` is an incompatible `tzim`, `4` is the lock, `5` is update. Reporting success `0` would be
dishonest, and a script cannot tell "nothing to do" from "I refused to start".

Proposal: add exit code **`6` — refused before processing; nothing was read for processing and nothing was
changed** (first-run gate, pause, low disk, disabled folder). This is a `SPEC.md` change, so it needs the owner.

*Until answered:* the coder must not invent a code, and AT-06 stays blocked on this one row only.

## D-04 · Two different dry runs

- `scan` builds **our** plan: discovery, stability, policy, argv preview. No `tzim` process.
- `run --once --dry-run` builds the plan and then invokes `tzim -n --json` per planned file, recording what
  `tzim` says it would do (`would_optimize` / `already_optimal`) — the numbers are exact, because `tzim`'s dry
  run does the real work ([study B-1](TZIM-STUDY-2026-10-29.md#3-behaviour-that-changes-our-design)).

Both satisfy the first-run gate of §8.1, and both are recorded as dry runs: history rows are marked `dry_run` and
`optimized` is never written for them. Documentation must state that a `tzim` dry run creates and removes a
temporary candidate in the target directory (needs write permission and free space) — our invariant 2 is about
bytes of user files, and pretending otherwise would be a false promise.

## D-05 · Empty `transforms` allowlist in v0.1 — **owner**

Invariant 11 requires a separate explicit opt-in for confirmation-required transformations, and §6.3 introduces a
`transforms` field. But the only candidate today (`fits-tile`) is not in the released binaries
([study G-3](TZIM-STUDY-2026-10-29.md#g-3--build-dependent-features-are-invisible-and-the-released-binaries-ship-without-fits)),
so enabling it would be a promise we cannot verify end to end. Proposal: the field exists, the allowlist is empty,
and any non-empty value fails validation with the reason. This keeps the schema stable for v0.2 without shipping an
unverifiable transformation.

## D-06 · What invalidates a "nothing to do" record

A cached `already_optimal` / `skipped_unsupported` / `skipped_refused` is reused only while **all** of these are
identical: `(st_dev, st_ino)`, size, `mtime_ns`, `tzim` binary SHA-256, adapter id, and a canonical hash of the
policy fields that produced the argv. Version alone is not enough — two builds of the same version differ (the FITS
and `jpeg-v1` features prove it), and a changed policy changes eligibility. Any difference re-plans the file.

## D-07 · One request, one command, documented — **owner**

`auto-tzim update-check` (and only it) performs `GET https://api.github.com/repos/RUdolf1517/tzim/releases/latest`
with `Accept: application/vnd.github+json`, no credentials, a 10 s timeout, and a 24 h cache in our state
directory. `run`/`scan` never touch the network; a scheduled check is not part of v0.1. The privacy section must
say: this is the only network request in the product, it discloses the caller's IP address to GitHub, and no path
or filename is ever part of it. Rate-limit and error responses are reported as an update-check failure (exit `5`),
never as "you are up to date". An incompatible major is displayed as incompatible and never acted on.

*Alternative if the owner prefers no API dependency:* point the check at the releases HTML page and parse it —
strictly worse (fragile, still a request, still undocumented for machines). The API is the documented interface.

## D-08 · `AUTO_TZIM_HOME` may not sit inside a managed root

Otherwise a run could "optimize" its own SQLite database and logs, and a rotation could delete a file that a plan
still references. Validation rejects such a root with a reason; `AUTO_TZIM_HOME` is also excluded from discovery
as a second line of defence. (Plus `include_hidden = false` by default, which already hides `.config`.)

## D-09 · Hash policy

Hashing every file before and after doubles I/O for no safety benefit: `tzim` verifies, and our identity check
already uses `(dev, ino, size, mtime_ns)`. Proposal: SHA-256 is computed for the *before* and *after* bytes of
files that were actually rewritten, up to a documented cap (proposed 64 MiB), and recorded as `null` — explicitly,
not silently — above it. `SPEC.md` §7.4 says "if computed", so this stays inside the contract.

## D-10 · Timeouts

Per-file wall-clock timeout default **900 s** (`--zopfli` is 10–100× slower; a 2 GiB TIFF can legitimately take
minutes), configurable per folder down to 1 s and up to 6 h. On expiry: `SIGTERM`, wait 10 s, `SIGKILL`; the file
is `failed` (`internal`, reason `timeout`) and is not retried automatically. The timeout value and the signal path
are recorded in the run manifest.

## D-11 · No hidden tightening

Whatever limits a profile sets are passed through; what it does not set stays `tzim`'s default. The effective argv
plus the resolved limits are stored with the run so a user can reproduce a single file by hand. Inventing tighter
defaults inside `auto-tzim` would create a second, invisible policy layer — and skips that look like refusals.

## D-12 · Files, not directories

`tzim` is always invoked as `tzim … <one file>`, never `-r`, never a directory. Our approved plan is the only
input; two independent traversals with two independent eligibility rules is how a path outside the root gets
touched.

## D-13 · Pinning

Integration tests download a named release artifact (for example
`tzim-x86_64-unknown-linux-musl.tar.gz` of `v0.14.0`), verify it against the release's `SHA256SUMS`, and use that
binary. No `latest`, no floating tag, no build from source in the MVP job (a source build would test a different
artifact from the one users have). When a new release appears, the adapter entry and the pin move together in one
reviewed change.

## D-14 · The clock is injected

`min_file_age_seconds` must not make the test suite sleep. Discovery and stability take a time source and a
`stat` source through an explicit seam; unit tests drive both. The integration test that proves the real path
uses a 1-second age with a real sleep of at most 2 seconds, on one fixture, and is marked as such.

## D-15 · Dependencies

`tomllib` reads TOML (stdlib, 3.11+); writing needs a third-party library, proposed `tomli-w` (pure Python, no
dependencies) — pinning it is a supply-chain decision, so it is listed here rather than assumed. Test-only
dependencies (`pytest`, `pytest-timeout` if used) are pinned in a separate file. Every pin carries a hash and a
`lockfile`-style CI job, following the pattern `tzim` already uses.
