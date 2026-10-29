# FAQ

> **Status: answers about the contracted product, which does not exist yet.** Every answer points at the document that has
> the long version.

## For a user

**Will it shrink my files?**
Sometimes. `auto-tzim` decides *which* files may be handed to `tzim`, and `tzim` decides whether anything happens.
An `already_optimal`, a refusal and an unsupported format are all normal outcomes with exit `0`.

**Why is nothing happening after I added a folder?**
Adding a folder arms a plan; it does not process anything (invariant 1). Run `auto-tzim scan` to see the candidate plan,
then apply it.

**Why does `run --once` refuse with exit code `6`?**
Six means "refused before processing": there is no matching plan yet (first run of a folder), a `pause` is active, free
space is below `min_free_bytes`, or the folder is disabled. Nothing was read for processing and nothing changed.

**Does it change files during a preview?**
No. `scan` (and `run --dry-run`, which is the same operation) never runs `tzim` on a file, never passes `--dry-run`, and
writes nothing in your folder — it does not even need write permission there. This is deliberate: `tzim`'s own dry run
*would* write a temporary candidate into the destination directory, which is why we do not use it
([CONFLICTS](spec/CONFLICTS.md) C-1).

**Why is there no "how much would I save" number before running?**
Because a number would require either running the compressor (which writes) or guessing. The plan tells you which files
will be handed over and with which flags; the savings appear after an apply, per file, in the history.

**Is there an undo?**
No. A successful optimization rewrites the file and the previous bytes are gone. Backup software may have a copy; that is
not a promise we make ([limitations](limitations.md) §1, invariant 16).

**What if a file is a symlink, or a hard link, or is being written right now?**
It is skipped, with the reason recorded: symlinks and special files are never followed, hard-linked files are skipped by
default, and a file that changes between stages ends as `skipped_unstable` rather than being raced.

**Why one file at a time?**
Attribution, cancellation and timeouts stay trivial, and parallelism inside a file is `tzim`'s own. Parallel files are a
later, separately reviewed change ([architecture](architecture.md) §3).

**Is anything sent over the network?**
One request, from one command: `update-check`. Everything else — including `run` and `scan` — opens no socket. The
complete list is in [privacy](privacy.md).

**Will it update `tzim` for me?**
Not in v0.1: `update-check` only tells you that a newer version exists. A controlled update is a later stage with its own
rules ([update](update.md)).

## For a reviewer or a newcomer

**Why does the exit code `4` not exist here?**
`tzim` uses `4` for a verification failure. Ours is `6` for a pre-flight refusal, `7` internal, `8` lock, and `4` is
deliberately never produced, so a child's failure can never be mistaken for our state
([cli](cli.md) §4, decision D-18/D-19).

**What is `skipped_by_tzim`, and why one bucket?**
`tzim` reports every refusal it makes as `skipped` plus a free-text note. Distinguishing them would mean parsing prose,
which the charter forbids (invariant 18). So the status stays generic and the note is stored verbatim; the distinctions we
*can* make — policy, instability, hard links — are made by our own layer, before any process starts.

**Where is the machine-readable capability list of `tzim`?**
There isn't one. `--info` and `--list-formats` are human text, and the default format set is build-dependent. That is why
we delegate by omitting `--formats` and record the binary's version and SHA-256 instead of a capability claim. The upstream
request is [tzim#85](https://github.com/RUdolf1517/tzim/issues/85).

**Two specifications — which one wins?**
For process, evidence and acceptance: the manager charter v0.2. For product behaviour: the technical specification 0.1,
except in the four places charter §16 overrides it, listed in [CONFLICTS](spec/CONFLICTS.md).

**Can the coder start?**
Not yet. Issue [#10](https://github.com/RUdolf1517/auto-tzim/issues/10) carries blocker **B-01**: the previous `tzim` set
is unfinished, and charter §3.2 forbids starting a new project as a reason to drop it
([pre-flight](manager/PREFLIGHT-2026-10-29.md) P2).

**Why so much documentation before any code?**
Because the acceptance criteria have to be written before the first result, or they get shaped by it. This project's three
sharpest rules — read-only scan, generic skip status, own exit codes — were decided in documents and only then became
issues ([GATES](manager/GATES.md), [FIXTURE-CORPUS](manager/FIXTURE-CORPUS.md)).

**What would a reviewer accept as proof?**
Raw rows, checksums, a bundle with `SHA256SUMS`, CI runs at the final head SHA, and six controls performed by the reviewer
personally — not a green badge and not a summary ([EVIDENCE-PROTOCOL](manager/EVIDENCE-PROTOCOL.md)).
