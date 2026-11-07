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
| D-15 | Dependencies pinned with hashes; ~~`tomli-w` is the only write dependency~~ **superseded by D-33**: pinned `Cargo.lock`, `cargo deny` allow-list | superseded (was a manager default) | manager | AT-01, AT-02 |
| D-16 | Our own JSON envelope: typed NDJSON, `schema: 1` | adopted | manager | AT-06 |
| D-17 | Local JSONL logs, 30 days, size cap, no environment dump | adopted | manager | AT-05, AT-09 |
| D-18 | Exit `7` internal, `8` lock, `4` unused, `130`/`143` signals; no raw `tzim` passthrough | adopted | charter §16 P-03 | AT-06 |
| D-19 | Precedence `130/143 > 7 > 2 > 8 > 3 > 6 > 5 > 1 > 0` and a fixed preflight order | adopted | charter §16 P-03 | AT-06 |
| D-20 | Apply requires a plan ID and plan hash produced by a scan of the same tree | adopted | charter §6-2, §8.4 | AT-03, AT-05, AT-06 |
| D-21 | No PTY is ever allocated; every child's stdin is `/dev/null`; no pseudo-terminal | adopted | charter §6-12 | AT-05 |
| D-22 | One upstream issue for a capabilities/structured-reasons API, outside this repo's PRs — it is `tzim#85`; my duplicate `#86` is to be closed | **delivered 2026-11-01** — `#85` implemented in `tzim` PR #91, shipped in v0.16.0; `#86` closed | charter §16 P-01 | [audit DR-01, DR-02](../audits/2026-11-01-tzim-v0.16.1.md) |
| D-23 | A candidate plan is not a promise of eligibility or gain | adopted | charter §8.4 | AT-03, AT-04, AT-09 |
| D-24 | `main` protection: **resolved 2026-11-01** — the owner chose §6a option 1, machine gates only: the required review is removed because one identity cannot satisfy it, and the gate moves to CI checks plus the written review verdict | **superseded by the owner's §6a choice** | [OWNER-ACTIONS](OWNER-ACTIONS.md) §6a; charter §2.2 | every PR to `main` |
| D-25 | No implementation start until the severity-A drift items of the 2026-11-01 audit are closed; the six `tzim` contract facts this project is built on changed under it | adopted | [audit](../audits/2026-11-01-tzim-v0.16.1.md); charter §3, §9 | AT-01…AT-09 (start), Q11–Q13 |
| D-26 | The adapter **reads `tzim --capabilities`** and fails closed on an unexpected `capabilities_schema_version` / `result_schema_version`; `--formats` is still never passed | adopted (owner, 2026-11-01) | [audit DR-01](../audits/2026-11-01-tzim-v0.16.1.md), Q11 | AT-04, AC-13, INV-18 |
| D-27 | `tzim`'s stable `reason_code` is **surfaced** in our own rows as `reason`, beside the verbatim `note`; no code ever drives our logic | adopted (owner, 2026-11-01) | [audit DR-02](../audits/2026-11-01-tzim-v0.16.1.md), Q12 | AT-05, `cli.md`, `state.md`, corpus `D-*` |
| D-28 | **`update-check` leaves the MVP** and ships in v0.2; v0.1 makes **zero network requests**. Charter §17.5 is overridden by this decision; the pinned `SPEC.md` is not edited | adopted (owner, 2026-11-01) | Q "AT-07", charter §16 | AT-07 → v0.2, `privacy.md`, `dependencies.md`, AC-9, INV-14/15 |
| D-29 | Two operating modes — `confirm` (proposal, then install on approval) and `auto` (no prompt) — sharing **one** install path; an original is deleted only after its verified replacement is in place | adopted (owner, 2026-11-01) | this section | AT-05, AT-06, `safety.md`, `cli.md`, `guarantees.md`, GATES AC-14…AC-17 |
| D-30 | Accepted `tzim` window: **`>=0.16.0, <0.17`**, tied to `capabilities_schema_version: 1` / `result_schema_version: 2`; widened only after a real run | **adopted (owner, 2026-11-01)** — Q13 answered | [audit DR-09](../audits/2026-11-01-tzim-v0.16.1.md), Q13 | AT-04 |
| D-31 | **`auto-tzim` updates itself as well as `tzim`** — v0.3 covers both, by delegating to the installer that owns each artifact, never by rewriting a package in place | adopted (owner, 2026-11-01); overrides charter §11.2 last bullet | SPEC §11.2, prior roadmap §7 | v0.3 issues, `update.md`, `releases.md` |
| D-32 | **Two update modes**, `confirm` and `auto`, separate from the optimisation modes; safety rules that protect a file and a binary hold in both | adopted (owner, 2026-11-01) | D-29 vocabulary | v0.3 issues, `config.md`, `cli.md` |
| D-33 | **The implementation language is Rust**, not Python: a static binary in a workspace (`auto-tzim-core` library + `auto-tzim` CLI), pinned toolchain, `cargo` build and test. Overrides `SPEC.md` §4.1 | adopted (owner, 2026-11-01) | `SPEC` §4.1/§17; [CONFLICTS C-5](../spec/CONFLICTS.md) | every document, AT-01…AT-09 |
| D-34 | **Three front doors, three artifacts**: the CLI, the TUI (v0.2) and the Qt app (v0.4) are separate binaries with separate installs; only the core crate is shared | adopted (owner, 2026-11-01) | D-33, charter §12 | v0.2, v0.4, packaging |
| D-35 | **No C dependencies, and no database engine**: state and history are plain files (JSONL + atomically replaced JSON) under `AUTO_TZIM_HOME`; `rusqlite` is removed | adopted (owner, 2026-11-01) | D-33; owner instruction | AT-02, AT-05, `state.md`, `recovery.md`, GATES, corpus `L-*` |
| D-36 | Distribution includes **`cargo install auto-tzim`**; the binary name keeps its hyphen; the CLI crate is published as `auto-tzim`, the library as `auto-tzim-core` | adopted (owner, 2026-11-01) | `releases.md`, `update.md` | AT-01, AT-09, packaging |
| D-37 | **Documentation-only pull requests** are merged by the manager without asking, with a STATUS-LOG line; the boundary is mechanical (no `.rs`, `Cargo.toml`, `.github/`, fixtures) and anything touching an acceptance row stays an owner decision | adopted (owner, 2026-11-02) | five "заливай" round trips in one day | every doc pull request |
| D-38 | "No C, no network" is enforced by **four mechanisms together** — exact pins, a deny-by-default allowlist, an explicit deny list, and the review that approves an allowlist entry; no one of them is the gate | adopted (manager default, 2026-11-02) | D-35; AT-01 question 3 | AT-01…AT-09, `deny.toml`, `dependencies.md` |
| D-39 | How a package is verified before publication — **withdrawn the same day by D-42** | *withdrawn* | review round 3 | — |
| D-40 | The **MSRV is the toolchain CI actually runs** — **1.89** since D-53 (was 1.82), not `tzim`'s: `auto-tzim` does not build `tzim` | adopted (manager default, 2026-11-02), number amended by [D-53](DECISIONS.md) | `rust-toolchain.toml`; review F-03 | every manifest, CI |
| D-41 | **CI-only tooling may be Python; the product may not** — with a self-test, a mention in `dependencies.md`, and folding into the Rust harness at AT-08 | adopted (manager default, 2026-11-02) | `scripts/check-pins.py`; review F-06 | AT-01, AT-08 |
| D-42 | **Packaging is not a CI gate** and nothing is published in AT-01; "publishable" means the manifest metadata, and the tarballs are verified at release, core first | adopted (owner's question, 2026-11-02) | three review rounds on an unpassable check | AT-01, `releases.md` §2 |
| D-43 | **A folder is identified by its own name** — no generated id: unique case-insensitively, the handle every command takes, renameable in place, and history rows keep the name they were written with | **adopted (owner, 2026-11-02)** — the manager's ULID proposal was rejected | `config.md` §4; owner's answer to the AT-02 question | AT-02, AT-05, `cli.md`, `state.md` |
| D-44 | The cloud-sync check is a **built-in list of well-known sync directories**, matched on the canonical path; such a folder is **rejected**, best-effort, bypassable only by the documented `allow_cloud_sync = true` | **adopted (owner, 2026-11-02)** | `config.md` §5 `cloud_sync_root`; charter §6-8 | AT-02 |
| D-45 | A root that is **itself a symlink is rejected**; intermediate components are resolved and the canonical path is stored, so `/tmp/x` on macOS is recorded as `/private/tmp/x` | **adopted (owner, 2026-11-02)** | `config.md` §5 `root_is_symlink`; D-08 | AT-02, AT-03 |
| D-46 | **The path comes first**: `auto-tzim PATH [--time HOURS] [--auto]` is one complete instruction; the plan is still always built, and `folder add` is a convenience, not a prerequisite | **adopted (owner, 2026-11-02)** | owner's CLI sketch; `cli.md` §0 | AT-02, AT-03, AT-05, AT-06, `README`, `cli.md` |
| D-47 | **`--time <HOURS>` is a cooldown, not a schedule and not an age filter**: compress the same folder at most once in that window, keyed by canonical root, consumed when processing starts, overridable with `--force`, refused with exit `6` `too_soon` | **adopted (owner, 2026-11-02)** — *"раз в 24 часа производится сжатие"* | owner's CLI sketch; `cli.md` §0.1 | AT-05, AT-06, `state.md` |
| D-48 | **`--auto` shows nothing at all** — no plan, no question, one summary line; without it the plan is printed and one answer is required | **adopted (owner, 2026-11-02)** — *"пользователь не видит даже плана и не соглашается"* | D-29; owner's answer | AT-06, `cli.md` |
| D-49 | **Several paths at once**: `auto-tzim PATH [PATH…] [--time H] [--auto]` runs a **batch** — all paths validated before anything, overlapping roots refused, one lock, one run record per folder under a shared `batch_id`, one question in `confirm` | **adopted (owner, 2026-11-02)** — *"можно добавлять несколько путей одновременно"* | `cli.md` §0.2; `state.md` §3.1 | AT-05, AT-06 |
| D-50 | **The cooldown and the exit code are per folder inside a batch**: a folder still inside its window is skipped and named, the others run; `6` only when *no* folder ran because of the clock | **adopted (owner, 2026-11-02)** | D-19, D-47 | AT-05, AT-06 |
| D-51 | **A check that reads manifests runs once, on Linux; macOS exists for platform behaviour.** Target shape: `build` (Linux), `test` (Linux + macOS), `deny` (Linux), `pinned-dependencies` (Linux) — 5 jobs, one macOS slot instead of four — plus `concurrency` and `timeout-minutes` | **adopted (owner, 2026-11-02)** — *"пусть кодер решит проблему macos jobs заодно"* | R-27; `ci.yml`; AT-02 | AT-02, every later stage |
| D-52 | **The plan is deterministic by construction**: id is 32 hex chars from OS randomness, the hash is SHA-256 over an ordered canonical row list (paths as bytes, argv included, ids and timestamps excluded), discovery is byte-wise sorted, depth is capped at 64 | **adopted (manager default, 2026-11-02)** — AT-03 cannot start without these four numbers | `state.md` §3.4; AC-1; corpus `K-14` | AT-03, AT-05 |
| D-53 | **File locking is `std`: MSRV moves 1.82 → 1.89**, where `File::lock` / `try_lock` / `unlock` were stabilised. `unsafe` stays forbidden, no wrapper crate, and the lock file is never deleted — the kernel releases it when the holder dies, which is exactly the `SIGKILL` case a plain lock file cannot survive | **adopted (manager default, 2026-11-02)** — answers the coder's AT-02 question 3; amends [D-40](DECISIONS.md) | `state.md` §5; `rust-toolchain.toml`; both manifests | AT-02, AT-05 |
| D-54 | **AT-02's seven grey areas decided**: an explicit `mode = "auto"` is valid (only the *default* is `confirm`); a non-empty `tzim_path` must exist, be a file and be executable; `--allow-cloud-sync` / `--no-allow-cloud-sync` exist on `folder add\|edit`; the permissions control is accepted as a whole-protection check with one added repair test; lock metadata is diagnostics only (`started_at`, host omitted when unknown); stale temp cleanup is safe **because** the config lock is held; and `config validate` takes no lock and creates nothing | **adopted (manager default, 2026-11-03)** — answers the coder's open questions on PR #34 | `config.md` §3; `cli.md` §2.2–2.4; `state.md` §5 | AT-02 |
| D-55 | **`macos-15-intel` may be measured as a diagnostic, never switched to silently**; the runner decision stays open and the AT-02 merge does not depend on GitHub's macOS capacity | **adopted (manager default, 2026-11-03)** — answer to the coder's D-51 question | R-27; [D-51](DECISIONS.md) | AT-02 onward |
| D-56 | **Validation splits in two**: *structure* (parsed into the model: schema, enums, names, duplicates, path shape, forbidden roots, cloud-sync rule, reserved features, ranges) is checked on **every** load and save; *environment* (the root exists, is a directory, is not a symlink, canonicalises; `tzim_path` exists and is executable) is checked only by `config validate`, by `folder add`, and at run time — an unplugged disk must not make the tool unrepairable | **adopted (manager default, 2026-11-04)** — amends [D-54](DECISIONS.md) item 2, after the review of `b1b6d5a` found the trap | `config.md` §5; `cli.md` §2.2–2.4 | AT-02 |
| D-57 | **The read-only plan is per folder, inside a batch**: `scan` / `auto-tzim PATH… --dry-run` takes several paths ([D-49](DECISIONS.md) applies unchanged), every path is validated before a single directory is read, one refusal ⇒ exit `2` with **nothing scanned and no plan file written**, duplicates are de-duplicated on the canonical root, overlap is `overlapping_roots`, and each folder gets **its own plan id, own hash and own `plans/<plan_id>.json`**, printed in the order typed | **adopted (manager default, 2026-11-06)** — the brief and `cli.md` §2.5 were written in the singular; the owner caught it | `cli.md` §0.2, §2.5; `state.md` §3.4; [HANDOFF-AT-03](HANDOFF-AT-03.md) | AT-03 |
| D-58 | **A named folder means the folder and everything in it**: `recursive` defaults to **`true`**, `--no-recursive` limits a run to the top level, `folder add --no-recursive` stores that choice, and every scan/run summary **states the scope** (directories walked, deepest level, subdirectories skipped) so "nothing found" is never silent | **adopted (manager default, 2026-11-06)** — the previous default (`false`, top level only) made the ordinary invocation find nothing in a downloads folder and say so quietly | `config.md` §3; `cli.md` §0, §2.3; [HANDOFF-AT-03](HANDOFF-AT-03.md) | AT-03 |
| D-59 | **What a plan records**: `argv` is the *profile-owned* part only — fixed flag order, `--`, then the **relative POSIX path** (never absolute: D-52 wants two machines to agree) — plus `root` (display, lossy) and `root_raw_b64` (exact); AT-04 owns every adapter flag; a non-UTF-8 argument is `{"b64": …}` in the canonical row; and "byte-identical" in acceptance 1 means the **rows**, not the file, because the id and `created` must differ | **adopted (manager default, 2026-11-07)** — answers the coder's argv, non-UTF-8 and acceptance-1 questions on PR #46 | `cli.md` §2.5; `state.md` §3.4; [HANDOFF-AT-03](HANDOFF-AT-03.md) | AT-03, AT-04 |
| D-60 | **A plan is published, not replaced**: write the temp file, `fs::hard_link(temp, final)` — which fails with `EEXIST` instead of overwriting — then unlink the temp and `fsync` the directory. A collision is `plan_id_collision` and exit `7`; there is **no fallback that can overwrite**, and a filesystem without hard links fails loudly | **adopted (manager default, 2026-11-07)** — the coder's proposal, which is better than the `rename` `state.md` §2 assumes | `state.md` §2, §3.4 | AT-03 |
| D-61 | **The order of the gates in a walk is contract**: excluded name → scope (depth, mount point, `recursive`) → type (regular file only) → hard link → age → format policy → planned. An orphan `.tzim-*…*.tmp` is decided by the **first** gate: no row, one `orphan_temp` diagnostic, even when it has several hard links | **adopted (manager default, 2026-11-07)** — the coder asked which rule wins for a hard-linked orphan temp | [HANDOFF-AT-03](HANDOFF-AT-03.md) §2 | AT-03 |
| D-62 | **Format policy reads the name, never the bytes**: `formats = []` selects every regular file; a non-empty list selects by a fixed extension table (`png`, `gzip`, `zip`, `ico`, `jpeg`, `tiff`, `zlib`), case-insensitively; anything else is `skipped_policy` `format_not_selected`. This is **our** allowlist, not a claim about `tzim`'s capabilities — [D-26](DECISIONS.md) item 3 said "`--formats` is still never passed", which was wrong and is corrected here | **adopted (manager default, 2026-11-07)** — corpus `P-02` required `--formats png` while D-26 appeared to forbid it (F-30) | `config.md` §3; corpus `P-02` | AT-03 |
| D-63 | **The glob grammar is exactly three wildcards**: `*` = any bytes except `/`, `**` = any bytes including `/`, `?` = one byte except `/`. Everything else is literal — no `[...]` classes, no backslash escapes, no braces, no negation. An empty pattern is a configuration error; nothing else is "malformed" | **adopted (manager default, 2026-11-07)** — the coder asked what an unsupported pattern should do | `config.md` §3 | AT-03 |
| D-64 | **The directory race is a named limitation, not a claim**: `std` cannot open a directory with `O_NOFOLLOW` without `libc`/`unsafe`, so a directory swapped for a symlink *during* enumeration can be read. What is **guaranteed** is that no published plan contains a row outside the canonical root (per-row canonical check ⇒ hard abort, `tree_changed`, exit `6`), and that a swap is detected by comparing `(dev, ino)` before and after enumeration through an **injected seam** so the test needs no race. Closing the window needs `openat`/`fstatat` — refused by [D-35](DECISIONS.md)/[D-38](DECISIONS.md), see [limitations](../limitations.md) §4 | **adopted (manager default, 2026-11-07)** — the coder asked whether best-effort is acceptable | `limitations.md` §4 | AT-03 |
| D-65 | **The 10,000-entry budget is a smoke bound, not a benchmark**: in a debug build on `ubuntu-24.04`, walking and planning one directory of 10,000 files must finish in **under 15 s**, the test prints the observed number, and the PR records it; setup is excluded from the measurement. A tighter number is refused because it would turn CI load into a red build | **adopted (manager default, 2026-11-07)** — acceptance point 10 referred to "the documented budget" that no document contained | [HANDOFF-AT-03](HANDOFF-AT-03.md) §3 | AT-03 |

---

## D-01 · Restricted adapter · *amended by D-26*

> **Amended 2026-11-01 by [D-26](#d-26--the-adapter-reads-the-capability-document).** The adapter makes **two** calls
> now — `tzim --version` and `tzim --capabilities`. Everything else below still holds: no prose is parsed, no
> `--list-formats`, fail closed on an unknown contract, identity recorded. Read "one call" as "two declared calls".

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

## D-07 · One request, one command · *superseded by D-28*

> **Superseded 2026-11-01 by [D-28](#d-28--no-network-in-v01).** `update-check` is not in v0.1 at all, so the request
> described here is v0.2 work. The design below is kept because it is what v0.2 will implement, not what v0.1 does.

`update-check` performs `GET https://api.github.com/repos/RUdolf1517/tzim/releases/latest` with no credentials, a 10 s
timeout, and a 24 h cache. `scan`/`run` never touch the network. Rate limits and errors are exit `5`, never "up to
date". A major version outside `allowed_major_versions` is reported as incompatible and never acted on. Documented in
[privacy](../privacy.md). If the owner prefers another official source, it is one function and one documentation page.

## D-08 · `AUTO_TZIM_HOME` is never inside a managed root

Otherwise a run could plan its own state directory, and retention could delete a file a plan references. Validation rejects
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

**Superseded on 2026-11-01 by [D-33](#d-33--rust-not-python).** The reasoning below belongs to the Python stack:
`tomli-w` was chosen because `tomllib` can only read. Under Rust the same requirement is met by `serde` +
`toml`, and the pinning policy moved to `Cargo.lock` plus `cargo deny`. Kept as written, because the *question*
("how does a dependency get pinned and verified?") did not change with the language.

## D-16 · Our own JSON envelope

Typed NDJSON on stdout, `"schema": 1` everywhere: one `{"type":"result", …}` for single-answer commands, one
`{"type":"plan", …}` after `scan`, one `{"type":"run", …}` after `run`, one `{"type":"file", …}` per file. `null` means
"not computed", never "probably fine". Diagnostics never on stdout. Field sketches are in [cli](../cli.md) §3; a field
added in code without that page changing is a review finding.

## D-17 · Logs

Local JSONL under `AUTO_TZIM_HOME`: timestamp, level, event, run id, folder name, relative path, binary identity, status.
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
hash, binary hash)` tuples, the folder name, and the timestamp. `run --once` accepts `--plan ID`; without a plan that is
`complete`, belongs to the same folder, and has the same plan hash as the tree it is about to process, it exits `6` and
does nothing. A plan that changed even by one file is not "close enough": it is re-scanned.

## D-21 · No PTY, no prompts

No pseudo-terminal, no `openpty`, stdin on `/dev/null` for every child (`Stdio::null()` in Rust), no TTY assumption anywhere in the runner, a
per-file timeout as the last line of defence. A child that blocks on stdin is killed and recorded `failed`.

## D-22 · Upstream issue for capabilities and reasons

One issue in `tzim` describes the future machine interface (a `--capabilities --json` with `version`, `schema`,
`formats`, `features`; and a structured reason code beside `note`). It is filed from this project and referenced here;
it is **not** part of any `tzim` pull request, and this project does not wait for it.

> **Delivered 2026-11-01.** [#85](https://github.com/RUdolf1517/tzim/issues/85) was implemented by `tzim` PR #91 and
> shipped in **v0.16.0**: `--capabilities` with a version-1 schema, and a stable `reason_code` on every non-optimizing
> row. The MVP consumes both ([D-26](#d-26--the-adapter-reads-the-capability-document),
> [D-27](#d-27--the-refusal-reason-is-passed-through)). This section stays as the record of what was asked, not as a
> description of the current binary.

## D-23 · A candidate plan is not a promise

Scan output is a statement about which files our policy would hand over, not about whether `tzim` will change them or
by how much. Documentation must not call it eligibility, savings, or "would optimize"; the status of a planned file is
`planned`, and the outcome exists only after an apply.

## D-24 · `main` protection — machine gates only · *applied 2026-11-02*

**Resolved 2026-11-01 (owner, §6a option 1) and applied 2026-11-02.** What is in force now:

- **no required reviews.** Removed first, because one identity cannot satisfy it: GitHub refuses to let an author approve
  their own pull request at every permission level, so the rule could only ever end in `--admin`;
- **no required status checks** — and this one was not a decision, it was a repair. The protection object required a
  context named `test`, no workflow in the repository produces any context, so every pull request was blocked
  permanently. A required check nobody reports protects nothing;
- **review moves to the written verdict** in [EVIDENCE-PROTOCOL](EVIDENCE-PROTOCOL.md): the acceptance rows are
  demonstrated and re-run by the reviewer, which is a stronger gate than a green badge, and it leaves a record;
- **verified, not assumed:** pull request #16 answered `mergeable_state: clean` after the second deletion, and
  `branches/main` reports `required_status_checks.enforcement_level: "off"` while `protected` stays `true`.

**Standing obligation that comes with it.** When AT-01 (#1) lands CI, required checks go back **with the context the
workflow actually reports**, never a placeholder — the failure of this very rule is the argument. Whether force-push
blocking and deletion restriction survived the deletions is not visible to this session's token; the admin dump in
[OWNER-ACTIONS](OWNER-ACTIONS.md) §6a answers it, and option 1 expects both to be on.

The text below is the 2026-10-29 state of the decision, kept because the interim rule it describes is how #11, #12, #13
and #15 were merged, and a reader should be able to see that the bypass was a documented exception rather than a habit.

---

**Decision as given by the owner on 2026-10-29: do not decide yet.** Three options are on the table in
[OWNER-ACTIONS](OWNER-ACTIONS.md) §6a (machine gates only; a second approving identity; status quo with logged bypasses).
None was chosen, so the project runs on an interim rule rather than on an unwritten habit:

1. A pull request against `main` that the owner's own account opens may be merged with `--admin` **only** when it is
   documentation-only.
2. Every such merge gets a [STATUS-LOG](STATUS-LOG.md) line naming the reason. Two have been logged so far (#11, #12).
3. The first pull request that touches code, tests, fixtures or CI is **not** covered by this interim rule: it waits for
   the §6a decision, because the entire point of that decision is what protects code.
4. If a documentation pull request ever changes an acceptance-relevant fact — a gate row, a fixture id, a decision, a hash —
   it stops being "documentation-only" for this purpose and is treated under (3).

**Why not simply keep the rule and bypass it silently:** a gate that is always bypassed stops being a gate and starts
training the habit of bypassing. Recording the deferral, and its interim rule, keeps the bypass visible and keeps the code
case open instead of letting it inherit the documentation case.

**Known cost, accepted:** until the decision lands, no pull request to `main` can carry an independent approval, because
the only identity able to approve is the one that authored it.

## D-25 · The start waits for the contract, not for the blocker · *satisfied 2026-11-02*

> **Gate open.** All six conditions below are closed: D-26 (`--capabilities`), D-27 (`reason_code`), the pin and fixture
> (DR-03), D-30 (the accepted window), and the exit-code and `EPIPE` rows (DR-07, corpus `X-17`); the
> [2026-11-01 audit](../audits/2026-11-01-tzim-v0.16.1.md) is closed with all ten findings mapped. What remains between
> this repository and a first line of code is the owner's go-ahead and a coder, not a document.
>
> The evidence that the wait bought something is now measurable in the other direction: the "no capa document, no
> `reason_code`" assumptions this decision froze were wrong about the current binary, and every one of them would have
> shipped into the first implementation.

**Decision:** the MVP start is gated on the severity-**A** findings of the
[2026-11-01 audit](../audits/2026-11-01-tzim-v0.16.1.md) being closed, not on charter §3 alone. §3 is satisfied — PR #84
merged on 2026-10-30, the upstream tracker has zero open issues and zero open pull requests, and v0.16.1 was published on
2026-11-01 — but six of the facts the specification was written against changed in the same window:

1. `tzim --capabilities` exists (capabilities schema 1, result schema **2**) — DR-01.
2. every non-optimizing row carries a stable `reason_code` — DR-02.
3. the pinned binary in our documents is three releases behind, and a frozen fixture hard-codes its version — DR-03.
4. the default format set now includes JPEG in `jpeg-v1` builds, so "delegate the default" means something different — DR-05.
5. exit-code precedence is now explicit, including `EPIPE` and missing-input behaviour — DR-07.
6. the adapter's accepted-version window was derived from a binary nobody can install today — DR-09.

Starting now would produce code that passes our tests and fails against every current `tzim`. That is worse than a
visible delay, because it would look finished.

**Why this is a manager decision and not a question:** ordering work and refusing a premature start is the manager's job
(charter §2.1, §9). Three parts of the fix are *not* mine and are asked as Q11–Q13: whether we consume `--capabilities`,
whether we surface `reason_code`, and which version window we accept.

**Reversal condition:** an explicit owner instruction to start anyway, recorded here with the reason and the accepted
consequence.

## D-26 · The adapter reads the capability document

`tzim` v0.16.0 answers `--capabilities` with one read-only JSON object: `capabilities_schema_version`, `result_schema_version`,
`build_features`, `compiled_formats`, `default_enabled_formats`, `selection_flags`, `handler_opt_ins`, `transformations`,
`confirmation_policies`. The MVP reads it at start and in `doctor`.

Rules that keep this from becoming a new source of invention:

1. **Read, never infer.** We act on the document or we fail; we never complete a missing field from the version string.
2. **Two schema numbers, both checked.** `capabilities_schema_version` must be `1` and `result_schema_version` must be `2`,
   the pair we are tested against. Anything else, including a *newer* number, is `incompatible_tzim` (exit `3`) with the
   observed values recorded. Fail closed, not forward.
3. **`--formats` is never *derived from this document*** (D-11, corrected by [D-62](DECISIONS.md)). The flag is passed exactly
   when the profile sets `formats`; what the capability document never does is supply, complete or correct that list. Knowing
   the default set does not make it a promise; the binary still decides.
4. **The document is not displayed as a guarantee.** `doctor` shows the observed set with the binary hash beside it, and the
   document says it is a description of that binary, not of the product (audit DR-05, option C refused).
5. **Cost:** one extra spawn at start, so "a scan is exactly one `tzim` invocation" becomes two. The harness contract is
   updated rather than left to discover this in a red test (see [testing-harness](../testing-harness.md) §4).

**Reversal condition:** a `tzim` release that removes the document, in which case the adapter falls back to the version-keyed
table and the audit is re-run — not a silent downgrade.

## D-27 · The refusal reason is passed through

Every non-optimizing `tzim` row now carries `reason_code`, a stable identifier, `null` when something was optimized.

- Our rows gain a field `reason`: the code we received, or `null`.
- `note` stays **verbatim and unparsed**. No branch in our code may read it. This keeps INV-18 intact: the prohibition was
  never "do not have a reason", it was "do not manufacture one from prose".
- An **unknown** code is passed through as an unknown code and is never treated as success, eligibility, or consent. A code
  we do not recognise changes nothing in the run; it is data for the human and for later analysis.
- Our own skip categories (`skipped_policy`, `skipped_unstable`, `skipped_changed`, `skipped_by_user`) are ours and are
  never mixed with `tzim`'s codes: the two live in different fields and are documented as such.
- The field is additive: a consumer written against the earlier shape still reads `status`, `note` and sizes unchanged.

**Why now:** the schema is unshipped, so a field is free. It answers the question users actually ask — *is this fixable?* —
without teaching anyone to read English prose in a log.

## D-28 · No network in v0.1

`update-check` is removed from the MVP. v0.1 makes **no outbound request of any kind**; `urllib.request` leaves the
dependency list; `privacy.md` becomes a one-paragraph statement instead of a section with an exception.

- Exit code `5` stays **reserved and unused** in v0.1, so v0.2 can introduce it without renumbering anything.
- **Charter §17.5 is overridden by this decision.** `SPEC.md` is pinned with a recorded SHA-256 and is **not** edited: the
  override lives here, dated and attributable, which is exactly the mechanism charter §16 established for P-01…P-03.
- The tests that guarded "update-check only notifies" (AC-9, INV-14) are not deleted: AC-9 becomes "**no command opens a
  socket**, asserted with the resolver disabled and a filesystem tripwire", which is a stronger claim than the old one.
- Cost, accepted: users of v0.1 are not told about v0.2. For a first release with no installed base, that costs nothing.

**Reversal:** v0.2 ships AT-07 under the old rules — notify only, one request, documented, opt-out.

## D-29 · Two modes, one install path

The owner's instruction, 2026-11-01: *there will be two modes — automatic (no confirmation) and optimize-with-confirmation
(the user reads the proposal, and once they agree the originals are deleted).*

### The modes

| | `confirm` | `auto` |
| --- | --- | --- |
| Plan | a separate `scan` step | computed inside the same run |
| Approval | **required**: proposal on stderr, one line on stdin | none |
| Who may use it | a person at a terminal | a terminal, cron, a script |
| Writes before approval | **none** | none — processing happens on a staged copy |
| Originals deleted | only after approval, file by file | file by file, as each replacement is verified |

**In `auto` there is still a plan.** The owner confirmed the semantics on 2026-11-01: a plan of files is computed inside
the run and optimized, originals are not kept. The plan is not skipped — it is the internal binding that lets us refuse a
file that changed between the walk and the work (step 2a). What `auto` removes is the *question*, not the plan.

**Mode selection:** `--mode auto|confirm` on the command line overrides `mode` in the config, which defaults to
`confirm`. `auto` must be **asked for**: a tool that destroys by default and asks forgiveness in the README is not the
product being built here.

### The single install path

Both modes run the same code; the mode gates one thing only — whether the proposal is shown and accepted.

1. **Preflight** — lock, binary identity and capabilities, `pause`, folder enabled, matching plan (in `confirm`; computed in
   `auto`), free space.
2. For each planned file, in plan order:
   a. **Re-stat** `(dev, ino, size, mtime_ns)` against the plan. A mismatch is `skipped_changed`; the original is not
      touched and the file is not processed. This is the anti-TOCTOU check, and it runs *immediately* before the work, not
      at plan time.
   b. **Copy** the file into `AUTO_TZIM_HOME/staging/<run_id>/` — never into the user's folder — and `fsync` the copy.
   c. **Run `tzim`** on the staged copy: argv array, stdin on `/dev/null`, no PTY, per-file timeout.
   d. **Verify the result** before it is allowed near the original: exit `0`, `status == optimized`, result smaller than the
      copy, and our own SHA-256 of the result recorded. Anything else, and the original is not touched.
   e. **Install**, journaled, in this order: `rename(original → .auto-tzim-old-<run>-<n>)` → `rename(staged → original)` →
      `fsync(directory)` → `unlink(.auto-tzim-old-…)`. At every instant at least one complete copy exists that recovery can
      restore from.
3. **Report**, then delete the run's staging directory.

### Rules that make deletion defensible

- **An original is deleted only after its verified replacement is in place and the directory is fsynced.** Not before, not in
  a batch at the end, not for a file that was not replaced.
- **We never delete an original we did not replace.** `already_optimal`, `unsupported`, `skipped`, `failed`, a file not in
  the plan, a file the user did not approve — untouched, byte for byte.
- **No undo after install.** The old bytes are gone by design, and the proposal says so in one sentence before asking. What
  changed is *when* the risk happens: nothing is destroyed until the user has read the proposal and agreed.
- **`confirm` never hangs and never assumes yes.** No TTY, `--json`, EOF, or a closed stdin is a refusal: exit `6`, nothing
  processed, with a message saying `--mode auto` exists for unattended use.
- **A declined proposal is not a failure.** Exit `0`, zero installs, every planned file reported `skipped_by_user`, one
  history row marked `declined`. Nothing is written to the user's folder.

### Cost, stated plainly

- Staging needs disk: free space must cover the staged copies plus headroom, checked *before* the first file, refused with
  `6` if not.
- Two modes are two paths to test. They are one code path plus one gate, and the corpus proves the gate is the only
  difference (`M-*` cases).
- A hard kill during install leaves a `.auto-tzim-old-*` beside the file. Recovery restores the original and says so; the
  file is never lost, but a user *will* see that name in a crash. It is documented in `recovery.md` rather than hidden.

### What this changes in the invariants

- **Invariant 12 is rewritten, not deleted.** "No PTY, ever" stays absolute. "No prompt, in any mode" becomes: *the only
  prompt is the `confirm` approval gate, it is one line on stderr and one line on stdin, and it fails closed when stdin is
  not a terminal.*
- **Invariant 16 is sharpened.** "No promise of undo" stays true; the documentation must now also say that nothing is
  destroyed before approval, and must not let that sentence imply an undo exists.
- Four new invariants (20–23) and four new acceptance rows (AC-14…AC-17) carry the mechanics.

## D-30 · Accepted `tzim` versions

`>=0.16.0, <0.17`, checked against the capability document rather than only the version string: within the window, the
schema pair (`capabilities_schema_version: 1`, `result_schema_version: 2`) must also match, or the answer is
`incompatible_tzim` (exit `3`). Anything outside the window is refused the same way, with the observed version, schemas and
binary SHA-256 recorded in history.

Widening the window requires a real run of the newer binary against the corpus (AT-08), not a reading of its changelog.

**Status: adopted by the owner on 2026-11-01.** The window is deliberately one minor series wide. It is not a claim that
`0.16.0` and `0.16.1` behave identically — it is a claim that both declare the capabilities/result schema pair we were
tested against, and that the pair is checked at runtime rather than assumed from the version string. A build inside the
window that declares a different pair is still refused.

## D-31 · Updating both binaries

The owner's instruction, 2026-11-01: *auto-update is not only for `tzim`; `auto-tzim` updates itself too, and there are two
modes — one updates automatically, the other asks the user.*

**This overrides two things, both recorded rather than quietly deleted.** Charter §11.2 ends with "Обновление `auto-tzim`
этим механизмом не выполняется", and [roadmap](../roadmap.md) §7 listed self-update among the things that are *not planned*
rather than deferred. The pinned `SPEC.md` is **not** edited (same rule as D-28): the override lives here. The roadmap entry
moves into v0.3.

### Both targets are now the same problem

The first draft of this section said the opposite, and D-33 is why it changed. `auto-tzim` was a Python package, so
"updating itself" meant rewriting `site-packages/`, an entry point and a dependency tree — surgery on someone else's
environment, whose failure modes are a half-importable installation that `pip check` cannot repair. That code was refused,
and the update was going to be delegated to `pip`/`brew`.

**Under D-33 both binaries are static Rust executables**, so one mechanism covers both:

```text
download to a temporary path beside the target ──► verify SHA-256 against the published SHA256SUMS
      ──► fsync ──► rename over the old path ──► keep the old inode until the new binary answers --version
      ──► verify (doctor, adapter probe, smoke run) ──► success: drop the old inode | failure: rename it back
```

That is the same shape for `tzim` and for `auto-tzim`, which is the whole point: one code path, one set of tests, one
failure mode. A `SIGKILL` during any step leaves a working binary — never half of one — because `rename` inside a
directory is atomic and the old inode is not unlinked until the new one has answered.

**What still differs, and must not be hidden:** a binary installed by Homebrew lives in a Cellar that the package manager
owns. Replacing it behind `brew`'s back makes the formula's bookkeeping wrong, and the next `brew upgrade` will either
silently revert the change or refuse. So:

| Installation | Who performs the update | Why |
| --- | --- | --- |
| a tarball from the release page, unpacked by the user | **we do**, by atomic replacement | we own the file's provenance, and we recorded its hash at install time |
| Homebrew formula in a tap | **we ask `brew` to do it**, then verify | the package manager owns the Cellar; replacing files under it is how installations become unrepairable |
| anything else — a system package, a checkout, an unknown path | **refused**, exit `6`, with the command to run by hand | provenance unknown, so automatic replacement is not ours to perform |

Provenance is **recorded at install time** and shown in `doctor`; it is never guessed from `PATH` at update time. A binary
that we did not install, at a location we cannot account for, is never replaced automatically.

### The rules that hold regardless of mode

1. **Nothing is updated while a run is active**, and no run starts while an update is in progress: one global lock, taken
   for the whole operation in both directions. The same rule as `tzim` (update.md §2.2).
2. **Identity is recorded on both sides** — version, provenance, artifact SHA-256 — before and after, in history.
3. **The result is verified before the update is considered finished**: the new binary answers `--version`, `doctor`
   passes, the adapter accepts `tzim`, and a read-only smoke run on a bundled fixture succeeds.
4. **A failed verification reverts to the previous version** — the kept inode is renamed back, or the package manager is
   asked to reinstall the old version — and the history row says `reverted`. This is the first release that may use the word
   "rollback", because it is the first one where rollback is a tested code path rather than a wish. If the revert itself
   fails, the update is reported `failed` with both versions and both hashes, and nothing else is attempted.
5. **Never a downgrade** unless the user asks for one explicitly: a version lower than the installed one is refused.
6. **Never across a contract change without consent**: a major-version jump, or a jump that changes a documented status,
   exit code or config field, is refused in `auto` mode and offered in `confirm` mode with the change summarised.
7. **No `curl | sh`, no unsigned redirect, no download executed before its published checksum matches.** A redirect to a
   host that is not the documented one aborts the update.
8. **Signing is not claimed.** Artifact integrity is SHA-256 against the published `SHA256SUMS`; that protects against
   corruption and a broken mirror, not against a compromised publisher account. [SECURITY.md](../../SECURITY.md) says so
   in those words rather than implying more.

### The order of an update

```text
lock ──► is a run active? ⇒ refuse 6
       ──► what does the user have? (version, installer, hash) ──► history
       ──► is a newer version published? ──► no ⇒ exit 0, "up to date"
       ──► mode gate: asks, or does not
       ──► tzim first (the binary we depend on), then ourselves
       ──► verify: --version, doctor, adapter probe, smoke run
       ──► success ⇒ history row "updated"  |  failure ⇒ revert, history row "reverted"
unlock
```

`tzim` goes first on purpose: if updating `tzim` breaks the pair, we still have a working `auto-tzim` to say so. Doing it
the other way round would leave the newer half of a mismatched pair in charge of diagnosing the older one.

## D-32 · Two update modes

Same vocabulary as D-29, deliberately: a user who has learned `confirm`/`auto` for optimisation should not have to learn a
second word for updates.

| | `update_mode = "confirm"` (default) | `update_mode = "auto"` |
| --- | --- | --- |
| When a newer version exists | prints what it is, what it changes, and the command | performs the update |
| Who acts | the user | the tool |
| Ask before replacing `tzim` | yes | no |
| Ask before replacing `auto-tzim` | yes | no |
| Where it can run | anywhere | anywhere, but refuses without confirmation only if a rule below bites |

**It is a separate setting from `mode`** (D-29), because the two decisions are genuinely different: "optimise my photos
without asking" says nothing about "replace the program that is doing it". Both defaults are the careful one, and both can
be raised independently.

```toml
mode = "confirm"          # optimisation: ask before touching files
update_mode = "confirm"   # updates: ask before touching the binaries
```

**What `auto` means in the presence of the rules above:** it removes the question, not the checks. A binary whose checksum
does not match is never installed automatically; a version that jumps the contract is never installed automatically; a
verification that fails reverts automatically and reports it. `auto` is "do the routine part without me", not "do whatever
is published".

**Recommended pairing, stated as a recommendation and not enforced:** `mode = "auto"` with `update_mode = "confirm"` for a
scheduled, unattended installation — the routine file work needs no supervision, while the rare event that changes the
software does. It is a recommendation because the owner's instruction was two independent switches, and enforcing a pairing
would be the manager deciding a product question.

**The residual risk in `auto`, named:** a bad release reaches the user without their consent, and the only things between
them and it are the release gate, the checksum, the smoke run and the revert. That is a real reduction in control compared
with `confirm`, and it is the user's choice to make — but the documentation must not describe `auto` as "safe because it
verifies", as if verification made the choice costless.

## D-33 · Rust, not Python

The owner's decision, 2026-11-01: *the project is built in Rust.*

`SPEC.md` §4.1 says "Core и CLI: Python 3.12+", with `tomllib`, stdlib `sqlite3`, Textual and PySide6 named under it, and
§17.1 says "Инициализировать Python package". That text is **not edited** — it is pinned with a recorded SHA-256 and stays
the document that was signed. The override lives here, dated and attributable, exactly as §16 established for P-01…P-03
([CONFLICTS C-5](../spec/CONFLICTS.md) records the conflict itself).

**Why this is the better fit, concretely.** Not "Rust is faster" — this program orchestrates a codec, it is not one.

1. **A static binary is the whole installation.** No interpreter, no virtualenv, no `pip check` to become unrepairable, no
   "works in my venv". The audience installs a tool on a machine that must keep working.
2. **Self-update becomes the `tzim` problem, not the `site-packages` problem** (D-31). A binary is replaced with a
   `rename` after a checksum check: atomic, revertible, already proven in the sibling repository. A Python package is
   replaced by surgery on someone else's environment, and I refused to write that code — under Rust, the refusal
   disappears because the situation does not exist.
3. **The zero-network invariant becomes structural.** In Python, `urllib` is one import away and the claim is defended by a
   test. In Rust there is no HTTP client in `std`, so an outbound request requires a dependency, and **a dependency is
   visible in `Cargo.lock` and greppable in CI**. AC-9 changes from "a test catches you" to "the artifact cannot do it
   without leaving a trace in the manifest".
4. **Same stack as `tzim`**: `clippy`, `cargo-deny`, musl static builds, `SHA256SUMS`, a Homebrew formula, the same CI
   shape. Two repositories with one author should not hold two build philosophies.
5. **`stdin`/`PTY` discipline is easier to prove**, because process handling is explicit and there is no interpreter to
   inherit anything.

**What it costs, stated plainly.**

- The Qt GUI (v0.4) is the loser: `PySide6` is mature and free, `cxx-qt` is neither. D-34 answers that by separating the
  artifact rather than by pretending the cost is small.
- The MVP takes longer: ownership, error types and process-group handling are more work than a one-line subprocess call
  in Python.
- Every Python-specific paragraph in this repository was rewritten on 2026-11-01. That work existed because the language
  decision arrived *after* 52 documents and 30 decisions — a fact worth remembering when the next foundational question
  appears.

**Accepted stack.**

| Concern | Choice | Why |
| --- | --- | --- |
| CLI | `clap` (derive) | argv exactly as specified, no parser hand-written, no shell |
| Config | `serde` + `toml` | strict typed schema; unknown fields rejected, not ignored |
| JSON | `serde_json` | the documented NDJSON output is a serialised struct, not string building |
| State | **plain files** — JSONL logs plus atomically replaced JSON (`std` only) | D-35: no database engine, no C in the build, and a format a user can read with `cat` |
| Hashing | `sha2` | binary identity and record hashes; pure Rust, no C |
| Errors | `thiserror` inside, no `anyhow` at the edges | the exit-code contract maps to error types, not strings |
| TUI (v0.2) | `ratatui` + `crossterm`, optional feature | a separate binary, not a startup branch |
| Qt (v0.4) | `cxx-qt` over `auto-tzim-core` | separate artifact (D-34) |

**Explicitly not adopted:** `tokio` or any async runtime (the runner is sequential, `jobs = 1`; concurrency without a proven
need is a second execution model to test), any HTTP crate in v0.1, any crate that spawns its own network activity, and
**any crate that compiles C** — no `rusqlite`, no `libsqlite3-sys`, no `openssl`, no `zlib` (D-35).

**Reversal condition:** a written owner decision, recorded here, with the reason. Not a drift discovered mid-implementation.

## D-34 · Three artifacts, one core

`auto-tzim` is not one program with three modes; it is one **library** with three front doors:

| Artifact | Crate | Installed by | Contains |
| --- | --- | --- | --- |
| `auto-tzim` | `auto-tzim` + `auto-tzim-core` | the release tarball, Homebrew, `cargo install` (D-36) | the CLI, the runner, the state |
| `auto-tzim-tui` | `auto-tzim-tui` + `auto-tzim-core` | optional, from v0.2 | the terminal interface only |
| `auto-tzim-qt` | `auto-tzim-qt` + `auto-tzim-core` | optional, from v0.4, its own package | the Qt interface only |

Rules that follow, and are gates rather than intentions:

1. **The core crate owns all behaviour**: discovery, policy, plan, staging, install, adapter, state. A front door that
   implements any of it is a review failure — the same rule charter §12 already stated, now with a crate boundary that
   makes it checkable (`cargo tree` shows who depends on what).
2. **Every front door is optional and separately installable.** A user who wants the CLI never downloads Qt; a user who
   installs the GUI is not forced to get a TUI.
3. **No front door shells out to another.** The Qt app links `auto-tzim-core`; it does not drive the CLI through a pipe and
   parse its stdout. A JSON output is an interface for *users*, not a private protocol between our own binaries.
4. **The CLI remains the reference implementation of the behaviour**, and every acceptance row is judged against it first.
   A front door is accepted only when it produces the same outcomes through the same core calls.

## D-35 · No C dependencies, and no database

The owner's instruction, 2026-11-01: *state — no C dependencies at all; the alternative is JSONL files next to the
history.*

**What changes.** `rusqlite` (`bundled` SQLite) is removed. The entire state layer becomes plain files: an append-only
JSONL log per run, atomically replaced JSON for the small mutable records, an exclusive-create file for the run lock. No
database engine, no C toolchain in the build, nothing on the host to link against.

**Why this is the better trade here.**

1. **A static binary with no C in it is the whole point of D-33.** `bundled` SQLite compiles ~250k lines of C into every
   build; it is the one part of the stack that could fail on an unusual target, and it is the one part a packager has to
   audit by hand.
2. **The workload is not a database workload.** One writer, a few thousand rows per run, append-only, read back in order.
   Every property we needed from SQLite — atomicity, durability, single-writer isolation — is achievable with `rename`
   and `fsync`, and those two primitives are in `std`.
3. **A user can read their own history with `cat`.** For a tool that touches personal files, that is not a nicety: it is
   the difference between "the program says what it did" and "the user can check".
4. **The failure mode is better.** A corrupt SQLite file needs tooling and often a copy to a lab; a corrupt JSONL file has
   exactly one likely failure — a truncated last line — and the reader can detect and report it.

**What we give up, recorded rather than glossed:** transactions across multiple files (replaced by one-writer-at-a-time
plus a journal with recovery), indexes (replaced by file naming and a linear scan, documented as a limit), and SQL as an
inspection tool. [state.md](../state.md) §4 tabulates the swap line by line, and §11 lists the limits we accept.

**What does not change:** the status vocabulary (D-02), the plan binding (D-20), the invalidation key (D-06), the retention
rules, and the exit code for a corrupt state (`7`). Those were properties of the product, not of the storage engine.

**Reversal condition:** measured evidence that the linear scan is a real problem at the scale users actually have
(hundreds of thousands of files in one folder), in which case the answer is still not a C database — it is an index file
written with the same primitives.

## D-36 · Distribution: `cargo install`, releases and Homebrew

The owner's instruction, 2026-11-01: *`cargo install` is planned; the binary keeps the hyphen.*

| Channel | Artifact | Who builds | What we record |
| --- | --- | --- | --- |
| GitHub releases | prebuilt static binaries (`x86_64`/`aarch64`, musl and macOS) + `SHA256SUMS` | our release workflow | the exact artifact hash |
| `cargo install auto-tzim` | built from source on the user's machine | the user's toolchain | `provenance: "cargo"`, plus the reported version |
| Homebrew tap | formula over the release tarball | Homebrew | `provenance: "brew"` |

Consequences that had to be written down rather than discovered:

1. **Two crates are published**: `auto-tzim-core` (the library) and `auto-tzim` (the CLI crate, which produces the
   `auto-tzim` binary). The workspace keeps its `[[bin]]`/package naming so that `cargo install auto-tzim` yields
   `auto-tzim` and not `auto-tzim-cli` — a published crate name is forever, so it is decided now (this is why the crate
   was renamed from `auto-tzim-cli`).
2. **The name keeps its hyphen** in the crate, the binary, the formula and every document. One spelling, everywhere.
3. **A `cargo install` build is not byte-identical to a release artifact**, and we do not pretend otherwise: same source,
   same lockfile, different compiler and flags. It passes the same checks (`--version`, `doctor`, adapter probe, smoke
   run) and its provenance is recorded, so an update can pick the right mechanism.
4. **`cargo install` needs a toolchain on the user's machine**, with at least the MSRV documented in
   [releases](../releases.md). A user without one uses the tarball.
5. **Updating a `cargo`-installed binary is delegated to `cargo`**, because cargo keeps its own bookkeeping in
   `.crates.toml`: replacing the file behind its back would make `cargo install --list` lie. `cargo install --force
   auto-tzim --version <v>` recompiles — slower, and true ([update](../update.md) §2.2).
## D-37 · Documentation merges do not need a round trip

**Decision (owner, 2026-11-02):** a pull request that touches **only** documentation — no `.rs`, no `Cargo.toml`, no
`.github/`, no fixture data — is merged by the manager without asking, and gets a [STATUS-LOG](STATUS-LOG.md) line like
every other merge.

**Why.** The opposite rule was correct when nobody could merge without `--admin` and every merge was a governance event
(D-24, before 2026-11-02). Once the review rule and the placeholder `test` check were removed, asking for permission to
merge a markdown file became a round trip with no decision in it: five separate "заливай" exchanges in one day, each
costing the owner attention and teaching nobody anything.

**The boundary is mechanical, and it is checked before every such merge:**

```sh
git diff --name-only origin/main...HEAD | grep -v '\.md$'    # must print nothing
git diff --name-only origin/main...HEAD | grep -E '\.(rs|toml|yml|yaml|json|lock)$|^\.github/'  # must print nothing
```

**What is NOT covered, and stays an owner decision:** any pull request containing code, tests, fixtures, CI or a release
artefact; any merge that would bypass a red check; and anything that changes an acceptance row, a decision or a fixture
expected value — those are governance changes that happen to be written in Markdown, and the D-24 rule lists them
explicitly.

**Reversal:** if a documentation merge is ever found to have carried a substantive change nobody approved, this rule is
withdrawn the same day and the reason recorded here.

## D-38 · How "no C, no network" is enforced, now that it is code

**Decision (manager default, 2026-11-02, answering AT-01's third question):** the dependency policy of D-35 is enforced by
four mechanisms together, and no one of them alone is the gate.

| Mechanism | What it catches | What it cannot catch |
| --- | --- | --- |
| **exact pins** (`=X.Y.Z`) in every manifest, checked in CI | a range that silently resolves to a new version | a transitive dependency nobody declared |
| **deny-by-default allowlist** in `deny.toml` | any crate that was never reviewed, including transitive ones | whether the crate is *behaviourally* safe |
| **explicit deny list** (`cc`, `rusqlite`, `libsqlite3-sys`, `openssl-sys`, `libz-sys`, `reqwest`, `ureq`, `hyper`, `curl`, `tokio`, `mio`, `socket2`) | the named families, with a message that says why | a C or network crate nobody thought of |
| **review at PR time** — the allowlist entry is a reviewed line | everything above, as long as the review is real | a **feature** flip on an already-allowed crate |

**Why this is a decision and not an implementation detail.** `cargo-deny` reads manifests, licences and advisory data; it
cannot tell whether a crate compiles C or opens a socket. So the allowlist is not a classifier — it is a *forcing
function* for review, and the review is the policy. Saying otherwise would be the same mistake as claiming a green check
proves a behaviour.

**Consequences, binding on later issues:**

1. A new dependency is deliberately **two steps** — add it, then add it to the allowlist — and the second step is where a
   human says yes. That friction is the feature.
2. A dependency change also gets `cargo tree -e features` in the pull request, because flipping a feature on an allowed
   crate is the one hole the allowlist has.
3. `docs/dependencies.md` describes this mechanism, so a contributor knows why their build fails and what to do.
4. The deny list is a safety net for the obvious families, never the argument. If a crate is not on the list, that is not
   permission.

**Reversal:** if a C-compiling or socket-opening crate ever reaches `main`, this policy failed, and the failure is recorded
here with the hole that allowed it — not patched by adding one more name to the deny list.


## D-39 · How a package is verified before anything is published · *withdrawn by D-42*

> **Withdrawn 2026-11-02, same day, by [D-42](#d-42--packaging-is-not-a-ci-gate-and-nothing-gets-published-in-at-01).**
> The second row below (`cargo package --no-verify`) turned out not to work on Cargo 1.82, and the third row (a guard
> asserting an expected failure) should never have been written. The decision is kept as the record of a wrong turn, with
> the reason the turn was taken.


**The problem, stated exactly.** `auto-tzim` depends on `auto-tzim-core` by version (D-36, and F-05 of the first review).
Cargo strips the `path` when packaging and resolves the dependency from the registry, so before the core exists on
crates.io, `cargo package -p auto-tzim` fails with:

```text
error: failed to prepare local package for uploading
Caused by: no matching package named `auto-tzim-core` found
  location searched: registry `crates-io`
```

That is correct behaviour, not a defect. It does mean the acceptance point "both crates are packagable" cannot be
satisfied in full before the release — and the two ways of forcing it would both be wrong: publishing early is
irreversible, and dropping the dependency contradicts D-34.

**Decision — four checks, each labelled with what it actually proves:**

| When | Check | Proves |
| --- | --- | --- |
| always | `cargo package -p auto-tzim-core` | the core tarball builds on its own; the core is a leaf |
| always | `cargo package --no-verify -p auto-tzim`, then assert the normalised manifest inside the `.crate` names `auto-tzim-core` and contains no `path` | the tarball is produced and the manifest a user would download is registry-correct — **not** that it builds |
| always | run `cargo package -p auto-tzim` and assert it fails with the exact message above | the failure is *this* failure. A red step nobody reads is dead within a week; a step that asserts red stays honest and flips on the day the core is published |
| at release | `cargo package -p auto-tzim` | full verification against the published core |

**Publication order is mandatory:** core first, CLI second, recorded in [releases](../releases.md) §2. A release that
publishes the CLI first is a release that cannot be installed.

**Reversal:** if cargo gains a way to verify an unpublished sibling dependency offline without a local registry, this
table shrinks — recorded here with the cargo version that made it possible, not silently relaxed.

## D-40 · The MSRV is the toolchain CI actually runs

**Decision:** `rust-version` is declared in both manifests and equals the channel pinned in `rust-toolchain.toml` — today
`1.82`, which is what every green job ran on. The number is raised deliberately, with a line here, never as a side effect
of a dependency bump.

**Why not `tzim`'s 1.88.** I recommended 1.88 in the first review and I withdraw that. `auto-tzim` does not build, link
or vendor `tzim`; it drives a binary at runtime. Matching `tzim`'s MSRV buys a user nothing and would invalidate every
piece of green evidence collected on 1.82 — in exchange for a number nobody asked for. What matters is that the declared
version is the tested one: `cargo install` on an older toolchain must fail in the compiler with a clear refusal, and
`cargo install` on the declared one must work.

**Why not the true minimum (1.74, given `clap`):** advertising a version CI never runs is a claim without evidence. If
someone needs an older toolchain, the honest way to learn how far back it works is to add that job and measure.

**Reversal:** an owner decision to support an older toolchain means adding a CI job that builds on it, and then lowering
the declared number. Not the other way round.

## D-41 · CI-only tooling may be Python; the product may not

**Decision:** scripts that run **only** in CI to judge the repository — today `scripts/check-pins.py` — may be written in
whatever the runners already provide. The product, the harness and anything a user installs stay Rust (D-33). The line is
"does this ship, or does it only judge what ships".

**Why it is worth a decision.** [testing-harness](../testing-harness.md) says the project's tooling is a Rust binary, and
F-06 pointed at the inconsistency. The harness is a different animal: the coder and the reviewer run it on their own
machines to reproduce a corpus case, so it must work without a language runtime. A pin check runs in CI, reads manifests
and exits.

**Conditions, all three binding:**

1. The script carries the same standard as code: a **self-test in CI** that asserts it fails on `"*"` and passes on
   `"=1.2.3"`. A checker with no control is a checker nobody has seen work.
2. It is named in [dependencies](../dependencies.md) §2a, so nobody has to discover it from a red build.
3. At AT-08, when the Rust harness exists, folding it in is a roadmap item rather than an ambition. If it is not folded
   in, this decision is the reason it is allowed to stay.

**Reversal:** anything a user, a packager or a release needs on their own machine may not be Python.

## D-42 · Packaging is not a CI gate, and nothing gets published in AT-01

**Supersedes [D-39](#d-39--how-a-package-is-verified-before-anything-is-published).** D-39 was wrong, the coder's
empirical result proves it, and the honest fix is to delete the gate rather than to find a cleverer way to satisfy it.

**What the coder measured.** On Cargo 1.82, `cargo package --no-verify -p auto-tzim` fails with the same
`no matching package named auto-tzim-core found` as the verified form. `--no-verify` skips the build check, not the
preparation step that resolves dependencies against the registry. So the middle row of D-39 cannot be executed at all,
and the "guard" row existed to assert a failure that nothing needed.

**Why the gate should never have existed.** Three facts, in order of importance:

1. **Nothing in this project publishes.** Publication is a release action (D-36), it is the owner's, it needs credentials
   this project does not have, and no part of AT-01 needs it. A gate that can only run after a release decision does not
   belong in the CI of a branch that is not being released.
2. **The acceptance point was my invention.** ROADMAP-AT-01 said "both crates publishable", meaning the manifests must
   carry what a registry requires. I turned that into `cargo package`, which is a different and much stronger claim — it
   builds the tarball **against the published registry**, and in a workspace with an unpublished sibling it cannot
   succeed by construction. Three review rounds, two red CI runs and one useless guard step were spent proving a fact
   about cargo rather than a fact about our code.
3. **A check that cannot pass before release is not a check, it is a reminder.** It teaches everyone to skip one red
   step, and the habit spreads to the steps that matter.

**What replaces it — two lines, both cheap:**

| Where | Check | Proves |
| --- | --- | --- |
| CI, every run | every manifest has `name`, `version`, `edition`, `license`, `description`, `repository`, `rust-version` — asserted by the pin checker's companion script, or by `cargo metadata --no-deps` in one line | the crates carry the metadata a registry requires. This is what "publishable" was ever meant to mean |
| release checklist, `docs/releases.md` §3 | `cargo publish -p auto-tzim-core`, then `cargo package -p auto-tzim`, then `cargo publish -p auto-tzim` | the tarballs build and install against a real registry. Owner's action, in that order, once |

**What is explicitly not done:** no `cargo package` in CI, no `--no-verify`, no assertion that a command fails. If the
project ever needs a pre-publication smoke test, the answer is a local registry (`cargo publish --dry-run` against one),
which is a release-time tool, not a branch gate.

**Reversal:** when the core is published, the release checklist verifies packaging for real — and if someone wants that
in CI afterwards, it goes in as a **release-branch** workflow, not on every pull request.

## D-43 · A folder is identified by its own name

**Status: adopted (owner, 2026-11-02).** The manager proposed a ULID — 26 Crockford characters, generated once, opaque.
The owner rejected it with one sentence: *"нет у каждой папки будет свое название"* — each folder has its own name. That
is the better answer, and the proposal was worse for a reason worth writing down.

**Decision.** The `name` field *is* the identifier. There is no `id`.

| Rule | Detail |
| --- | --- |
| Default | the basename of the canonical root at `folder add` time, `--name` to override |
| Shape | 1–120 characters after trimming; no control characters; no leading or trailing space |
| Uniqueness | compared case-insensitively, stored exactly as typed; a second folder with the same name is `duplicate_name`, and so is a rename onto an existing name |
| Commands | every subcommand takes `NAME` — `folder remove project`, not `folder remove 01J9Z…` |
| Rename | `folder edit OLD --name NEW`: the profile is rewritten, nothing else changes |
| History | rows written before a rename keep the name they were written with (AT-05). A name is a label, not a key into the past |

**Why my ULID proposal was wrong.** I argued from durability — a stable key survives a rename — and missed what the user
actually has to do all day: type the thing. An opaque id means every file operation becomes copy-paste out of
`config.toml`, and the first support question of the project would have been "which of these `01J…` strings is my
photos folder". A name the user chose is self-documenting, and the one thing it costs — a rename does not follow into
rows already written — is the *correct* behaviour: a history entry describes a run, and at that moment the folder was
called what it was called.

**Consequences, binding on AT-02 and later:**

1. `config.md` §4 no longer has an `id` row, and the example lost its `id = "01JEXAMPLE…"` line.
2. `cli.md` §2.3–2.5 take `NAME`; there is no id to quote.
3. `state.md` records `folder`, not `folder_id`, and says it is a snapshot.
4. A duplicate **root** is still an error on its own — two names for one directory would put two plans on the same files.

**Reversal:** this is the kind of change that becomes expensive the moment a user has a config file, because it changes
the handle they type. If a rename ever has to preserve identity across rows — the reason I reached for a ULID — the fix
is an optional `--keep-history` on `edit`, not a second identifier.

## D-44 · The cloud-sync check is a built-in list, and it rejects

**Status: adopted (owner, 2026-11-02)** — the owner confirmed the refusal: *"отменять такие папки"*. `config.md` §5 refuses a root that matches a "known cloud-sync root"
unless `allow_cloud_sync = true`, and `config.md` §4 calls the check "best-effort and documented as such" — but no
document listed the roots. A coder would have invented six paths and called them the list.

**Decision.** One table in `auto-tzim-core`, matched against the canonical root **and** its ancestors under `$HOME`,
case-insensitively:

| Family | Matched directory names under `$HOME` |
| --- | --- |
| Dropbox | `Dropbox`, `Dropbox (Personal)`, `Dropbox (Business)` |
| iCloud | `Library/Mobile Documents` |
| Google | `Google Drive`, `GoogleDrive`, `My Drive` |
| Microsoft | `OneDrive`, `OneDrive - *` |
| Yandex | `Yandex.Disk`, `Яндекс.Диск` |
| Other | `pCloud Drive`, `MEGAsync`, `Nextcloud`, `ownCloud`, `Sync` |

**A match is a refusal, not a warning.** `folder add ~/Dropbox/photos` exits `2` with `cloud_sync_root` and names the
flag that overrides it. The default is the safe one because the failure it prevents is silent: two programs writing the
same file, no error, no message, a file that is neither the old nor the new version.

**The semantics are the important half.** It is *best-effort*: a sync folder somewhere else is simply not detected, and
that is documented rather than solved, because the alternatives are reading another program's database (fragile, and a
privacy question of its own) or opening a network connection (forbidden by D-28). It is a **path comparison only** — no
file inside the folder is opened. A false positive costs the user one flag: `allow_cloud_sync = true`. A false negative
costs them a silent conflict between two writers, which is why the list exists at all.

## D-45 · Symlinked roots, and how far canonicalisation goes

**Status: adopted (owner, 2026-11-02)** — *"да"*. `config.md` §5 has both `root_is_symlink` ("canonicalisation must
not launder a link") and a requirement to store the canonical path, and those two pull in opposite directions on macOS,
where `/tmp` is a symlink to `/private/tmp` and every path under it has a symlinked component.

**Decision.** Two steps, in this order:

1. `lstat` the path **as typed**. If the path itself is a symlink, refuse with `root_is_symlink`. A user who points at a
   link has pointed at something that can be re-pointed tomorrow, and a plan built on it is not reproducible.
2. Otherwise canonicalise and store the result. Intermediate symlinked components are resolved, so `/tmp/x` is recorded
   as `/private/tmp/x`. That is not a leak: it is the truth about which directory the tool will walk, and it is what
   makes duplicate-root detection work when the same folder is spelled two ways.

**Consequence, documented rather than discovered:** on macOS a root added as `/tmp/photos` appears in `folder list` as
`/private/tmp/photos`. Both spellings resolve to the same root, so the second one is a duplicate — which is the desired
behaviour and the reason the second step exists.

## D-46 · The path comes first

**Status: adopted (owner, 2026-11-02).** The owner wrote the CLI he wants in one line:

```sh
auto-tzim /user/home/downloads --time 24 --auto
```

and the specification I had written did not offer it. Mine was `folder add` → `scan` → `run --once --mode auto`: three
verbs, a config file, and a stored plan id before the user could do the thing they installed the program for. The owner's
line says the same thing in one command, and it is right — a batch optimiser whose first interaction is editing TOML is
a batch optimiser people do not use.

| What the owner asked for | Where it lands |
| --- | --- |
| `auto-tzim <PATH>` | the path is the first argument; no subcommand, no profile required (§0 of `cli.md`) |
| `--time 24` | the **cooldown** — compress this folder at most once every 24 hours ([D-47](DECISIONS.md)); integer hours, `0` disables it. *This row used to say "only files older than 24 hours", which was my wrong first reading and is corrected here* |
| `--auto` | install without asking; the same thing `--mode auto` meant, with a shorter name |

**What this does not change, and would not change if the owner asked.** The plan is still built on every run, hashed and
recorded before the first file is touched (`--auto` removes the conversation, never the plan); every file is re-checked
against the plan immediately before install, and a file that changed is skipped rather than overwritten; `tzim` still runs
on a staged copy, never on the user's file; `tzim --dry-run` stays forbidden; there are still no prompts anywhere a
terminal cannot answer, and `--json`, a pipe and a scheduler behave exactly like a TTY minus the question.

**One gate is genuinely superseded, and it is recorded rather than quietly dropped.** [D-20](DECISIONS.md) required an
*externally stored* plan from a previous `scan` before the first apply of a folder, and exit `6` without one. With the
path-first form, the plan a `confirm` run shows and the plan it executes are the same object in the same process, so the
external step bought nothing: the protection was always "the user approved **this** plan", and identity re-checking is
what makes it real. `--plan <ID>` keeps the old binding available for scripts. Consequences:

1. `GATES.md` AC-11's first case becomes "apply whose plan no longer matches the tree", not "apply with no plan at all";
   AC-12 (fabricated plan id, foreign plan, mismatched hash ⇒ `6`) is unchanged and still tested.
2. `config.md` stays valid with **zero** folders: a path given on the command line needs no profile.
3. `folder add` remains, for repetition, and is the only way to change a persisted default.

**Why I did not just add an alias.** An alias `run --once PATH` would have left the documentation teaching the three-step
choreography, and the coder would have built the choreography. The first line of `cli.md` now shows the invocation a user
will type, and everything else in that page is what the one-shot form is made of.

**Reversal:** if the owner ever wants the two-step flow back, `--plan` already carries it; the cost is one paragraph of
documentation, not a redesign.

## D-47 · `--time 24` is a cooldown, not a schedule

**Status: adopted (owner, 2026-11-02).** The owner's words: *"--time 24 это раз в 24 часа производится сжатие"* — the
compression happens once every 24 hours. My first reading was wrong: I wrote it as an age filter ("only files that have
not changed in 24 hours"), which is a different feature — it protects an in-progress download rather than throttling the
tool. The owner's reading is the one that matches `auto-tzim /home/user/downloads --time 24 --auto`, because a cooldown is
what makes an hourly cron invocation harmless.

**Decision.**

| Question | Answer |
| --- | --- |
| What is measured | the interval since this folder's last compression, keyed by **canonical root** |
| Where it is kept | `state/last-compression.json`, atomic replace ([state](../state.md) §3.5) |
| When the window is consumed | when processing starts — not when it ends, not on `--dry-run`, not on a run refused with `6` |
| Too soon | exit `6`, reason `too_soon`, the next eligible time printed; nothing is read for processing |
| Escape hatch | `--force`, which bypasses the comparison and restarts the window from now |
| Default | `--time 0`: no cooldown. A manual run does what it says |
| Nothing runs by itself | there is no daemon, no timer, no scheduler in v0.1; the cooldown only constrains an invocation that happens |

**Why "consumed when processing starts" and not on success.** A folder whose compression keeps crashing would otherwise be
retried hourly forever by whatever timer calls it, and the log would fill with identical failures. Consuming the window
turns "something is wrong" into one failure per day plus a `--force` for the human who wants to retry immediately. That is
the right trade for a tool meant to be called by cron.

**Why keyed by canonical root.** The name is the user-facing handle ([D-43](DECISIONS.md)), and names are meant to be
changed. Keying the clock on the name would hand a folder a fresh window every time it was renamed — a loophole with no
upside — and a folder run from the command line (which may have no name at all) would not share the window with the same
folder added to the config.

**This is not the `[folder.schedule]` feature.** That table stays rejected in v0.1 with `not_in_v01`, because it describes
a *daemon*: the tool deciding on its own when to run. `--time` is the opposite — a limit on calls the user's own timer
makes. Both can exist later; only the second exists now, and `config.md` says so rather than leaving a coder to guess.

**Reversal:** if a real scheduler arrives in v0.2, `--time` keeps its meaning and the daemon uses the same file. Nothing
about this decision is wasted by that.

## D-48 · `--auto` is silent

**Status: adopted (owner, 2026-11-02).** The owner's words: *"--auto означает что пользователь не видит даже плана и не
соглашается; без пользователя покажется план и он соглашается/отказывается"*. Two modes, and the difference is exactly
what the user sees:

| | `confirm` — `auto-tzim PATH` | `auto` — `auto-tzim PATH --auto` |
| --- | --- | --- |
| The plan | printed, in full: every file, every size, the total | not printed |
| The question | one line on stdin; `n` by default; no terminal ⇒ exit `6` | none |
| Per-file output | progress only on a terminal | none |
| The end | the run summary | **one** summary line, and the exit code |
| `--json` | NDJSON on stdout, proposal on stderr | NDJSON on stdout, nothing else |

**What stays identical, and it is the whole point of [D-29](DECISIONS.md).** One install path, one plan, one set of safety
checks: the plan is still built, hashed and recorded before the first file is touched, every file is re-checked against it
immediately before install, and `tzim` still runs on a staged copy. `--auto` deletes the *conversation*, not the
machinery; a bug in `auto` is a bug in `confirm` too, which is what makes one test suite cover both.

**Why silence is the right default for `auto`.** It exists for an unattended caller — cron, launchd, a scheduler the user
writes themselves — where a plan printed into a log every hour is noise that hides the one line that mattered. A user who
wants to see the plan runs the command without `--auto`; that is one keystroke, and it is the mode whose whole purpose is
to be read.

## D-49 · Several paths at once are one batch

**Status: adopted (owner, 2026-11-02).** *"учти что можно добавлять несколько путей одновременно"* — one invocation may name
several folders:

```sh
auto-tzim ~/downloads ~/Pictures/raw ~/tmp/incoming --time 24 --auto
```

That is the whole change, and it forces six answers that a single-path tool never has to give. Each one is decided here so
that the coder is not inventing batch semantics in a for-loop.

| Question | Answer, and why |
| --- | --- |
| Is one bad path forgiven? | **No.** Every path is validated before any file is touched — forbidden root, symlink, cloud sync, `AUTO_TZIM_HOME` containment. One refusal ⇒ exit `2`, message naming the path, **nothing processed anywhere**. The alternative is a run that did half the job and reported failure, which is the worst of both |
| Two paths where one contains the other | exit `2`, `overlapping_roots`, naming both. A file inside both would be planned twice and could be processed twice; a containment check is cheap and this is exactly what it is for |
| The same path twice | de-duplicated on the canonical path and recorded once. Refusing would be pedantic; processing twice would be a bug |
| Locking | **one** global run lock for the whole batch ([state](../state.md) §5). A per-folder lock would let two batches interleave on the same tree |
| State | one `run.json` + `files.jsonl` **per folder**, each with its own `run_id`, all carrying the same `batch_id` ([state](../state.md) §3.1). A single record with several folders would make the per-folder history queries — `history --folder NAME`, "last run of this folder" — into scans over a multi-folder blob |
| Order | the order typed on the command line. Not alphabetical, not the config order: a deterministic order the user chose is the one they can reason about |
| Approval (`confirm`) | **one** proposal listing every folder with its file count and totals, and **one** question. Declining installs nothing anywhere and exits `0`. Per-folder prompts would break the one-prompt invariant ([D-21](DECISIONS.md)) and make an unattended `confirm` impossible to reason about |

**What does not change.** Every folder gets its own plan, hashed and recorded before its first file is touched; every file is
re-checked against that plan immediately before install; `tzim` still runs on staged copies; `--auto` still prints nothing.
A batch is a sequence of ordinary runs that share a lock and a summary — not a new engine.

**One failure stops the batch.** If `tzim` exits `4` (verification failure) for a file in the second folder, the batch is
aborted: the remaining folders are recorded `interrupted` and are not started ([D-02](DECISIONS.md)). A verification failure
means the assumptions about `tzim` are wrong, and continuing to touch more files on wrong assumptions is the one thing this
project must never do.

**`folder add` takes several paths too.** `auto-tzim folder add ~/Pictures/a ~/Pictures/b` adds both or, if one is invalid,
neither — the same all-or-nothing rule, for the same reason.

## D-50 · Inside a batch, the clock and the exit code are per folder

**Status: adopted (owner, 2026-11-02)**, implied by D-47 plus D-49 and decided here rather than left to the coder.

A batch of five folders where one was compressed three hours ago must still compress the other four. So the cooldown is
evaluated **per canonical root**, and its outcome is decided per folder:

| Situation | What happens | Exit code |
| --- | --- | --- |
| every folder eligible | all run | `0`, or `1` if a file failed |
| some folders inside their window | those are skipped and **named in the summary**; the rest run | `0` (or `1`), **not** `6` |
| **no** folder eligible | nothing runs, message says when each becomes eligible | `6` `too_soon` |
| `--force` | applies to every folder in the batch, and every window restarts from now | as above, minus the skips |
| `--dry-run` | every folder's plan is printed; no window is consumed | `0` |

**This is a deliberate refinement of precedence ([D-19](DECISIONS.md)), and it is written down rather than implied.** The
precedence table orders **whole-run** refusals; a folder skipped by the clock is not a refusal of the run, it is a result
of the run, recorded in that folder's row. Without this rule the numbers lie in the expensive direction: `6` outranks `1`,
so one folder's clock would mask a failed file in another folder, and a scheduler reading `$status` would see "not now"
for a run that actually broke something. `6` therefore means exactly what it always meant — *nothing was processed* — and a
mixed batch is never silent about the skips: they appear in the summary line and in `history`.

## D-51 · The macOS queue: fewer slots, same coverage

**Status: adopted (owner, 2026-11-02).** *"пусть кодер решит проблему macos jobs заодно"* — the coder fixes it as part of
AT-02. This decision sets what "fixed" means, because the obvious fixes are all wrong.

**The facts, measured today.** Four incidents in one day, and — more useful — the timings on runs that *succeeded*:

| Run | Result | Ubuntu jobs started | macOS jobs started |
| --- | --- | --- | --- |
| #25 (`c0c575a`, push) | 8/8 | +3 s | +10 min |
| #26 (`8bd8f56`, push) | 8/8 | +3 s | +10 min |
| #29 (`8c162e1`, push) | 8/8 | +3 s | +11 min |
| PR #30 (`297e32f6`) | 6 green, 2 cancelled | +3 s | **never acquired**, cancelled at 15 min |

So the macOS jobs are not slow; they wait ten minutes for a runner and are killed if the wait passes GitHub's window. The
queue is the product of **four simultaneous requests for an arm64 macOS runner on every push and every pull request**, and
two of those four have no business being on macOS at all.

**Decision — the target shape for `ci.yml`:**

| Job | Runs on | Why |
| --- | --- | --- |
| `build` | **Linux only** | a macOS `cargo test` compiles the whole workspace anyway, and the `Install CLI` step compiles the binary; a second macOS job proving `cargo build` adds a queue slot, not coverage |
| `test` | **Linux + macOS** | the one job that tests platform behaviour: fmt, clippy, the workspace tests, the CLI contract, the install. This is the macOS coverage [R-20](RISK-REGISTER.md) requires |
| `deny` | **Linux only** | it reads manifests, licences and the advisory database. None of those have an operating system |
| `pinned-dependencies` | **Linux only** | it reads manifests with a Python script. Same argument |

Eight jobs become five, and the demand for the scarce resource drops from four slots to one. Plus two hygiene items that
cost nothing and remove queue pressure:

- **`concurrency`** keyed on workflow + ref, so a superseded pull-request run is cancelled instead of queueing — **but not
  on `main`**. Amended 2026-11-02 after reading the first implementation: with `cancel-in-progress: true` applied to every
  event, two commits landing on `main` within a couple of minutes leave the first commit with a **cancelled** run and no
  green one — the exact state [D-24](DECISIONS.md) and [R-27](RISK-REGISTER.md) say must never be assumed to be passed. The
  condition is `cancel-in-progress: ${{ github.event_name == 'pull_request' }}`: superseded PR runs die, every commit on
  `main` gets its own completed run.
- **`timeout-minutes`** on every job, so a wedged job fails in twenty minutes instead of holding a slot for hours.

**What "fixed" does not mean, and these are forbidden:**

| Shortcut | Why it is not a fix |
| --- | --- |
| `continue-on-error: true` on macOS | the job would report green while testing nothing; the first real macOS regression ships |
| `\|\| true`, or a wrapper that swallows the exit code | same thing with more steps |
| `set +e` **used to capture output and then `exit $status`** | *not* a shortcut — amended 2026-11-02, because saying otherwise cost a working diagnostic. The `cargo deny` step captured its output, emitted an annotation with it, and re-exited with the real status. My acceptance line ("`set +e` must not appear") could not tell that apart from swallowing, so the step was removed — and with it the annotation that made a `cargo deny` failure readable. It is restored, and the rule is restated behaviourally: **no step may end with a zero exit when the command it ran failed** |
| marking a cancelled job as passed | a cancelled job is `not-run`, and `not-run` is never `passed` ([R-27](RISK-REGISTER.md)) |
| dropping macOS entirely | `cargo install auto-tzim` on a Mac is a supported path; untested platforms are where the bugs live |
| switching to `macos-15-intel` silently | a different architecture is a different platform. Allowed **only** with measured evidence in the pull request — queue time and a green run on both — and recorded here |
| a third-party retry action | a new dependency in the trust path for a scheduling problem ([D-38](DECISIONS.md)) |

**Acceptance — amended 2026-11-02, because the original was unsatisfiable.** It demanded *two pushes to `main`* before the
merge that creates `main` pushes, which the coder correctly refused to fake by pushing to `main` or by merging a draft. The
split is now:

| When | Evidence | Who checks |
| --- | --- | --- |
| before merge | **three** `pull_request` runs on the PR branch, each with a table of `run id · job · queued → started · conclusion`, macOS acquired within 5 minutes every time | coder, in the handoff; manager, independently from the API |
| before merge | one **cancelled** run, with the run that cancelled it (B3) | same |
| after merge | the push run on `main`, and the next one after that | **manager**, in the AT-02 review addendum |

The queue is a property of the runner pool and the job shape, not of the event type, so three PR runs measure the thing being
fixed; the push runs exist to prove that `main` is not left with a cancelled or missing run, which is the amendment above and
can only be observed after a push. Acquisition failures are reported, not hidden: if macOS still misses the window, that is a
measured result, and the next step is the platform decision above — never `continue-on-error`.

**The rerun question, closed by the manager on 2026-11-02 — with the cause still unknown, which is the honest outcome.**
`gh run rerun 41444874714 --failed` answers *"run 41444874714 cannot be rerun; its workflow file may be broken"*. That phrase
is **gh's own wrapper text for any failed API call**, not something GitHub returned: calling the endpoint directly gives

```text
POST /repos/RUdolf1517/auto-tzim/actions/runs/41444874714/rerun-failed-jobs
{"message": "Server Error"}          HTTP 502
```

So the "broken workflow file" theory has no evidence behind it, the workflow file **does** exist at that head SHA
(`.github/workflows/ci.yml`, blob `78886f8`), and the branch still exists. Conclusion to record: a rerun on that run is not
available for an unidentified server-side reason, `gh`'s message is misleading and must not be quoted as the cause, and
**nothing in this project depends on a rerun** — every piece of evidence above is produced by new runs. If a rerun is ever
needed in a real emergency, the fallback is a new commit, not a retried job.

## D-52 · What makes two plans the same plan

**Status: adopted (manager default, 2026-11-02).** AT-03 has to produce a plan with an id and a hash (D-20) and assert that
two scans of an unchanged tree are byte-identical (AC-1). Four things were never written down, and all four are the kind a
coder answers by accident: what an id looks like, what exactly is hashed, in what order, and how deep a walk may go.

| Question | Answer |
| --- | --- |
| **Plan id** | 32 lowercase hex characters — 128 bits read from `/dev/urandom` at plan creation. Never derived from the tree, never reused, and not sortable: a plan is an event, not a sequence |
| **Plan hash** | SHA-256 over the canonical serialisation described below. Excludes the plan id, every timestamp, and the config hash; includes only what a comparison must react to |
| **Order** | Discovery walks each directory in **byte-wise sorted** order of the entry name, and rows are sorted byte-wise by the relative POSIX path. Not locale, not case-insensitive: two machines must agree on an unchanged tree |
| **Depth** | **64 levels** below the root. Level 65 is refused with `depth_exceeded`, recorded **once per directory**, not once per file. Corpus `K-14` builds a 65-deep tree and expects exactly this |
| **Paths** | Relative, POSIX separators, no leading `./`. Kept as bytes (`OsStr`); the row carries `path` (lossy, for display) and `path_raw_b64` (exact — the corpus already names it that way) so a non-UTF-8 name is representable without panicking (corpus `N-17`) |
| **Excluded names** | `.tzim-<pid>-<nanos>-<attempt>.tmp` — matched as prefix `.tzim-` and suffix `.tmp`, at any depth, **even when** `include_hidden = true` (corpus `K-12`) |

**The canonical row, and why it is exactly this.** One line of JSON per candidate, keys in a fixed order, no whitespace:

```text
{"path_raw_b64":"<base64 of the relative path bytes>","size":12345,"dev":…,"ino":…,"mtime_ns":…,"argv":["--zopfli","--", "x.png"]}
```

The hash is taken over the concatenation of those lines, in the sorted order, with `\n` after each — nothing else. So it
changes when a file is added, removed, renamed, resized, touched, or when the argv a run would use changes; and it does
**not** change when the plan id, the clock, the mode, the config path or the order of discovery attempts differ.

| In the hash | Deliberately out |
| --- | --- |
| relative path bytes, size, `dev`, `ino`, `mtime_ns`, the exact argv | plan id, created-at, run id, mode, config hash, `tzim` version |

**Why the identity tuple is in it.** A file that is touched between plan and install is `skipped_changed` at the per-file
check (AC-16), but a file that is *replaced* with the same name and size would otherwise leave the plan looking valid. Hashing
the inode and mtime makes that a hash mismatch, which is exit `6` before anything is installed — the cheapest place to
refuse.

**Why SHA-256 is written in the core instead of pulled in.** A hash is one of the few things where a hand implementation is
defensible: it is deterministic, it is compared against published FIPS 180-4 vectors in a test, and it has no key material to
leak. The alternative is `sha2` and its transitive tree, an allowlist entry and a review, for eighty lines we can test
exhaustively. The test **must** assert the empty-input and `abc` vectors and a 1 MiB input against a recorded digest; if a
dependency becomes preferable later, that is a decision, not a refactor.

**Reversal.** If a second hash is ever needed (the eligibility cache, D-06), the same module serves it, and swapping in
`sha2` is a one-file change behind the same trait.

## D-53 · File locking: bump the toolchain, not the trust boundary

**Status: adopted (manager default, 2026-11-02).** The coder asked the right question and — worth saying out loud — did not
quietly break a rule to keep moving. Three options were put on the table; one of them is now the rule.

**The problem, stated exactly.** `auto-tzim` must hold a lock for the read-modify-write of `config.toml`, and later for a run
([state](../state.md) §5). Rust **1.82** has no file locking in `std`. A plain lock file — `O_EXCL` create, delete on exit —
survives `SIGKILL`, and then every later write is blocked until a human deletes a file, which for a `folder add` is a bad
trade. Two processes must never be inside that critical section at once; that is the whole requirement, and it is not
negotiable because the alternative is a lost folder entry.

**Verified before deciding** ([Rust RELEASES.md](https://github.com/rust-lang/rust/blob/master/RELEASES.md)): `File::lock`,
`lock_shared`, `try_lock`, `try_lock_shared` and `unlock` were stabilised in **1.89.0 (2025-08-07)**, and on Unix `std`
implements them with `flock`, the same primitive `fs2`/`fs4` wrap. So the choice is not "lock or no lock"; it is *which layer
owns the primitive*.

| Option | Verdict | Why |
| --- | --- | --- |
| **1. `unsafe` FFI `flock`** | **not now** | It works, and it is genuinely small — but it puts the one thing this project claims never to have into the codebase for a capability `std` ships. Every later reviewer would have to re-verify it, and the safety argument for a tool that deletes originals is the product |
| **2. MSRV → 1.89, `std` locking** | **adopted** | No `unsafe`, no dependency, no C compilation, kernel-owned semantics: the lock dies with the process, so `SIGKILL` needs no stale-file ritual. The bump is free **today** — nothing is published, no user has an old toolchain in a lockfile — and it will never be this cheap again |
| **3. A wrapper crate** (`fs2`/`fs4`) | **refused** | A dependency, an allowlist entry, a review and a transitive tree, to wrap a call `std` makes. `fs2` has had no release since 2018; the crate that needs a crate is the crate that did not read the release notes |

**What moves, in one commit:**

1. `rust-toolchain.toml` → `1.89.0`; both the `dtolnay/rust-toolchain` steps in `ci.yml` → `1.89.0`;
2. `rust-version = "1.89"` in **both** manifests (the field is already required by F-18's metadata check);
3. the README line that names the toolchain.

D-40's principle is untouched — *the MSRV is the toolchain CI actually runs* — only the number moved, and it moved because a
standard-library capability arrived. The next bump has the same test: a concrete API or a concrete fix, never "newest".

**One consequence to expect.** 1.89 enables the `mismatched_lifetime_syntaxes` lint by default, and CI runs
`cargo clippy -- -D warnings`. Code that compiled quietly on 1.82 may now warn. Fix the warnings; do **not** add
`#[allow(...)]` at module or crate level to get a green run — a blanket allow here is the same class of shortcut as
`continue-on-error` ([D-51](DECISIONS.md)).

**The lock design, binding ([state](../state.md) §5):**

| Rule | Detail |
| --- | --- |
| Primitive | `File::try_lock()` for acquisition (fail fast), `unlock()` or the file closing for release |
| Contention | exit `8`, message names the holder read from the file (`pid`, process start time, host); nothing is processed |
| The file | `locks/config.lock` (AT-02) and `locks/run.lock` (AT-05), created `0600`, **never deleted** — deleting a lock file another process holds is the classic race, and the kernel handles release for us |
| Contents | written after acquiring, for diagnostics only: `{"pid":…,"start_time":…,"host":…}`. The kernel is the authority; the contents are for the human reading exit `8` |
| Stale locks | **there are none.** A dead holder means a released lock, so the old "report it stale, never remove it" ritual is retired — the reason it existed was that a lock file could outlive its owner |
| Scope | `config.lock` guards the read-modify-write of the config; taken before the read, released after the atomic write. The run lock reuses the same helper in AT-05 |

**The controls that make this real** (they go in the AT-02 handoff, §4):

1. `SIGKILL` a process holding the lock, then acquire it in a second process — must succeed **immediately**, which is the whole
   reason for this decision;
2. two processes race for the lock 100 times, each incrementing a counter inside the critical section — the counter must equal
   100 and no increment may be lost;
3. a lock **file** left by a dead process with no lock held — acquisition succeeds without asking a human;
4. a live holder — the second process exits `8` and names the pid.

**Reversal.** If 1.89 ever becomes unacceptable — a user on an older compiler who cannot upgrade, a distribution constraint —
the fallback is option 1, in **one** module, with a written safety contract: `flock(LOCK_EX|LOCK_NB)` through `libc`,
`cfg(unix)` only, no other `unsafe` anywhere, and the same four controls above. Option 3 stays refused.

## D-54 · AT-02's grey areas, decided

**Status: adopted (manager default, 2026-11-03).** Seven questions came up while the code was being written. Each was worth
asking; each is now a rule, so the next coder does not ask again.

**1. An explicit `mode = "auto"` is valid.** `config.md` says a *default* of `auto` is refused, and the implementation read
that as "only `confirm` is accepted". It is not the same sentence. The default value of the field is `confirm`, so a config
that never mentions `mode` cannot silently install; a user who writes `mode = "auto"` has made the choice in the file, which
is exactly what D-29 allows, and `--auto` on the command line still overrides per run. The validator accepts both values and
rejects everything else.

**2. A non-empty `tzim_path` must be real.** Absolute, not a directory, **exists**, **is a regular file**, **is executable** —
otherwise exit `2` (`tzim_path_missing`, `tzim_path_not_executable`). Empty means "find it on `PATH`", which is resolved at
run time by the adapter (AT-04), not by `config validate`. Rationale: the user who names a path is naming a specific binary;
a path that is not there is a typo, and `validate` exists to catch typos before a run rather than after one.

**3. `--allow-cloud-sync` and `--no-allow-cloud-sync` exist** on `folder add` and `folder edit`; the pair sets the field.
D-44's escape hatch has to be reachable from the command line, otherwise it is a documented flag that only people who edit
TOML by hand can use — and the whole point of `folder add` is not editing TOML by hand.

**4. The permissions control is accepted as it is, plus one test.** The control removes the explicit `set_permissions` **and**
the safe creation modes and sees `0666`, which proves the *protection as a whole* fails when both layers go. That is the
right control: making a file `0666` for a moment to prove a single `chmod` is worse than the gap it closes. The gap is
closed from the other side with a **repair test**: take a config that is already `0666`, run any mutating command, and assert
it ends `0600`. Together: layer one is proven by the negative control, layer two by the repair test.

**5. Lock metadata is diagnostics, and is labelled as such.** `started_at` is the process entry time (not an OS-reported
creation time), and `host` is `HOSTNAME` when set and **omitted** when not — never the string `"unknown"`, which reads like a
host called unknown. No dependency is added for OS-reported values: since D-53 the kernel owns the lock, nothing is decided
from this file, and a dependency for cosmetics fails the test in [D-38](DECISIONS.md).

**6. Stale temp cleanup is allowed — because the lock is held.** Every writer takes `config.lock`, so any
`.config.toml.<ts>.<n>.tmp` present while the lock is held belongs to a dead process. Cleanup happens **after** a successful
rename, only in the config directory, only for names matching our exact pattern, never recursively. Without the lock this
would be the unsafe deletion the coder correctly refused to write.

**7. `config validate` takes no lock and creates nothing (F-21).** `cli.md` §2.2 promises "without touching state", and taking
the lock creates `$AUTO_TZIM_HOME/locks/` — a promise broken for a command whose entire purpose is to inspect. It is also
unnecessary: config writes are atomic renames, so a reader sees the old file or the new one, never a half-written one. Reads
take no lock; only read-modify-write does. The same applies to `folder list` and `folder show`.

**Reversal:** items 1–3 are user-visible and cheap to change; 4–7 are internal and cheaper. None of them weakens a safety
property, which is the test they were all held to.

## D-55 · The macOS runner: measure, do not switch

**Status: adopted (manager default, 2026-11-03).** After D-51 the macOS job is acquired in seconds when it is acquired at
all — 2 s, 3 s, 4 s on the three qualifying runs — and it still fails to be acquired on some runs at all (15-minute
cancellations). The mitigation removed the self-inflicted part of the problem (four simultaneous requests); it cannot remove
GitHub's capacity.

| Rule | Detail |
| --- | --- |
| Measuring `macos-15-intel` is **allowed** | as a diagnostic, with numbers, clearly labelled as a different architecture |
| Switching to it is **a decision**, not an experiment | it changes the platform the project claims to support; it needs D-53-class reasoning and the owner's word |
| A red run because a runner was never acquired is **`not-run`** | never `passed`, and never a reason to add `continue-on-error` ([D-51](DECISIONS.md)) |
| AT-02's merge does **not** wait on capacity | it waits on a green run that exists: `41900805601` is green on both platforms at `a4e9b1e`. Unacquired runs are recorded as misses, and R-27 stays open |

The honest summary for the log: the change reduced the demand from four macOS slots per push to one, and the remaining misses
are GitHub's arm64 pool. If intel turns out to be reliable, the decision to switch will be made on those numbers.

## D-56 · Two kinds of validation, because a disk can be unplugged

**Status: adopted (manager default, 2026-11-04).** Amends [D-54](DECISIONS.md) item 2.

`load()` calls `validate_config()`, and `validate_config()` calls `validate_root()`, which asks the filesystem whether the
root exists, is a directory and is not a symlink. `save()` does the same before writing. So a profile whose disk is not
mounted makes **every** command exit `2` — `folder list`, `folder show`, `folder disable`, `folder remove`, `config
validate`. The user cannot even delete the profile that is in the way; the only repair left is hand-editing TOML, in a tool
whose purpose is to avoid exactly that. Same shape for `tzim_path`: uninstall `tzim`, and the configuration becomes
unreadable.

The mistake is mine, not the coder's: [D-54](DECISIONS.md) item 2 said `config validate` must check that the path exists and
is executable, and the implementation put that check where the rest of validation lives. The rule was right and the
placement was wrong.

**The split.** Two questions, asked at different times, by different commands:

| Kind | What it asks | Examples | Asked by |
| --- | --- | --- | --- |
| **Structure** | is this a well-formed configuration? | schema, `mode`/`update_policy` enums, name rules, duplicate name and root, absolute path, empty root, `forbidden_root`, `state_inside_root`, `cloud_sync_root` without the flag, `not_in_v01`, ranges, format names, `tzim_path` shape | `load()` and `save()` — always |
| **Environment** | is the world still the way the file says? | root exists, is a directory, is not a symlink, canonicalises; `tzim_path` exists, is a regular file, is executable | `config validate` (all folders), `folder add` (the folders being added), and `run` (the folder being run) |

Both columns are string or metadata facts about *this machine right now* in the second column, and facts about *the document*
in the first. Only the first is a reason to refuse to read the file.

**Consequences, stated as promises:**

- a config whose root is on an unmounted disk still **loads**; `folder list` and `folder show` print it; `folder edit`,
  `folder enable`, `folder disable` and `folder remove` work;
- `config validate` **is** the command that reports reality: it exits `2` with `root_missing` for that folder, and that is
  the intended way to find out;
- `folder add` still refuses a root that does not exist, is a file, or is a symlink — the user is naming it *now*;
- a corrupt configuration is still never overwritten: corruption is a *structure* failure, and structure is still checked on
  every save.

**Why not warn instead of error on load:** a warning printed by `folder list` is read by nobody, and a program that prints
warnings while refusing nothing teaches its users to ignore output. The split keeps every fact checkable — in the command
whose name is `validate`.

**Reversal:** if it turns out users expect `folder list` to fail loudly on a broken root, the alternative is a `--strict`
flag on the read-only commands, not a return to the trap.

**"Structure" means no requirement on the environment, not no syscalls** (F-27). `validate_root_structure` resolves existing
ancestors through `canonicalize_with_missing` so two names for the same directory are still one folder, and falls back to a
lexical form when the disk is absent — that is what keeps an unplugged root from failing. A later reader should not assume
"structural" is I/O-free, and should not put anything expensive behind it.

## D-57 · One folder, one plan — even when the command names five

**Status: adopted (manager default, 2026-11-06).** The owner asked the obvious question about the AT-03 brief: *"почему обход
папки? их же может быть несколько"*. They can, and the brief said otherwise — `HANDOFF-AT-03.md` was written as "`lstat` on
every entry of one approved root", and `cli.md` §2.5 was headed `scan [PATH | --folder NAME]`. [D-49](DECISIONS.md) had
already answered this for `run`; the read-only path never got the same treatment, and a coder reading AT-03 would have built a
single-root tool.

**The rules, all of them inherited from [D-49](DECISIONS.md) §0.2 and made explicit for the plan:**

| Question | Answer |
| --- | --- |
| Does `scan` take several paths? | **yes** — `auto-tzim a b c --dry-run`, and `--folder` may be repeated. One path is the common case, not the only one |
| When is the batch validated? | **before any directory is read.** Every path: forbidden root, symlink, cloud sync, `AUTO_TZIM_HOME` containment, overlap. One refusal ⇒ exit `2`, the message names the path, **nothing is scanned anywhere** |
| Is a plan file written for the good folders when one path is refused? | **no.** "Nothing is processed anywhere" includes not leaving half a batch of plan files that authorise a later apply |
| Two paths, one inside the other | exit `2`, `overlapping_roots`, both named — a file in both would be planned twice |
| The same root twice | de-duplicated on the canonical path, planned **once**, printed once |
| Order of output | the order typed on the command line. Rows inside a plan stay byte-wise sorted ([D-52](DECISIONS.md)); sorting the *folders* would hide which one the user named first |
| How many plans? | **one per folder** — own plan id, own hash, own `plans/<plan_id>.json`, each recording its canonical root |
| Why not one plan for the batch? | `--plan ID` binds **one** folder (`cli.md` §2.6); a batch-wide id could not authorise a per-folder apply, and D-52's hash is defined over one tree's rows — mixing two trees would make the hash change when the *batch* changed, which is exactly what D-52 says a hash must not react to |
| What does `--json` do | not in AT-03 (AT-06). Human text only, and it prints each folder's rows under that folder's heading |
| Exit code | `0` when every folder produced a plan, `2` on any validation refusal. A folder with zero candidates is still a plan, still `0` |

**What this does not change:** the walk itself is unchanged — `lstat`, never `stat`, never follow a link, 64 levels, `.tzim-*…*.tmp`
excluded everywhere, rows sorted byte-wise, no `tzim` spawned. A batch is that walk repeated over each root, with the
validation done once, up front.

**Reversal:** if a later stage needs a batch-wide record, it is an addition (`batch_id` on each plan file, as `state.md` §3.1
already does for runs), not a merge of the plans.

## D-58 · A folder means the folder and everything in it

**Status: adopted (manager default, 2026-11-06).** Amends the `recursive` default in [config](../config.md) §3.

The owner asked whether a folder is walked recursively, and the honest answer was *no, unless you pass `--recursive`*. That is
the wrong default for the tool the project is, and the failure it causes is quiet:

```sh
auto-tzim ~/Downloads --auto          # 41 images in the top level, 1 300 in subfolders
```

With `recursive = false` that run plans the 41 and prints a summary. The 1 300 files are not skipped, not refused, not
mentioned — they were never looked at. The user reads "done", sees a disk that did not shrink, and concludes the program does
not work. Every other refusal in this project is loud; this one was silent, which makes it the worst kind.

The scope question is really *"what did I point at?"*. When somebody types `~/Downloads` they mean the folder they can see in
a file manager, with its subfolders. Non-recursive is the surprising reading, not the safe one — and the safety it appears to
buy is already provided by the rules that live in the walk: `.git`/`.hg`/`.svn` excluded by default, depth 64, mount points
refused, symlinks never followed, cloud-sync roots refused, plan + confirm + identity re-check before any install.

| Question | Answer |
| --- | --- |
| Default | **`recursive = true`** — the folder and everything below it |
| Top level only | `--no-recursive` on a run; `folder add --no-recursive` stores it |
| The one-shot form | `auto-tzim DIR` is recursive; `auto-tzim DIR --no-recursive` is not |
| Existing configs | a config that never mentioned `recursive` starts walking deeper after this change. That is the intended reading of "the folder", and it is said in [CHANGELOG](../../CHANGELOG.md) rather than slipped in |
| Does recursion widen what can be modified? | yes, and that is why the summary must state the scope (below), and why every other boundary stays where it is |
| Are mounted disks crossed | no — `cross_filesystems = false` is unchanged; the boundary is recorded once |
| Symlinks | never followed, at any depth ([D-45](DECISIONS.md)) |

**The summary must state the scope**, in both modes, in one line each:

```text
downloads   /home/user/Downloads   41 files at the top level, 1 300 below, 12 directories walked
```

and, when a limit did something, it says which: `3 subfolders not walked (mount point)`, `1 not walked (depth 64)`,
`14 not walked (excluded by .git/**)`. A run that plans zero candidates says why: `0 candidates — 1 341 files looked at,
none with a supported format`.

**Why not "recursive by default, and also print a warning"?** A warning is a thing people learn to scroll past. The scope line
is not a warning, it is the answer to the question the user asked by typing the path, and it is printed whether or not
anything went wrong.

**Reversal:** `recursive = false` stays a stored value, so a user who wants the old behaviour gets it exactly with
`--no-recursive` or the config field. If it turns out that people are surprised in the *other* direction — "I did not mean
that deep" — the answer is the scope line, not a silent narrower walk.

## D-59 · What a plan records, and what "byte-identical" means

**Status: adopted (manager default, 2026-11-07).** Answers three questions from PR #46 that share one subject: the bytes a plan
writes down.

**1. `argv` is the profile-owned part of a future command, and nothing else.** The coder asked whether AT-03 should put
`--json` and an absolute path in it. Neither:

| Element | AT-03 writes | Why |
| --- | --- | --- |
| profile flags, fixed order, byte-sorted values | `--formats png,gzip` when `formats` is non-empty ([D-62](DECISIONS.md)) | this is the part the *user* chose, and the part [D-11](DECISIONS.md) says is ours to pass |
| `--` | always, before the path | a name may start with `-` (corpus `N-09`); the separator is not optional |
| the file argument | the **relative POSIX path** | absolute paths would make the plan hash depend on where the tree lives, and [D-52](DECISIONS.md) exists so two machines agree on an unchanged tree |
| `--json`, `--jobs`, `--strip-metadata`, the staged path, the timeout | **nothing** | every one of them is adapter territory ([D-26](DECISIONS.md), AT-04). AT-03 must not invent the adapter contract, and the coder was right not to |

AT-04 will prepend its own flags and replace the final element with the staged copy's path. That is a **different argv**, and the
hash changes with it — which is correct: a plan made by AT-03 does not authorise an AT-04 run, and the plan will be refused with
exit `6` rather than quietly reused.

**2. A name that is not UTF-8 is carried as bytes.** `argv` in the *canonical row* is an array whose elements are JSON strings
when the bytes are valid UTF-8, and `{"b64":"<base64 of the bytes>"}` when they are not. Only the path element can be
non-UTF-8 — profile flags are validated ASCII — but the row does not need to know that. The human rendering of a plan uses the
lossy form and never feeds it back into a hash.

**3. `root` and `root_raw_b64`.** A plan records the canonical root twice: `root` for display (lossy) and `root_raw_b64` for
exactness, the same pair [D-52](DECISIONS.md) already uses for `path`. A directly supplied root may itself be non-UTF-8 on Unix,
and rejecting such a root would refuse a legal directory — corpus `N-17` exists precisely because names are bytes.

**4. "Byte-identical" cannot mean the file, and the corpus never said it did.** Acceptance 1 asks that two scans of an
unchanged tree be byte-identical, while [D-52](DECISIONS.md) requires each plan to carry its own id, and a plan also records
`created`. The three fields that *must* differ are `plan_id`, `created` and any mode/timestamp; what must be identical is the
**`rows` array** — the canonical lines — and therefore `plan_hash`. `GATES.md` AC-1 says "the two plan **bodies**" and means the
same thing. The test asserts both halves: rows equal, hashes equal, ids different.

**Reversal:** if AT-04 needs a different argv composition, it changes it and says so; the hash's job is to notice.

## D-60 · A plan is published, never replaced

**Status: adopted (manager default, 2026-11-07).** The coder's proposal, accepted because it is better than what my own
`state.md` §2 assumed.

`state.md` §2 says "atomic replacement of every small file", and §3.4 says a plan file is written once and never modified. Those
two rules collide for exactly one case: a `rename` onto an existing path **overwrites** it, so a plan id that collides would
silently destroy the plan it collided with.

**The primitive is a hard link:**

```text
write plans/<id>.json.tmp   (0600, fsync the file)
hard_link(tmp, plans/<id>.json)   → EEXIST means the id is taken
unlink(tmp)
fsync the directory
```

`fs::hard_link` is in `std`, creates the destination only if it does not exist, and fails with `EEXIST` otherwise — the
"create-if-absent" semantics that `rename` does not have. Same directory, so same filesystem, so the link always succeeds when
the name is free.

| Case | Behaviour |
| --- | --- |
| name free | published, temp removed, directory fsynced |
| name taken | `plan_id_collision`, **exit `7`**, the existing plan untouched, the temp removed |
| filesystem without hard links (`ENOTSUP`/`EPERM`) | fails loudly with the OS error; **no** fallback that could overwrite |

A 128-bit random id makes a collision a broken-randomness event, not bad luck ([D-52](DECISIONS.md)), which is why the reaction
is an internal error rather than a retry loop: retrying would hide the only symptom.

**Consequence for `state.md`:** "atomic replacement" stays the rule for every *mutable* small file (`last-compression.json`,
`run.json`), and becomes "atomic publication" for the immutable ones. The distinction is the point of the two sections.

## D-61 · The order of the gates, because a file can trip several

**Status: adopted (manager default, 2026-11-07).** The coder asked what wins when an orphan temp file also has two hard links.

A walk meets the same file with several properties at once, and "which check runs first" decides which status the row carries.
It is contract, not implementation detail, because the plan is compared byte for byte:

| # | Gate | Outcome when it fires |
| --- | --- | --- |
| 1 | **excluded name** — `.tzim-` prefix and `.tmp` suffix, at any depth, even with `include_hidden = true` | **no row**; one `orphan_temp` diagnostic per run; never deleted, never an error |
| 2 | **scope** — depth 64, mount point with `cross_filesystems = false`, above the top level with `recursive = false` | recorded once per directory (`depth_exceeded`, `mount_boundary`, `not_recursive`), not per file |
| 3 | **type** — not a regular file (directory, symlink, FIFO, socket, device) | `skipped_policy` with the kind named |
| 4 | **hard link** — `st_nlink > 1` | `hardlink_skip` |
| 5 | **age** — younger than `min_file_age_seconds` | `skipped_unstable` |
| 6 | **format** — `formats` non-empty and the extension not in it ([D-62](DECISIONS.md)) | `skipped_policy` `format_not_selected` |
| 7 | otherwise | `planned` |

So a hard-linked orphan temp is **not a row at all**: gate 1 answers before gate 4 is reached. That is the only reading
consistent with "never planned, diagnostic emitted" in corpus `K-12` — and with the rule that we never touch, delete or report an
error about a file that is not ours to touch.

Gate 4 before 5 and 6 is deliberate: two links to one inode are one file's bytes under two names, and a plan whose rows disagree
about the same bytes is worse than a smaller plan.

## D-62 · The format policy reads a name, never a file

**Status: adopted (manager default, 2026-11-07).** Resolves a real contradiction the coder found between corpus `P-02`,
[D-11](DECISIONS.md) and [D-26](DECISIONS.md) item 3.

Two different things had been written with the same words:

- **our policy** — *which files we offer to `tzim`*. The user writes `formats = ["png"]`, `config.md` validates the names, and
  the plan selects `.png` files and skips a `.gz`. Corpus `P-02` is exactly this.
- **`tzim`'s capability** — *what it can actually do*. That comes from `--capabilities` ([D-26](DECISIONS.md)) and is never
  reconstructed, inferred from a version, or guessed from an extension (invariant 18).

`D-26` item 3 said "`--formats` is still never passed", which conflated them and contradicted `P-02`. **Corrected:** the flag is
passed exactly when the profile sets `formats` ([D-11](DECISIONS.md)); what is never derived from the capability document is the
list itself.

| Situation | Plan |
| --- | --- |
| `formats = []` | every regular file passes gate 6; `tzim` decides later (D-11), and the plan claims nothing (D-23) |
| `formats = ["png"]` | only files whose extension maps to `png` pass; everything else is `skipped_policy` `format_not_selected` |

The extension table, fixed in this decision so two implementations cannot differ:

| Format name | Extensions |
| --- | --- |
| `png` | `.png` |
| `gzip` | `.gz` |
| `zip` | `.zip` |
| `ico` | `.ico` |
| `jpeg` | `.jpg`, `.jpeg` |
| `tiff` | `.tif`, `.tiff` |
| `zlib` | `.zlib` |

Matching is on the **last** suffix, ASCII case-insensitively; a file with no suffix never matches a non-empty list. **No file is
opened**, no signature is read, and a file whose contents disagree with its name is still selected — because the plan is not a
promise ([D-23](DECISIONS.md)), and `tzim` will report `unsupported` if it disagrees.

**Why this is not "guessing from an extension" (invariant 18):** we are not claiming what `tzim` can do, we are applying the
user's own selection rule. The invariant forbids inferring *capabilities and reasons*; `formats` is a filter the user typed.

**Reversal:** if a workload needs content sniffing, it is a new decision with a corpus entry and a cost, not a quiet `magic`
crate.

## D-63 · The glob grammar is three wildcards and no escaper

**Status: adopted (manager default, 2026-11-07).** `config.md` said "`**` spans directories, no brace expansion, no negation" and
stopped there; the coder asked what `[...]` and `\` do, and what "malformed" means.

| Token | Meaning |
| --- | --- |
| `*` | any run of bytes **except** `/` |
| `**` | any run of bytes **including** `/` |
| `?` | exactly one byte, **not** `/` |
| everything else | literal bytes, including `[`, `]`, `\`, `{`, `}`, `!` |

There are no character classes, no escapes and no negation in v0.1: they are the part of glob syntax that turns into a
specification of its own (ranges, collation, escaping a metacharacter) and then into a compatibility promise. A pattern that looks
malformed under some other glob dialect is a **literal** here, which is why nothing is rejected at match time.

The one refusal: an **empty** pattern (`exclude_globs = [""]`) is a configuration error at load — `exclude_globs: empty pattern`
— because "" matches nothing and silently excludes nothing, and the user meant something by it. Matching is over the **relative
POSIX path** as bytes, so a pattern can express a non-UTF-8 name only if it is written as bytes, which TOML does not allow —
acceptable, and stated.

## D-64 · The directory race: what is guaranteed, and what is not

**Status: adopted (manager default, 2026-11-07).** The coder is right that `std` cannot make the traversal atomic, and right to
refuse to claim otherwise.

**The window.** We `lstat` a path, decide it is a directory, then `read_dir` it. Between those two calls another process can
replace the directory with a symlink to somewhere else, and our enumeration follows it. Closing the window needs
`openat(O_NOFOLLOW | O_DIRECTORY)` + `fstatat` + `fdopendir`, which is `libc` and, through it, `unsafe` — refused by
[D-35](DECISIONS.md)/[D-38](DECISIONS.md) and [D-53](DECISIONS.md), or a crate that wraps them, refused for the same reason.

**What is guaranteed, and is what the product promises:**

| Claim | Mechanism |
| --- | --- |
| No published plan contains a row outside the canonical root | each candidate's parent is canonicalised and compared with the root before the row is emitted; a violation is a **hard abort** (`tree_changed`, exit `6`), never a skip ([HANDOFF-AT-03](HANDOFF-AT-03.md) §2) |
| A swap is detected rather than ignored | `(st_dev, st_ino)` of each directory is captured before enumeration and compared after; a mismatch aborts the whole scan with `tree_changed` |
| Nothing in the user's folder is written, ever | there is no write path in this stage at all — this is the reason the residual risk is a refused run, not a damaged tree |
| A file that changed between plan and install is not installed | the per-file identity re-check at install time ([D-20](DECISIONS.md)), which is AT-05's job and does not depend on this window |

**What is not guaranteed, and is written in [limitations](../limitations.md) §4:** that we never `lstat` a name outside the root
during a concurrent swap, and that no diagnostic ever mentions such a name. An attacker who can replace a directory inside the
user's own tree, with the user's own privileges, is outside what a read-only planner can defend against; the honest response is a
refused run, and that is what happens.

**Testability without a race.** The re-check must be reachable through an **injected seam** — a hook called after enumeration and
before the comparison — so a test performs the swap deterministically instead of racing, the same discipline [D-14](DECISIONS.md)
applies to the clock. A test that sleeps and hopes is a flaky test with a slow half-life.

## D-65 · The 10,000-entry budget, written down at last

**Status: adopted (manager default, 2026-11-07).** Acceptance point 10 says "within the documented budget" and no document had a
number — a requirement that cannot fail is not a check.

| Property | Value |
| --- | --- |
| Shape | one directory, 10 000 regular files, no subdirectories |
| Build | debug (`cargo test`), because that is what CI runs |
| Platform | `ubuntu-24.04`, the CI runner |
| Measured | walk + policy + plan rows; **fixture creation excluded** |
| Ceiling | **15 s** — a smoke bound, roughly an order of magnitude above the expected debug time |
| Evidence | the test prints the observed duration; the PR quotes it |

**Why a loose bound and not a target.** A tight number on shared CI hardware turns runner load into a red build, and a test that
fails for reasons unrelated to the code teaches people to re-run instead of read. The number exists to catch an accidental
quadratic — a per-file canonicalise of the whole root, a re-read of the directory per row — which is exactly the failure this
stage can have. If a real budget is ever needed, it belongs in a benchmark with a recorded baseline, not in a unit test.
