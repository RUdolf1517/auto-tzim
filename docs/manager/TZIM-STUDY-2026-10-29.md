# `tzim` study — what we may rely on, and what we may not

Date: **2026-10-29**. Subject: `RUdolf1517/tzim` at `main` = `d4e45974b32fc2bb696eee346892846fe3e0a5d1`, release
`v0.14.0` (published 2026-10-27T15:26:04Z). Purpose: decide, before any `auto-tzim` code exists, which parts of
`tzim` are a contract and which are documentation we must not lean on.

Method: the tree was cloned and read (`tool-executed`: `git clone --depth 1`), then specific files were inspected
(`source-inspected`). The binary itself was **not** executed — the release assets are served from
`release-assets.githubusercontent.com`, which this session cannot reach, and no Rust toolchain is installed. Every
behavioural claim below is therefore read from source or documentation and must be confirmed by executing the
pinned binary in CI (AT-08, section 5).

Files read: `README.md`, `docs/cli.md`, `docs/safety.md`, `docs/releases.md`, `docs/roadmap.md` (partly),
`Cargo.toml`, `src/main.rs`, `src/cli.rs`, `src/report.rs`, `src/replace.rs`, `src/error.rs`.

---

## 1. The contract surface `auto-tzim` may depend on

| Surface | What it is | Why it is a contract | Evidence |
| --- | --- | --- | --- |
| Invocation shape | `tzim [OPTIONS] <PATH>...` — the path is the command; no verb is needed | documented as a deliberate CLI decision, and a path is never mistaken for a command | `docs/cli.md` §1, §11.1 |
| `--json` | newline-delimited JSON: one `file` object per file in discovery order, then exactly one `summary` object; nothing else on stdout | documented schema, and "object keys and enum values are stable; new optional fields may be added in minor versions" | `docs/cli.md` §4; writer at `src/report.rs:301-363` |
| `schema` | integer in the summary object, `1` today, bumped on breaking changes | the adapter keys off it | `docs/cli.md` §4; `src/report.rs:349` |
| Status values | `optimized`, `would_optimize`, `already_optimal`, `unsupported`, `skipped`, `failed` | documented enum, not prose | `docs/cli.md` §3, §4 |
| Error object | `{"kind": …, "message": …}` with `kind` ∈ `io`, `parse`, `verify`, `internal`, present only with `status=failed` | documented, and confirmed in `src/error.rs:69-76` | `docs/cli.md` §4 |
| Exit codes | `0` no failures (including skips/refusals/unsupported), `1` at least one file failed, `2` usage, `3` fatal, `4` verification failure (run aborted, file untouched) | documented table with precedence `2 → 4 → 3 → 1 → 0` | `docs/cli.md` §5 |
| Signal exits | `130` on `SIGINT`, `143` on `SIGTERM`, `137` on `SIGKILL`, `134` on abort | documented in the failure-mode table | `docs/safety.md` §5 |
| `--version` | exactly one line, `tzim <semver>`, stdout, exit `0` | the only machine-shaped capability signal that exists today | `src/main.rs:22`; `docs/cli.md` §11.1 |
| `--dry-run` | full pipeline, candidate built and verified, only the final rename skipped | documented semantics, and the reason the reported numbers are exact | `docs/cli.md` §6 |
| `--formats <LIST>` | allowlist over `png`, `gzip`, `zip`, `ico`, `jpeg`, `tiff`, `zlib`, `fits`; `jpeg` is off unless named | documented; naming `jpeg` is the only way to enable it | `docs/cli.md` §2 |
| No stdin, no prompts | nothing in `tzim` reads stdin; there is no interactive mode | `grep -rn stdin src/` at `d4e4597` returns **no hits** | `source-inspected` |
| No own config, no daemon, no network | no config file, no environment variables of its own (only `NO_COLOR`/`CLICOLOR`/`COLUMNS`/`TERM`, which affect appearance only), no sockets, no telemetry | documented as a promise; "never checks for network access" | `docs/cli.md` intro, §10; `docs/releases.md` §1 |
| Replacement discipline | candidate written to `.tzim-<pid>-<nanoseconds>-<attempt>.tmp` in the destination directory, `fsync`, atomic `rename`; original untouched on any failure | documented algorithm with a per-failure-mode table | `docs/safety.md` §2, §5; `src/replace.rs:28` |
| Guarantee classes | G1 byte-identical, G3 format-equivalent, G4 metadata removed on request | per-format contracts with verification independence stated | `README.md`, `docs/guarantees.md` |

