# Guarantees

> **Status: specification, no implementation.** These are the promises the first pull request will be reviewed
> against; nothing below is currently observable because there is no program.
> Related: [safety](safety.md) · [formats](formats.md) · [limitations](limitations.md)

`tzim` guarantees things about file bytes: G1 (byte-identical), G3 (format-equivalent), G4 (metadata removed on
request). `auto-tzim` guarantees things about **decisions and records**. Keeping the two sets separate is the point of
this page: a reader must never think that `auto-tzim` adds a verification layer of its own, because it does not and
must not.

## 1. What `auto-tzim` guarantees (A-class)

| ID | Guarantee | Meaning | Proof |
| --- | --- | --- | --- |
| **A1** | Nothing outside the plan | Only paths present in an approved, printed plan are ever passed to `tzim`; every path is inside the canonical root | INV-7, AC-1, AC-3 |
| **A2** | No byte changed by us | `auto-tzim` writes only under `AUTO_TZIM_HOME` (config, SQLite, logs, update cache). It never writes, renames, deletes or chmods a user file | INV-8, source review of every write site |
| **A3** | A refusal stays refused | An outcome `tzim` declined, or our policy excluded, is recorded and finished. No fallback, no second handler, no "try the other format" | INV-9, INV-10 |
| **A4** | Identity is checked, not assumed | A file is only handed over while its `(dev, ino, size, mtime_ns)` matches the captured identity, and the identity is re-read after the process exits | INV-6, R-03, AC-4 |
| **A5** | The record is honest | Every row states what happened, by whom (our layer or `tzim`), with the raw child result preserved; a dry run is never recorded as a completed optimization | AC-8, AC-10 |
| **A6** | Local only | No telemetry; no network call except the single documented `update-check` request | INV-15, [privacy](privacy.md) |
| **A7** | No self-repair of the state | A corrupt config or database is reported, not silently rewritten, because a silent reset erases the evidence a user needs | AC-7, [recovery](recovery.md) |

## 2. What `auto-tzim` does not guarantee

- **That a file shrinks.** Only `tzim` decides, and it may legitimately do nothing (already optimal, refused,
  unsupported).
- **That an optimization is reversible.** There is no undo, no backup, no trash. A success rewrites the file. This is
  stated in `--help`, in the README and in [limitations](limitations.md) (invariant 16).
- **That the directory is untouched by a dry run.** `tzim`'s own dry run creates and removes a candidate in the
  destination directory; see [P-02](manager/problems/P-02-dry-run-is-not-write-free.md).
- **That a refusal is typed precisely.** `tzim` reports refusals as `skipped` with prose; our normalized status comes
  from our own policy layer, and the raw note is kept beside it (study G-2).
- **That the installed build has a given format.** Capabilities are build-dependent and there is no machine-readable
  list (study G-1); we record the version and the binary hash.
- **Anything about a platform we do not test.** macOS and Linux are the MVP claim; Windows and network filesystems
  are named as untested.

## 3. How a status is derived (the honest version)

A user reading `optimized` should know exactly who said it:

1. `optimized` — `tzim` exited `0` with `status: "optimized"`, `before > after`, and the file's identity changed in the
   way a replacement implies. We record the before/after sizes and the `tzim` row verbatim.
2. `already_optimal` — `tzim` exited `0` with that status; the bytes are unchanged, and the identity is unchanged
   (which is what makes a cache entry legitimate).
3. `skipped_refused` / `skipped_unsupported` — `tzim` exited `0` and reported `skipped` / `unsupported`; the prose
   note is stored as-is, and no interpretation is added.
4. `skipped_policy` / `hardlink_skip` / `skipped_unstable` — **our** decision, taken before any process started; the
   row names the rule that fired.
5. `failed` — either `tzim` reported a failure (with its `kind` and `message`), or our timeout fired, or the output
   was malformed. The distinction is in the row; a reader never has to guess.
6. `interrupted` — the run stopped before this file finished, or a crash orphaned it.

## 4. What a green test suite would have to show

Per the evidence standard, these are the assertions a reviewer will demand, not a list of good intentions:

- every non-optimizing outcome has a before/after SHA-256 proving the bytes are identical (AC-4);
- every failure row's `kind` matches what the child actually reported, including `verify` for `tzim` exit `4`;
- a second run over an unchanged corpus spawns zero processes (AC-6);
- a dry run leaves no extra directory entry and writes no `optimized` row (AC-2, AC-10);
- `run`/`scan` open no socket; `update-check` opens exactly one (AC-9);
- documentation defaults equal schema defaults by reflection (AC-10).

## 5. Why there is no G2-like guarantee here

`tzim` documents G2 (exactly reversible) as unused, because reversibility requires a container. `auto-tzim` cannot
manufacture reversibility that `tzim` declines to provide, and offering a "restore" would be a promise the program
cannot keep after the original bytes are gone. The honest offering is a record: what was done, to which file, with
which binary, at what time — and a documented statement that the previous bytes no longer exist.
