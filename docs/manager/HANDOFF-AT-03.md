# Brief — AT-03: what is in the folder, and what never leaves it

Written 2026-11-02 at `main` = `6c55db2`. Coder's brief for **AT-03 only**, in the same form as
[HANDOFF-AT-01](HANDOFF-AT-01.md) and [HANDOFF-AT-02](HANDOFF-AT-02.md). It is not the handoff
[format](HANDOFF-TEMPLATE.md) and does not replace it.

**Status: queued behind AT-02.** Do not start until the AT-02 review says PASS — a discovery layer built on an unaccepted
config schema is built on sand. Read [CODER-QUEUE](CODER-QUEUE.md) §0 first: it is Rust, and it is 1.89.0.

## 1. What AT-03 is for

AT-02 made the tool able to describe a folder. AT-03 makes it able to **look at one**, and this is the stage where the
project's central promise is either true or false:

> A scan reads your folder and writes nothing. Nothing is invoked on your files. What it shows you is what would happen —
> and if it changes between now and then, the run refuses rather than guessing.

Everything here is read-only, and read-only is not a property of good intentions; it is a property of a walk that uses
`lstat`, never follows a link, never opens a file, and produces a list somebody else can reproduce byte for byte.

## 2. Build exactly this

| Item | Requirement |
| --- | --- |
| Where the code lives | `auto-tzim-core`: `discovery.rs` (the walk), `stability.rs` (identity and age), `planner.rs` (the plan), plus a `hash.rs` for SHA-256 ([D-52](DECISIONS.md)). No new crate, no new dependency |
| Walk | `lstat` on every entry of one approved root. **Never** `stat`, never follow a link: a symlink is a candidate for `skipped_policy`, not a door |
| Order | byte-wise sorted entry names per directory, rows sorted byte-wise by relative POSIX path. Not locale, not case-insensitive ([D-52](DECISIONS.md)) |
| Depth | **64 levels** below the root; level 65 is `depth_exceeded`, recorded once per directory, not once per file |
| Scope rules | `recursive` (top level only when false), `include_hidden`, `cross_filesystems` (do not cross a mount point when false, and record the boundary once), `exclude_globs` on the POSIX relative path, `**` spans directories, no braces, no negation |
| Always excluded | `.tzim-*…*.tmp`, at any depth, **even with `include_hidden = true`** (corpus `K-12`, R-14). Never deleted, never reported as an error |
| Regular files only | FIFOs, sockets, device nodes, directories, symlinks: `skipped_policy` with a reason, never an error, never a read. A symlink loop must not hang (corpus `K-05`, 5 s bound) |
| Identity | `(st_dev, st_ino, size, mtime_ns)` captured once at discovery via `std::os::unix::fs::MetadataExt`; `st_nlink > 1` ⇒ `hardlink_skip` **before** anything else |
| Age | younger than `min_file_age_seconds` ⇒ `skipped_unstable`. The clock is **injected** ([D-14](DECISIONS.md)): `stability.rs` takes a "now", never calls `SystemTime::now()` inside the decision |
| Containment re-check | after canonicalising each candidate's parent, assert it is still inside the canonical root. A path that escapes is a **hard abort**, not a skip (this is the `..`/swap defence, R-01/R-02) |
| The plan | an ordered list of rows + a plan id + a plan hash, exactly as [D-52](DECISIONS.md) defines them; written to `plans/<plan_id>.json` **once**, never modified |
| CLI surface | `auto-tzim PATH --dry-run` and `scan [PATH \| --folder NAME]`: print the plan, install nothing, write nothing inside the root. `--json` is AT-06 — print human text only |
| What the plan may claim | every row is `planned`, and nothing else. No "will be smaller", no "optimizable", no savings, no predicted size ([D-23](DECISIONS.md), charter §8.4) |
| Not in this stage | no `tzim` invocation **at all** — not `--version`, not `--capabilities` (those arrive with the adapter in AT-04); no staging, no runner, no install, no lock, no history rows, no cooldown |

## 3. Acceptance points

`not-run` is not `passed`, and a number the reviewer cannot recompute is not evidence.

