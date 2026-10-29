# Limitations

> **Status: specification, no implementation.** Everything here will be true of v0.1; a README that omits any of it
> fails acceptance (AC-10).
> Related: [guarantees](guarantees.md) · [safety](safety.md) · [recovery](recovery.md)

A limitations list is part of the product. Each item below is a deliberate boundary, not a bug queued for later.

## 1. Irreversible by design

- **There is no undo.** A successful optimization rewrites the file; the previous bytes are gone. Nothing is copied to
  a trash folder, no backup is made, and `auto-tzim` will not grow a "restore" command without a design that can
  actually keep the promise.
- Backup software (Time Machine and similar) may hold an older copy. That is the user's arrangement, not our guarantee.

## 2. Platform

- **macOS and Linux only.** Windows is not supported in v0.1: the path, locking and permission models differ enough
  that an untested port would be a false claim.
- Tested on the platforms CI actually runs; network filesystems (SMB/NFS), FUSE mounts and external exFAT volumes are
  **untested** and may behave differently around `fsync` and atomic `rename`.
- File systems without reliable atomic rename are out of scope; the replacement guarantee belongs to `tzim` and is
  stated for local file systems.

## 3. What will not be optimized

- Hard-linked files: skipped by default; a verified sibling-safety contract does not exist.
- Symlinks and special files: skipped, never followed.
- Cloud-sync roots: rejected unless explicitly allowed, and the detection is best-effort.
- Hidden files: excluded by default.
- Anything `tzim` refuses: progressive JPEG, LZW/tiled/BigTIFF, signed/ZIP64/encrypted archives, preset-dictionary
  zlib, and every format `tzim` declines by measurement (PDF, FLAC, WebP, AVIF, MP4, GIF, xz/zstd/7z).

## 4. Behaviour that surprises people

- **A dry run writes.** `tzim`'s dry run builds and deletes a real candidate in the destination directory: the folder
  must be writable, and free space is needed. After a hard kill, an orphaned `.tzim-*.tmp` may remain; we report it
  and never delete it ([P-02](manager/problems/P-02-dry-run-is-not-write-free.md)).
- **One file at a time.** `jobs = 1` in v0.1; parallelism inside a file is `tzim`'s own.
- **No prompts, ever.** A command that needs a decision requires a flag; in a scheduler or a pipe nothing will ask.
- **A refusal is not an error.** `skipped_refused` and `skipped_unsupported` exit `0`, because nothing went wrong.
- **`failed` is not retried automatically.** A second run does not try again until the file, the binary or the policy
  changes; `history` shows why.
- **The first run of a new folder does nothing but preview.**
- **Absolute paths are recorded** in history and, on request, on stdout; they are local data and should not be pasted
  into a public issue without thought.

## 5. Honest gaps we inherited from `tzim`

- No machine-readable capability list: we record version and binary hash instead of a real capability statement, and a
  format that exists in a custom build is invisible to us until it is used (study G-1).
- Refusal reasons are prose: our normalized status comes from our own policy layer, and the raw note is preserved but
  not interpreted (study G-2).
- The `fits-tile` transformation cannot be tested against release binaries, so its opt-in is disabled (proposal D-05).

## 6. What v0.1 does not have

No scheduler, no TUI, no GUI, no automatic update, no backup, no cloud sync, no telemetry, no Windows, no
multi-user/multi-install coordination (one `AUTO_TZIM_HOME` per user), no remote folders, no file syncing, no
reporting dashboard. Each is either a later stage in the [roadmap](roadmap.md) or explicitly refused.