This is enough to run `tzim` safely: argv, stdout JSON, exit codes, and a version string.

## 2. Machine-interface gaps

Each gap is stated with its impact on `SPEC.md`, the fail-closed mitigation `auto-tzim` will use, and the
upstream action that would remove the gap. None of them is worked around by parsing human output.

### G-1 · No machine-readable capability/format list

`--info` and `--list-formats` are human text — `--list-formats` prints a three-column table
(`src/main.rs:196-221`), `--info` prints prose with a `Formats in this build: …` line (`src/main.rs:93-120`).
There is no `--capabilities`, and no JSON variant of either command (`grep -rn capabilit src/ docs/cli.md`
returns only an unrelated comment in the JPEG handler).

Impact: `SPEC.md` §7.1 asks a run to record "supported formats/opt-ins and the JSON schema version, if
available", and §6.3 makes `formats=[]` mean "default-enabled set of the current compatible `tzim`". That set is
**build-dependent**: the published binaries are built from
`default = [png, gzip, zip, ico, jpeg, tiff, zlib, zopfli]` (`Cargo.toml:40`), FITS is deliberately not in
`default` (`Cargo.toml`, `[features]` block) and `jpeg-v1` (independent JPEG verification) is not either. A tool
cannot see any of that.

Mitigation (decision D-01): a version-keyed adapter table (`tzim 0.14.x`) plus `tzim --version`, plus an optional
runtime probe that runs `tzim -n --json` on a 1×1 PNG fixture inside a private temporary directory. The probe gives
"the png handler exists and produces schema 1" as a fact, without parsing prose. Anything outside the adapter's
table — a different major, an unreported schema, a probe that fails — is `incompatible_tzim`, fail-closed.

Upstream action (not a blocker): an issue asking for `tzim --capabilities --json` with `version`, `schema`,
`formats`, `features` (including `fits`, `jpeg-v1`) and `opt_ins`.

### G-2 · No structured skip/refusal reason

`tzim` reports refusals and skips as `status: "skipped"` with a free-text `note`, and unsupported content as
`status: "unsupported"` with an optional `note` (`docs/cli.md` §4). There is no reason code.

Impact: `SPEC.md` §7.3 wants `skipped_refused`, `skipped_policy`, `skipped_unstable` and `hardlink_skip` as
distinct statuses, and §7.4 exempts some of them from retry while keeping `failed` retryable only manually. A
`skipped` bucket also covers cases that are not refusals at all — a compression that succeeded but whose rename
failed is reported `failed` (`io`), while limit and policy exclusions are `skipped`, and the reason is only
prose. Classifying by matching words in `note` is exactly what `SPEC.md` §2.1 forbids.

Mitigation (decision D-02): `auto-tzim` produces most of those statuses **before** spawning `tzim` — symlinks,
special files, hard links, instability, excluded paths, opt-in gates are all decided by our own discovery and
policy layer. The only statuses taken from `tzim` are the six documented ones; a `tzim` `skipped` maps to our
`skipped_refused` and the raw `note` is stored verbatim next to it. No word-matching, no heuristic. The
distinction "policy skip" vs "refusal" therefore comes from our layer, never from prose.