| # | Point | Evidence required |
| --- | --- | --- |
| 1 | Two scans of an unchanged tree are **byte-identical** | the two plan files side by side, `sha256sum` equal, and the plan **id** different — the id is an event, the hash is the content |
| 2 | The plan is deterministic against a hostile enum order | a test that forces the directory-read order backwards (or a fake walker) and produces the same rows; state how the order was forced |
| 3 | Nothing outside the root is ever planned | a root with `../escape`, a symlinked directory pointing out, and a symlink to `/etc/hostname`: no row outside the canonical root, in any of them |
| 4 | The scan writes nothing | before/after listing **and** SHA-256 of every file in the root; a second run with the root made read-only must still succeed (`chmod -R a-w`, then restore) |
| 5 | No `tzim` is spawned | the invocation counter is **zero of both kinds** (flag-only and path) — there is no adapter yet, so any spawn is a defect; evidence is the counter, not the absence of an error |
| 6 | Hostile names are data | corpus `N-01`…`N-17`: the plan contains each name once, correctly, and `N-05`/`N-06` leave no `/tmp/ats_pwned*` behind |
| 7 | Special files are skipped, not read | corpus `K-01`…`K-08`: each row `skipped_policy` with the right reason; the FIFO case completes without hanging |
| 8 | Hard links and age | `K-09`/`K-13`: both links `hardlink_skip`; `S-03`: a file younger than the threshold is `skipped_unstable`, and becomes eligible when the injected clock advances |
| 9 | Orphan temp files | `K-12`: never planned, diagnostic emitted once, file left untouched |
| 10 | Depth and size | `K-14`: 65-deep tree refused at the limit, no stack overflow, reason recorded; a 10 000-entry directory scans within the documented budget |
| 11 | The plan hash reacts to the right things | touch one file ⇒ different hash; change one argv ⇒ different hash; run twice untouched ⇒ same hash; the plan id, the clock and the mode must **not** change the hash ([D-52](DECISIONS.md)) |
| 12 | The hash itself is right | SHA-256 test vectors from FIPS 180-4: empty input, `abc`, and one 1 MiB input with a recorded digest |
| 13 | Nothing is claimed | grep the plan output for `save`, `smaller`, `optimiz`, `%` — no row and no summary may contain a predicted outcome (D-23) |

## 4. The negative controls are the deliverable

Five of the points above are only worth something if you show them going **red** on purpose. Each in its own commit, each
with the run id, then reverted:

1. **containment** — disable the canonical re-check; the `../escape` case must go green when it must be red;
2. **no-follow** — use `stat` instead of `lstat` in the walk; the symlinked-directory case must start planning outside the root;
3. **temp-file exclusion** — drop the `.tzim-`/`.tmp` filter; `K-12` must appear in the plan;
4. **determinism** — replace the sort with the raw `read_dir` order; the forced-reverse-order test must fail;
5. **hash content** — drop `mtime_ns` from the hashed row; the "touch one file" case must stop changing the hash.

The `N-05`/`N-06` cases are a control of a different kind: they fail if a filename is ever handed to a shell. If you write
no shell, they pass by construction — say so in the handoff rather than claiming a test proved it.

## 5. Traps that are already in the documents

- **`skipped_unstable` is not a failure.** A file too young to touch is a normal outcome, recorded as a row, exit `0`.
- **`hardlink_skip` comes before policy.** Two links to one inode are two files that are one file; ordering the check after
  the format filter would produce a plan whose rows disagree about the same bytes.
- **The clock is injected ([D-14](DECISIONS.md)).** A test that sleeps for 61 seconds is a flaky test with a slow
  half-life. Take `now` as a parameter and pass a fake in tests.
- **`include_hidden = true` does not resurrect `.tzim-*.tmp`.** That is the whole point of `K-12`.
- **The root is already canonical by the time AT-03 sees it.** Config validation did that (D-45); discovery re-checks
  containment per candidate, it does not re-decide the root.
- **No `--json` yet.** Adding it here means AT-06 inherits an output shape nobody reviewed. Human text only.
- **Do not touch the corpus expectations.** If `K-14` seems wrong about depth, that is a finding, not an edit.

## 6. What the pull request must contain

The [handoff block](HANDOFF-TEMPLATE.md) with every line filled, `remaining blocker: none`, `Merge/Release performed by
coder: no`, commits labelled `AT-03: …`, one pull request.

Under "CI runs and artifacts", include the count of planned files for the fixture root **as a number the reviewer can
recompute** from the corpus cases — a summary nobody can re-derive is the first reason a handoff comes back unread.

## 7. What comes after, so you can size this

AT-04 gives the plan its first real `tzim` calls (`--version`, `--capabilities`, argv per file), and it is the stage that
needs the fake `tzim` — the harness crate starts there. AT-03 deliberately leaves a clean seam: the planner produces
`argv` from the profile and the path, and AT-04 is the first stage that is allowed to *run* it. Keeping that seam honest now
is what makes AT-04 a fortnight rather than a rewrite.
