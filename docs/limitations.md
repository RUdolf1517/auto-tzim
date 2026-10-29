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

- **A scan writes nothing; an apply does.** `auto-tzim scan` (and `run --dry-run`, which is the same operation) never
  runs file-processing `tzim` and never needs write access to the folder. An **apply** lets `tzim` write a candidate
  `.tzim-<pid>-<ns>-<attempt>.tmp` in the destination directory and rename it; after a hard kill an orphan can remain, we
  report it and never delete it (R-13).
- **A scan knows no numbers.** The plan lists candidates, not predicted savings, and is not a promise that `tzim` will
  change anything (D-23).
- **One file at a time.** `jobs = 1` in v0.1; parallelism inside a file is `tzim`'s own.
- **No prompts, ever.** A command that needs a decision requires a flag; in a scheduler or a pipe nothing will ask.
- **A refusal is not an error, and it is not typed.** `skipped_by_tzim` and `skipped_unsupported` exit `0`, because
  nothing went wrong; the reason survives as `tzim`'s own note, not as a status we invented.
- **Exit codes are ours.** `6` means "refused before processing" (no plan yet, paused, low disk), `8` means another run
  holds the lock, `7` means an internal error, and `4` is never produced.
- **`failed` is not retried automatically.** A second run does not try again until the file, the binary or the policy
  changes; `history` shows why.
- **The first run of a new folder does nothing but preview.**
- **Absolute paths are recorded** in history and, on request, on stdout; they are local data and should not be pasted
  into a public issue without thought.

## 5. Honest gaps we inherited from `tzim`

- No machine-readable capability list: we record version and binary hash, delegate the default format set to the binary,
  and never display a set we cannot observe (study G-1, charter §6-18).
- Refusal reasons are prose: every `tzim`-decided skip arrives as one generic `skipped_by_tzim` with the note preserved
  verbatim and never interpreted (study G-2). The upstream request `RUdolf1517/tzim#85` describes the interface that would
  change this; nothing here waits for it.
- The `fits-tile` transformation cannot be tested against release binaries, so its opt-in is forbidden in the MVP (D-05, charter §6-11).

## 6. What v0.1 does not have

No scheduler, no TUI, no GUI, no automatic update, no backup, no cloud sync, no telemetry, no Windows, no
multi-user/multi-install coordination (one `AUTO_TZIM_HOME` per user), no remote folders, no file syncing, no
reporting dashboard. Each is either a later stage in the [roadmap](roadmap.md) or explicitly refused.