Upstream action: an issue asking for a stable `reason` code alongside `note` (for example `refused_lzw_tiff`,
`skipped_hardlink`, `limit_max_pixels`) so the mapping stops being lossy for refusals we did not pre-classify.

### G-3 · Build-dependent features are invisible, and the released binaries ship without FITS

`docs/releases.md` §2: "FITS is deliberately not compiled into these archives and requires a separate
`--features fits` build as well as `--fits-tile` at run time." `--list-formats` names FITS only when the feature
is compiled in (`README.md`).

Impact: a confirmation-required transformation such as `fits-tile` cannot be integration-tested against the
official release artifact that users actually have. Enabling the opt-in against a stock binary would produce
`unsupported` on every FITS file — safe, but the opt-in would be a promise we cannot verify.

Mitigation (decision D-05): in v0.1 the `transforms` field exists structurally but its allowlist is **empty**;
any non-empty value is a validation error with the reason stated. The field is introduced (satisfying invariant
11 and §6.3) without pretending a transformation is supported. Revisiting this needs either a verified
FITS-enabled build in the test matrix or FITS in the release archives — an owner-level scope decision.

### G-4 · Documentation drift inside `tzim`: `strip_metadata` in the summary object

The summary line emitted by the binary contains `"strip_metadata":<bool>`
(`src/report.rs:349`, and the argument list at `src/report.rs:355-362`), but the field table in `docs/cli.md` §4
does not list it.

Impact: small, but it is a live example of the rule "validate required fields, tolerate unknown optional fields".
An adapter that rejects unknown keys would break on a minor release that follows the documented policy of adding
optional fields.

Mitigation: the parser validates the fields it needs (`type`, `status`, `before`, `after`, `saved`, `schema`,
`dry_run`) and ignores unknown ones, recording the ignored keys per run so that drift is visible instead of fatal.
Also worth reporting upstream as a documentation fix (`source-inspected`, `related: docs/cli.md §4`).

### G-5 · `tzim` issue #83 proposes interactive confirmation — a future prompt surface

Issue #83 (`feat(cli): add interactive confirmation for opt-in format handling without a force bypass`) is open.
Today `tzim` has no prompt at all, so `stdin=DEVNULL` can never deadlock. Once a version prompts, a non-TTY
subprocess that must answer would either fail or hang.

Impact: charter invariant 12 ("scheduler/non-TTY/JSON modes never wait for a prompt") and `SPEC.md` §7.2
(`stdin=DEVNULL`, "prompts are impossible and are not served") become a **cross-repository compatibility
condition**: a `tzim` that prompts without a documented non-TTY refusal would be incompatible with `auto-tzim`.

Mitigation: the adapter pins a version range that provably has no prompts; any newer version is `incompatible_tzim`
until an adapter update documents its confirmation semantics, its non-TTY behaviour (refuse, with which status
and exit code) and its interaction with `--json`. Before enabling any confirmation-required transformation we
require from `tzim` a written statement that a non-TTY invocation refuses rather than waits. That is an issue in
`tzim`, not an `auto-tzim` workaround.

### G-6 · Update discovery and privacy

`tzim` documents stable asset URLs and a `SHA256SUMS` file, and states that the binary itself never checks the
network (`docs/releases.md` §1-2). There is no documented "latest version" machine endpoint inside `tzim`.

Impact: `SPEC.md` §11.1 requires update checks against "только официально документированный источник". Once
`auto-tzim` starts checking, that check is a **new network surface owned by `auto-tzim`** and must be documented
in our privacy section (`SPEC.md` §13), limited to one explicit command, and must send nothing but what an
unauthenticated request requires.

Mitigation (decision D-07): a single `GET https://api.github.com/repos/RUdolf1517/tzim/releases/latest`, invoked
only by `auto-tzim update-check`, never by `run` or `scan`, with a timeout, a cache of 24 hours, response handling
that tolerates rate-limit and error replies, and a documented note that the request discloses the client IP to
GitHub. Comparison uses the semantic version and `allowed_major_versions` from the profile; an incompatible major
is shown as incompatible and never "activated".

