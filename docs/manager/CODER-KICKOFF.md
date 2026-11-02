# Kickoff — message to the coder, and the channel through the owner

Written 2026-11-02. The coder is a separate session with no context from the manager's. Everything it needs must be in
the message it receives, or in files in this repository — not in a conversation it cannot see.

The manager does not write product code. The coder does not merge, does not close issues, does not publish. Every claim
in a handoff is checked independently by the manager, who runs the commands rather than reading the summary.

## 1. Before sending anything: the precondition

The kickoff message assumes `main` contains `docs/manager/HANDOFF-AT-01.md`. It does not until PR #19 is merged. The
message therefore opens with a check rather than a task: if the file is missing at the head of `main`, the coder stops
and says so. A coder that starts from stale documents is the exact failure this project spent three days removing.

## 2. The message

Copy the block below to the coder verbatim. It is written to be read by someone who has never seen this project.

```text
You are the coder for the project auto-tzim. I am the owner; the manager who wrote your brief
is a separate reviewer you do not talk to directly — everything goes through me.

STEP 0 — get the brief from a known commit, not from your memory:
  gh api "repos/RUdolf1517/auto-tzim/contents/docs/manager/HANDOFF-AT-01.md?ref=main" --jq .name
and if that 404s, read it from the pinned ref instead:
  gh api "repos/RUdolf1517/auto-tzim/contents/docs/manager/HANDOFF-AT-01.md?ref=arena/e2cb3538-auto-tzim" --jq .name
Report which ref you used and its commit SHA in your first message. Do not start without one of them.

READ FIRST, in this order, from a fresh clone of main:
  1. docs/manager/HANDOFF-AT-01.md        <- your actual task, one page
  2. docs/manager/ASSIGNMENT-AT-MVP-DRAFT.md  sections 1 and 2 only (context; the other
     issues are NOT yours today)
  3. docs/manager/GATES.md                <- how your work will be judged
  4. docs/manager/HANDOFF-TEMPLATE.md     <- the report format you must fill in
  5. docs/manager/DECISIONS.md            <- D-33, D-34, D-35, D-36 concern you today

YOUR TASK: AT-01 only. A Rust workspace that does nothing yet.
  - root Cargo.toml, resolver 2, exactly two members
  - crates/auto-tzim-core, package name auto-tzim-core, no behaviour
  - crates/auto-tzim, package name auto-tzim, [[bin]] name = "auto-tzim"
  - rust-toolchain.toml pinning one exact stable version (same version in CI and README)
  - Cargo.lock committed
  - deny.toml: refuse crates that compile C, refuse HTTP/network crates
  - one GitHub workflow: ubuntu-latest + macos-latest, pinned toolchain, jobs for build,
    test, cargo deny, and a pinned-dependency check
  - CLI: --help and --version only, both exit 0; unknown command or flag exits 2 with a
    message on stderr and nothing on stdout

NOT IN THIS TASK — adding any of it is a review failure, not initiative:
  config, discovery, planner, adapter, runner, state, --json, --mode, any .rs that opens
  a file, any third crate, any second workflow, the tui/ and qt/ crates, any network code.

THE REPOSITORY WINS OVER THE ISSUE TRACKER:
  Issues #1..#9 still carry text from 2026-10-29 written for a Python package. They are
  stale by construction and nobody has rewritten them yet. Source of truth: the files
  listed above. If an issue body contradicts docs/manager/HANDOFF-AT-01.md or
  DECISIONS.md, the file wins and you tell me which issue was wrong - do not "reconcile"
  them by picking the middle.

RULES THAT ARE NOT NEGOTIABLE:
  - Do not merge your own pull request. Do not publish to crates.io. Do not close issues.
  - Do not edit GATES.md, FIXTURE-CORPUS.md, DECISIONS.md or the spec copies. If an
    acceptance row looks wrong or unsatisfiable, report it as a finding instead.
  - The crate is called auto-tzim, not auto-tzim-cli. If you see the old name in a file,
    it is stale - say so, do not follow it.
  - Commits labelled "AT-01: ...". One pull request. Do not squash my review history.

THE NEGATIVE CONTROL IS PART OF THE DELIVERABLE:
  Add a dependency with an open version range (e.g. serde = "*") in a throwaway commit,
  let CI go RED, record the run id, the job name and the failing message, then revert in a
  separate commit and record the green run. Do the same once with rusqlite to prove
  deny.toml actually rejects C. A check that cannot fail is decoration and it fails silently.

WHAT YOU SEND ME WHEN ASKING FOR REVIEW (one message, in this order):
  1. the pull request number and the final head SHA
  2. the handoff block from docs/manager/HANDOFF-TEMPLATE.md with every line filled
  3. the red run id for the negative control and the green run id after the revert
  4. anything you were unsure about, stated as a question, not silently decided

The reviewer will re-run your acceptance points themselves. "It works locally" and "CI is
green" are not evidence; run ids at the head SHA are. If something is not done, write
"not done" - an honest gap is accepted, a false claim ends the review.
```

## 3. The channel, and how a review round works

The owner forwards; the manager answers with a verdict. Each round has exactly one of three outcomes, and the coder
should be told which it got:

| Verdict | Meaning | What the coder does next |
| --- | --- | --- |
| **ACCEPTED** | every acceptance point reproduced by the reviewer | merge is the owner's action; the coder moves to AT-02 only if asked |
| **CHANGES REQUESTED** | findings numbered `F-01…`, each with the command that shows it | fix, push to the same pull request, send the same report again |
| **STOPPED** | a Critical finding, or the work is out of scope | stop and wait; a stop is not a negotiation |

Findings are never "please improve". Each one names what is wrong, where, and how the reviewer saw it — the same standard
the coder is held to.

## 4. What the owner should forward, and what not to

Forward: the coder's single review-request message, and any question it asks. Both, verbatim, including the SHA.

Do not forward: the coder's internal reasoning, its draft commits, or "it's almost done" — a status without a SHA cannot
be checked, and a half-finished claim wastes a round.

If the coder asks a question that belongs to the product rather than the code — a policy, a default, a mode — the answer
is the owner's, not the manager's, and it becomes a `D-` entry that the manager writes down.
