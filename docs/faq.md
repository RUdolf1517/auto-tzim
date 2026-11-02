# FAQ

> **Upstream contract moved (2026-11-01).** `tzim` v0.16.0 shipped `--capabilities`, a stable `reason_code` and
> JSON result schema 2; v0.16.1 is the current release and its exit-code precedence is now documented and
> enforced. Statements below written against `tzim` 0.14 that assume no capability document, an untyped
> refusal or an older pin are stale: see the [2026-11-01 audit](audits/2026-11-01-tzim-v0.16.1.md).

> **Status: answers about the contracted product, which does not exist yet.** Every answer points at the document that has
> the long version.

## For a user

**Will it shrink my files?**
Sometimes. `auto-tzim` decides *which* files may be handed to `tzim`, and `tzim` decides whether anything happens.
An `already_optimal`, a refusal and an unsupported format are all normal outcomes with exit `0`.

**Why is nothing happening after I added a folder?**
Adding a folder arms a plan; it does not process anything (invariant 1). Run `auto-tzim scan` to see the candidate plan,
then apply it.

**Why does a run refuse with exit code `6`?**
Six means "refused before processing": a `--plan` you named no longer matches the folder, a `pause` is active, free
space is below `min_free_bytes`, the folder is disabled — or you asked for a run without `--auto` and there is no terminal
to confirm on. Nothing was read for processing and nothing changed.

**Does it change files during a preview?**
No. `scan` (and `run --dry-run`, which is the same operation) never runs `tzim` on a file, never passes `--dry-run`, and
writes nothing in your folder — it does not even need write permission there. This is deliberate: `tzim`'s own dry run
*would* write a temporary candidate into the destination directory, which is why we do not use it
([CONFLICTS](spec/CONFLICTS.md) C-1).

**Why is there no "how much would I save" number before running?**
Because a number would require either running the compressor (which writes) or guessing. The plan tells you which files
will be handed over and with which flags; the savings appear after an apply, per file, in the history.

**Is there an undo?**
No. Once a replacement is installed, the previous bytes are gone ([limitations](limitations.md) §1, invariant 16). What
you *do* get is control over when that happens: in the default `confirm` mode nothing is touched until you have read the
proposal and typed `y`, and a declined proposal installs nothing and exits `0`.

**What are the two modes, and which one should I use?**
`confirm` (default) shows what would change and waits for your answer; `auto` does the work without asking, for a script
or a cron job. Both run the same code — the mode only decides whether you are asked. Start with `confirm`
([cli](cli.md) §2.6, decision D-29).

**What if a file is a symlink, or a hard link, or is being written right now?**
It is skipped, with the reason recorded: symlinks and special files are never followed, hard-linked files are skipped by
default, and a file that changes between stages ends as `skipped_unstable` rather than being raced.

**Why one file at a time?**
Attribution, cancellation and timeouts stay trivial, and parallelism inside a file is `tzim`'s own. Parallel files are a
later, separately reviewed change ([architecture](architecture.md) §3).

**Is anything sent over the network?**
Nothing. v0.1 opens no socket at all (D-28) — there is no network code in the binary to switch off — and `run`/`scan`
are no exception. The v0.2 `update-check` will be the first request; [privacy](privacy.md) describes it in the future
tense, on purpose.

**Will it update `tzim` for me?**
No. v0.1 has no update command at all. v0.2 adds `update-check`, which only tells you a newer version exists; a controlled
update with the lock held and the result verified is v0.3, with its own rules ([update](update.md), [roadmap](roadmap.md)).

## For a reviewer or a newcomer

**Why does the exit code `4` not exist here?**
`tzim` uses `4` for a verification failure. Ours is `6` for a pre-flight refusal, `7` internal, `8` lock, and `4` is
deliberately never produced, so a child's failure can never be mistaken for our state
([cli](cli.md) §4, decision D-18/D-19).

**Why was my file skipped?**
Since `tzim` v0.16.0 every refusal carries a stable `reason_code`, and we pass it through in the `reason` field beside
`tzim`'s own text, which we keep verbatim. We never parse that text (invariant 18) — but you no longer have to read a
sentence to know whether the cause is fixable ([cli](cli.md) §2.8, decision D-27).

**Where is the machine-readable capability list of `tzim`?**
It exists now, and we read it: `tzim --capabilities` returns one JSON object with the build's features, compiled and
default-enabled formats and opt-ins. We check its schema versions and fail closed if they are not the pair we were tested
against (decision D-26). We still never pass `--formats` and still do not display the default set as a promise — a set
observed on one build is not a fact about the next one.

**Two specifications — which one wins?**
For process, evidence and acceptance: the manager charter v0.2. For product behaviour: the technical specification 0.1,
except in the four places charter §16 overrides it, listed in [CONFLICTS](spec/CONFLICTS.md).

**Does `auto-tzim` talk to the network?**
No. v0.1 opens no socket at all: there is no update check, no telemetry, no crash reporting ([privacy](privacy.md),
decision D-28).

**Can the coder start?**
Not yet, but for a different reason than before. The old blocker (**B-01**) is gone: `tzim` PR #84 merged on 2026-10-30,
the upstream tracker is empty and v0.16.1 shipped. What holds the start now is the contract itself — six acceptance-critical
facts changed in that release, and the documents were corrected on 2026-11-01
([audit](audits/2026-11-01-tzim-v0.16.1.md), decision D-25). Starting earlier would mean writing code that passes our tests
and fails against every current `tzim`.

**Why so much documentation before any code?**
Because the acceptance criteria have to be written before the first result, or they get shaped by it. This project's three
sharpest rules — read-only scan, generic skip status, own exit codes — were decided in documents and only then became
issues ([GATES](manager/GATES.md), [FIXTURE-CORPUS](manager/FIXTURE-CORPUS.md)).

**What would a reviewer accept as proof?**
Raw rows, checksums, a bundle with `SHA256SUMS`, CI runs at the final head SHA, and six controls performed by the reviewer
personally — not a green badge and not a summary ([EVIDENCE-PROTOCOL](manager/EVIDENCE-PROTOCOL.md)).