## 3. Behaviour that changes our design

These are not gaps; they are facts that must show up in our acceptance criteria and in our own documentation
honestly.

**B-1 — a `tzim` dry run is not write-free.** `docs/cli.md` §6: a dry run still creates a candidate "in the
destination directory (verification and exact sizing require the real bytes) and removes it afterwards". So
`run --once --dry-run` needs write permission **and** free space roughly equal to the candidate in the target
directory, and cannot work on read-only media. Our invariant 2 must be stated precisely: a dry run changes no
byte of any original and leaves no permanent file — it does not promise that the directory is never written.

**B-2 — orphaned temporary files are possible.** Names follow `.tzim-<pid>-<nanoseconds>-<attempt>.tmp`
(`src/replace.rs:28`) and can survive `SIGKILL`, `abort()` or loss of power (`docs/safety.md` §2). Discovery must
therefore never treat that pattern as a candidate — including when `include_hidden = true` — and must report such
files as a diagnostic. `auto-tzim` never deletes them: they are not its data.

**B-3 — exit `4` aborts the run.** A verification failure stops `tzim` even without `--fail-fast`, and earlier
files keep their verified changes (`docs/cli.md` §5). With one file per process, the correct response is: mark that
file `failed` (`verify`), stop the plan, mark the remaining planned files `interrupted`, write a critical
diagnostic, exit non-zero, and never retry automatically. `auto-tzim` exit codes have no "verification failure"
value; see decision D-02 for the mapping table.

**B-4 — a vanished path is reported as a failure, not a skip.** `docs/cli.md` §1: a non-existent path is "a failed
argument", exit `1`. Our pre-run stability check must make this rare: a file that is gone before the spawn becomes
`skipped_unstable` and never spawns `tzim`. If it vanishes in the window between the check and the spawn, the
`failed`/`io` result is recorded as-is and is not retried automatically (invariant 13).

**B-5 — our statuses are mostly ours.** Of the twelve normalized statuses in `SPEC.md` §7.3, `tzim` can only
supply `optimized`, `already_optimal`, `skipped_unsupported` (from `unsupported`), `skipped_refused` (from
`skipped`) and `failed`; `planned`, `skipped_policy`, `skipped_unstable`, `hardlink_skip`, `interrupted` and
`incompatible_tzim` are produced by `auto-tzim` itself. The mapping must live in one table with tests, not be
spread through the runner.

**B-6 — what a dry run reports for non-candidates must be measured, not assumed.** `docs/cli.md` §6 names
`would_optimize` and `already_optimal` as the dry-run statuses; whether `unsupported` and `skipped` also appear
in a dry run of a mixed directory is not stated outright. We will not guess: AT-08 asserts the observed set with
the pinned binary before any planner logic depends on it, and the fake `tzim` models whatever is observed plus the
documented cases.

**B-7 — `--strip-metadata` changes the acceptance rule to "never larger".** With that flag an equal-size result
can be accepted (`docs/cli.md` §2), so `saved == 0` does not imply "nothing happened". v0.1 does not enable it
(D-05), but the normalizer must not assume `saved > 0` for an accepted rewrite.

**B-8 — we pass files, never directories.** `auto-tzim` owns discovery (`SPEC.md` §6), so `tzim` must never be
given `-r` or a directory: the approved plan is the only input. This keeps `tzim`'s traversal from ever seeing a
path our policy layer did not approve, and it keeps the two layers' eligibility rules from diverging. It also
means `--jobs`, `--include`, `--exclude`, `--min-size`/`--max-size` and `--max-*` are *our* filters, and the ones
we pass to `tzim` are recorded verbatim in the run manifest.

**B-9 — limits are recorded, not silently tightened.** `tzim` has its own defaults (`--max-size` 2 GiB,
`--max-pixels` 100 000 000, `--max-uncompressed` 8 GiB, a derived `--memory-budget`, `--jobs min(cores,4)`).
`auto-tzim` v0.1 spawns one file per process, so `tzim`'s own parallelism is the only parallelism. We do not
invent tighter hidden defaults; whatever the profile sets is passed, and the effective argv and the resulting
limits are stored with the run (proposal D-11).

## 4. Exact checks for the real binary (input to AT-08)

These are the behavioural assertions the coder must implement in CI against a **pinned** release artifact whose
SHA-256 is verified from the release's `SHA256SUMS`. They are stated as expectations now, before any result is
seen, so that a mismatch is an adapter-incompatibility finding rather than a test that gets adjusted afterwards.

| # | Command | Expected |
| --- | --- | --- |
| T1 | `tzim --version` | one line `tzim <semver>`, exit `0` |
| T2 | `tzim -n --json <png fixture>` | one `file` row + one `summary` row on stdout, `schema == 1`, `dry_run == true`, no byte change to the fixture, exit `0` |
| T3 | `tzim --json <text file>` | `file.status == "unsupported"`, exit `0` |
| T4 | `tzim --json <missing path>` | `file.status == "failed"`, `file.error.kind == "io"`, exit `1` |
| T5 | `tzim --json --dry-run <hard-linked file>` | a `skipped` row carrying the hard-link reason in `note`, exit `0` |
| T6 | `tzim --json <symlink>` | a `skipped` row, exit `0` (never a followed target) |
| T7 | `tzim -n --json <png fixture>` then `SIGINT` during a long run | exit `130` or `143`, no replaced file, possibly an orphan `.tzim-*.tmp` |
| T8 | `<pinned binary> --json <file>` with `stdin` closed | no prompt, no hang, completes within the per-file timeout |
| T9 | `tzim --json --formats jpeg <baseline jpeg>` | either a rewrite or `already_optimal`; never a silently different format |
| T10 | checksum of the downloaded artifact vs the release `SHA256SUMS` | equal, or the job fails before running any assertion |

Anything T1–T10 cannot confirm (for example FITS, which the release archives do not carry) stays out of the MVP
claim and is written down as a limitation instead.

## 5. Upstream state on 2026-10-29 (pinning consequences)

- `main` = `d4e4597`; latest release `v0.14.0` (2026-10-27T15:26:04Z).
- **PR #84 open** (`Extend residual FITS acceptance coverage for #78`) with two failing measurement jobs —
  see [pre-flight P2](PREFLIGHT-2026-10-29.md). Open issues: #57, #59, #78, #79, #80, #82, #83.
- The specification documents used by this project live on branch `arena/b3d4085f-tzim` (`efc31756`) and are not
  merged into `main`.
- Consequence for `auto-tzim`: the adapter and the integration tests pin **`v0.14.0` artifacts by checksum**, not
  "latest". A later `tzim` release is incompatible until an adapter entry for it exists, and that entry is a
  reviewed change with its own tests — never a wildcard.

## 6. What is still unverified

1. Any runtime behaviour of the binary (everything above is read from source or docs).
2. Whether a dry run reports `unsupported`/`skipped` for non-candidates (B-6).
3. The exact JSON byte shape of `path_raw_b64` for non-UTF-8 names, and how `tzim` renders such paths in `path`
   (documented as lossy UTF-8; the raw bytes field is "present on Unix only when the path is not valid UTF-8").
4. Whether `--memory-budget` interacts with a 1-file-per-process invocation in a way that changes results for
   very large files — relevant only above ~1 GiB in v0.1.
5. Whether the release artifacts for `aarch64-apple-darwin` and `x86_64-unknown-linux-musl` behave identically
   for our cases (they should; CI on both platforms is what proves it).

Items 1–3 are AT-08 acceptance items; 4–5 are notes for the reviewer, not requirements.
