# Status log

Chronological record of what the manager did, what was decided, and where the evidence is. Newest last. One line per
event, with the artifact it produced; anything longer belongs in a document this log points to.

Purpose: a reader six weeks from now must be able to reconstruct *why* the project is in its current state without
reading a chat history, and without trusting anybody's memory.

---

## 2026-10-29

| Time (UTC, approx.) | Actor | Event | Artifact |
| --- | --- | --- | --- |
| 19:22 | owner | Repository `RUdolf1517/auto-tzim` created: public, MIT, `main`, issues enabled | — |
| 19:30 | manager | Charter **v0.1** and technical specification **0.1** received; first pass over `tzim` at `d4e4597` | — |
| 19:35 | manager | Specs copied verbatim with provenance; manager records created (pre-flight, study, decisions, questions, risk register, issue breakdown, gates, assignment draft) | `SPEC.md`, `MANAGER-SPEC.md`, `docs/spec/PROVENANCE.md`, `docs/manager/*` |
| 19:40 | manager | Pre-flight: **P2 not met** — `tzim` PR #84 open with two failing measurement jobs; seven issues open | `docs/manager/PREFLIGHT-2026-10-29.md` |
| 19:45 | manager | Four questions to the owner (Q1 authorisation, Q2 machine interface, Q3 dry-run semantics, Q4 exit code). Owner: wait; and **describe** Q2–Q4 rather than decide | `docs/manager/QUESTIONS-TO-OWNER.md` §0 |
| 19:50 | manager | Problem notes P-01 (no capabilities/reasons API), P-02 (a `tzim` dry run writes a temporary candidate), P-03 (missing pre-flight exit code, collision on `4`) | `docs/manager/problems/P-01…P-03` |
| 19:55 | manager | Product documentation set written as specification (16 documents); every one marked "no implementation" | `docs/*.md`, `CONTRIBUTING.md`, `CHANGELOG.md` |
| 19:59 | **upstream** | `tzim#85` created — the capabilities / structured-reasons request. **Not by me**; this is the canonical one | `tzim#85` |
| 20:00 | owner | Charter **v0.2** received: §16 decides P-01/P-02/P-03, nineteen invariants, first managerial result = issues + assignment + acceptance plan | `MANAGER-SPEC.md` (v0.2) |
| 20:05 | manager | Charter v0.2 recorded; `CONFLICTS.md` resolves the four places where the older technical spec disagrees | `docs/spec/CONFLICTS.md` |
| 20:07 | manager | Issues created: AT-01…AT-09 = **#1…#9**; consolidated assignment = **#10** (prepared, start blocked) | GitHub #1…#10 |
| 20:11 | manager | **Mistake:** filed `tzim#86` duplicating `tzim#85` — the open list was not re-read first | `tzim#86` (to be closed) |
| 20:14 | manager | Discovered the duplicate; #85 declared canonical, the "correction" of #10's reference reverted, root cause recorded | `ROADMAP-ISSUES.md`, `CHANGELOG.md` |
| 20:15 | manager | HTTP 403 confirmed for every issue edit/comment/label-attach; owner actions written as copy-paste commands | `docs/manager/OWNER-ACTIONS.md` |
| 20:20 | manager | Acceptance matrix revised for v0.2: 13 rows, 19 invariants, read-only-scan proof by invocation counter, exit-code matrix | `docs/manager/GATES.md` |
| 20:25 | manager | Corpus **frozen before any code**: ~110 cases by ID with expected outcomes | `docs/manager/FIXTURE-CORPUS.md` |
| 20:30 | manager | Review protocol: SHA binding, CI checks, `jq` reconciliation, byte evidence, six controls the reviewer runs | `docs/manager/EVIDENCE-PROTOCOL.md` |
| 20:35 | manager | Harness contract written (fake `tzim`, invocation log schema, `run-case`, pinned binary, bundle layout) — the protocol referenced it before it existed | `docs/testing-harness.md` |
| 20:40 | manager | Handoff and review templates as fill-in forms; upstream tracker; FAQ; this log | `docs/manager/HANDOFF-TEMPLATE.md`, `REVIEW-TEMPLATE.md`, `UPSTREAM.md`, `docs/faq.md` |
| 20:40 | manager | Re-checked the blocker: PR #84 has a **new commit** `2ff29b95`, CI running (2 queued, 2 in progress). The coder is actively finishing the `tzim` set — the MVP start stays blocked | `gh pr checks 84` |
| 20:22 | manager | Pull request **#11** opened: branch → `main`, documentation only (44 files, no code, no CI) | `#11` |
| 20:23 | owner | Instruction received: move the documentation to `main` | — |
| 20:23 | manager | `main` is a **protected branch** (`required_pull_request_reviews`), and GitHub forbids approving one's own pull request, so a normal merge was impossible: `gh pr merge 11` → *"the base branch policy prohibits the merge"*. Merged with `--admin`, i.e. **bypassing the review requirement**, on the owner's explicit instruction. Merge commit `1328919`; `main` = 45 files, `docs/manager` = 17 documents | `#11` MERGED |
| 20:52 | manager | Owner is ready to close out the owner actions; re-tested the integration instead of trusting yesterday's 403 list. **Split confirmed:** issue/label writes still `403` (tested with an idempotent label attach on `tzim#86`); pull requests, `git push`, `gh pr merge --admin` and all reads work | `gh issue edit 86 --add-label enhancement` → 403 |
| 20:53 | manager | Owner action §2 (**labels**) verified **done**: #1…#9 carry `v0.1`, #10 carries `manager`+`assignment`, `tzim#86` carries `enhancement`. One gap: `tzim#85`, the canonical issue, is unlabelled | `gh issue view` on all ten |
| 20:54 | manager | **Correction:** OWNER-ACTIONS §6a offered a command that cannot work — GitHub cannot scope a review requirement to file paths ("restrict file paths" is a *push* rule). Section rewritten with three implementable options and the mistake recorded | [OWNER-ACTIONS](OWNER-ACTIONS.md) §6a |
| 20:58 | owner | Decision on §6a: **defer** — no change to `main` protection today; documentation-only pull requests continue to merge with `--admin` and are logged | [D-24](DECISIONS.md) |
| 20:58 | owner | Decision on `tzim#86`: close it now, do not port the JSON sketches into `#85`. **Not executed** — issue writes are still `403` for me (re-tested with an idempotent `PATCH state=open`), so it needs the owner's own shell | [OWNER-ACTIONS](OWNER-ACTIONS.md) §1 |
| 20:59 | owner | Decision on this branch: merge to `main` **after** the §6a decision. Since §6a is now deferred, the corrections stay on `arena/db523563-auto-tzim` until that changes | branch `84348ec` |
| 2026-11-01 20:20 | manager | Read the upstream live state instead of the 2026-10-29 snapshot: **PR #84 merged** (2026-10-30T14:55:59Z), `tzim` tracker **0 open issues, 0 open PRs**, three releases since (`v0.15.0`, `v0.16.0`, `v0.16.1`). **B-01 is cleared** — charter §3 is satisfied | `gh pr view 84`, `gh issue list`, `gh release list` |
| 2026-11-01 20:25 | manager | **The contract moved under the documentation.** `tzim#85` was implemented (PR #91, `503d1eca`, v0.16.0): `--capabilities`, JSON result schema **2**, stable `reason_code`. `--dry-run` and the FITS confirmation also exist now. Every "no capability list" statement in this repository is false | [audit](../audits/2026-11-01-tzim-v0.16.1.md) |
| 2026-11-01 20:30 | manager | Audited the whole contract against `tzim` v0.16.1 (`f605c2ea`): **ten findings, six severity A**. Recorded with `file:line`, evidence and an action map, in the new `docs/audits/` folder — the first entry, and the reason the folder now exists | [2026-11-01-tzim-v0.16.1.md](../audits/2026-11-01-tzim-v0.16.1.md) |
| 2026-11-01 20:35 | manager | Added the repository files `tzim` has and we did not: `SECURITY.md`, `OWNERS.md`, `THIRD-PARTY.md`, `.github/CODEOWNERS`, `docs/audits/`. Wrote [DOC-MODEL](DOC-MODEL.md) mapping every document in one repository to the other, with the reason for each difference | this commit |
| 2026-11-01 20:36 | owner | Instruction: merge the documentation corrections | — |
| 2026-11-01 20:36 | manager | Pull request **#13** merged to `main` as `4c6631f` (25 files, markdown only — the tree was checked for code, config and workflow paths before merging). `--admin` under the **D-24** interim rule, as with #11 and #12 | [#13](https://github.com/RUdolf1517/auto-tzim/pull/13) MERGED |
| 2026-11-01 21:05 | owner | Product decisions in one sitting: consume `--capabilities` (D-26); surface the refusal `reason` (D-27); remove `update-check` from the MVP (D-28); **two modes — automatic, and with confirmation where the originals are deleted once the user agrees** (D-29) | [DECISIONS](DECISIONS.md) |
| 2026-11-01 21:10 | manager | Sharpened D-29 into something testable: one install path, the mode gates only the proposal; work happens on a staged copy so the user's folder is untouched until install; journaled swap; an original is deleted only after its verified replacement is in place; identity re-checked per file; `confirm` fails closed without a terminal | `safety.md` INV-20…23, `GATES.md` AC-14…17, corpus `M-01`…`M-14` |
| 2026-11-01 21:15 | manager | Recorded the two invariants this touches. Invariant 12 ("no prompt in any mode") is **rewritten, not deleted**: no PTY stays absolute, and the single approval gate must fail closed. Invariant 16 ("no undo") stays true and now carries the ordering guarantee beside it | `safety.md`, `guarantees.md`, `limitations.md` |
| 2026-11-01 21:20 | manager | Consequences of D-28 recorded where they belong: `privacy.md` becomes a zero-request statement, `urllib` leaves the dependencies, exit `5` becomes reserved, and charter §17.5 is overridden **without editing the pinned `SPEC.md`** | `privacy.md`, `dependencies.md`, `cli.md`, D-28 |
| 2026-11-01 21:35 | owner | Confirmed the `auto` semantics: **a plan of files is created, they are optimized, originals are not kept**. Recorded inside D-29 — the plan is not skipped in `auto`, it is the internal binding that refuses a file changed between the walk and the work; what `auto` removes is the question, not the plan | [DECISIONS](DECISIONS.md) D-29 |
| 2026-11-01 21:40 | manager | Rewrote the assignment draft (#10) against the new contract: staging and journaled install in AT-05, `--mode` and the approval gate in AT-06, `--capabilities` and `reason` in AT-04, AT-07 struck from the iteration, the harness counters and the M-block added to the required tests | `ASSIGNMENT-AT-MVP-DRAFT.md` |
| 2026-11-01 21:50 | owner | Q13 answered: accepted `tzim` window is **`>=0.16.0, <0.17`**. Recorded as adopted in [D-30](DECISIONS.md) — one minor series, tied to the schema pair and checked at runtime rather than inferred from the version string | [DECISIONS](DECISIONS.md) D-30 |
| 2026-11-01 21:50 | owner | Instruction: merge | — |
| 2026-11-01 21:52 | manager | Pull request **#14** merged to `main`. It is a **contract** change, not an editorial one — two modes, deletion of originals after verified install, charter §17.5 overridden, invariant 12 rewritten — so the `--admin` merge under the D-24 interim rule is logged here explicitly rather than treated as routine | [#14](https://github.com/RUdolf1517/auto-tzim/pull/14) MERGED |
| 2026-11-01 22:10 | owner | **The project is Rust, not Python.** `SPEC.md` §4.1 recommends Python 3.12+; the recommendation is superseded, not edited | [D-33](DECISIONS.md), [C-5](../spec/CONFLICTS.md) |
| 2026-11-01 22:12 | owner | Qt is a **separate binary, installed separately, without the CLI or TUI** | [D-34](DECISIONS.md) |
| 2026-11-01 22:15 | manager | Recorded D-33 (stack, accepted crates, what it costs, reversal condition) and D-34 (three artifacts, one core, no front door shells out to another). `SPEC.md` stays pinned; the override is a decision plus a conflict row | [DECISIONS](DECISIONS.md), [CONFLICTS](../spec/CONFLICTS.md) C-5 |
| 2026-11-01 22:20 | manager | Rewrote the Python-specific surface the language change touched: `dependencies.md` (crates, `cargo deny`, what left and why), `THIRD-PARTY.md`, `releases.md` (tarballs + `SHA256SUMS` + Homebrew instead of PyPI), `testing.md`, `testing-harness.md` (Rust harness binary, `run-case` via `cargo run`), `CONTRIBUTING.md`, `architecture.md` (workspace layout), `cli.md`, `roadmap.md`, `update.md`, GATES/corpus/assignment/templates | 22 files |
| 2026-11-01 22:22 | manager | D-31 rewritten: with both targets static binaries, self-update is the same atomic `rename` for `tzim` and for us. The earlier "delegate to pip/brew because `site-packages` cannot be replaced safely" argument is kept in the record as history, since it was true of the Python stack | D-31 |
| 2026-11-01 23:05 | owner | Four answers: §6a → **option 1** (machine gates only); issue #10 → **option Б** (refresh the body); **`cargo install` is planned, the binary keeps its hyphen**; **state must have no C dependencies** — plain JSONL files instead | [DECISIONS](DECISIONS.md) D-24, D-35, D-36 |
| 2026-11-01 23:10 | manager | Rewrote the state layer as files: run directories with append-only `files.jsonl`, atomic `run.json` replacement, exclusive-create lock with a start-time discriminator, compaction for the eligibility log. `rusqlite` removed; the build now has no C in it | [D-35](DECISIONS.md), `state.md`, `recovery.md` |
| 2026-11-01 23:15 | manager | Distribution recorded: GitHub releases, Homebrew and `cargo install auto-tzim`; the CLI crate is renamed `auto-tzim` so the published name matches the binary. A `cargo`-installed binary is updated by asking `cargo` | [D-36](DECISIONS.md), `releases.md`, `update.md` |
| 2026-11-01 23:20 | manager | §6a closed as decided and not applied: the `DELETE` needs the owner's credentials (`403` here). Verification commands and the ruleset checklist are in place; D-24 is marked resolved, not deleted | [OWNER-ACTIONS](OWNER-ACTIONS.md) §6a |
| 2026-11-01 23:22 | manager | §6b added: refreshing issue #10 is the one action that still blocks the coder, because the issue still describes a Python package with no modes | [OWNER-ACTIONS](OWNER-ACTIONS.md) §6b |
| 2026-11-02 00:20 | owner | Reported §6a and §6b done | — |
| 2026-11-02 01:05 | manager | **Correction to the row above:** the issue body does not say "Python package" — it says `python --version` once, in a list of toolchain facts. I read a `grep -ci python` count of `1` as the phrase without looking at the line. The finding stands and is now stated properly: the body is byte-identical to the pre-report copy, 122 lines against the file's 170 | [OWNER-ACTIONS](OWNER-ACTIONS.md) §6b |
| 2026-11-02 01:06 | manager | Owner's re-run of the §6a check returned `mergeStateStatus: UNKNOWN`, which is GitHub's "not computed yet" answer, not success; my own query a minute later returned `BLOCKED` again. `UNKNOWN` is now called out in §6a so it is not read as a pass | [OWNER-ACTIONS](OWNER-ACTIONS.md) §6a |
| 2026-11-02 02:05 | manager | **PR #16 merged as `03fe713` with a plain `--merge`, no `--admin` — the first merge in this repository that needed no bypass.** That is the evidence §6a works: once the review rule and the placeholder `test` check were gone, the gate stopped being a formality and started being a gate. `main`: 52 files, 36 decisions, 0 code files | [#16](https://github.com/RUdolf1517/auto-tzim/pull/16) MERGED |
| 2026-11-02 01:52 | owner | Dumped the protection object: required context **`test`**, `strict: true`, `signatures/linear/conversations/admins` all `false`. Zero workflows exist in the repository, so that check could never be reported — the real blocker, not the review rule | — |
| 2026-11-02 01:54 | owner | Deleted `required_status_checks` as well; the shell was in `/`, so my §6b glob matched nothing (`$status 124`) and the issue body stayed at 122 lines | — |
| 2026-11-02 01:56 | manager | **Verified independently:** `pulls/16` answers `mergeable_state: clean` four times in a row, and `branches/main` reports `required_status_checks: {contexts: [], enforcement_level: "off"}` with `protected: true`. §6a is done. Follow-up recorded: when AT-01 lands CI, required checks return **with the real context**, never a placeholder | [D-24](DECISIONS.md), [OWNER-ACTIONS](OWNER-ACTIONS.md) §6a |
| 2026-11-02 01:30 | owner | Ran the §6a `DELETE`: `HTTP/2.0 204 No Content` — the rule existed and is gone. (His shell is fish, so `$?` in my block was wrong; `$status` is the one) | — |
| 2026-11-02 01:33 | manager | **Corrected my own inference.** After the `204`, pull request #16 still reports `blocked`, but `review_decision` is now `null` — the review rule really is gone, and "blocked with no checks" was never evidence of a required review. The repository has **zero workflows**, so a required status check nothing reports is the leading candidate; required signatures second. Asked for the protection dump instead of inferring a third time | [OWNER-ACTIONS](OWNER-ACTIONS.md) §6a |
| 2026-11-02 01:35 | manager | **§6b never ran, and it was my command.** The path in my message was rendered as a markdown link, the paste carried the link, and `gh issue edit` answered `no such file or directory` (`$status` 1, body still 122 lines). Rewritten to a fish glob and an API fetch, neither of which depends on a filename surviving a renderer | [OWNER-ACTIONS](OWNER-ACTIONS.md) §6b |
| 2026-11-02 00:22 | manager | Measured instead of accepting it: issue #10 `updatedAt` still `2026-10-29T20:13:58Z` and the body still says "Python package"; `main` still `protected: true`; pull request #16 `BLOCKED` with no checks configured, so a required review is the only possible blocker. Recorded as **not observed**, with the three commands that settle each | [OWNER-ACTIONS](OWNER-ACTIONS.md) §6a, §6b |
| 2026-11-02 00:26 | manager | Re-read GATES against D-26…D-36 instead of trusting the earlier sweep: five acceptance rows still demanded Python evidence (`shell=False`, `stdin=DEVNULL`, `import urllib.request`, `socket.socket`, `os.replace`, "no `pty` import") and AC-7 still said "the database is readable" a day after D-35. Rewrote AC-5, AC-7, INV-8, INV-14, INV-15 | [GATES](GATES.md) |
| 2026-11-02 00:29 | manager | Found the v0.1 network claim alive in nine documents — README, SECURITY, guarantees A2/A6, architecture, faq, CONTRIBUTING, cli §2.11, config — all still describing the `update-check` request that D-28 removed. The documents disagreed with AC-9, the gate that tests them. Rewrote them; `update-check` is now "a command that does not exist and exits `2`" | 9 files |
| 2026-11-02 00:33 | manager | DR-02 was marked decided but not done: `cli.md` never mentioned `reason` at all. Added it to §2.8 and §3 and wrote corpus case `A-12` so an unknown code has somewhere to fail | [cli](../cli.md), [corpus](FIXTURE-CORPUS.md) |
| 2026-11-02 00:36 | manager | Closed the 2026-11-01 audit: ten findings mapped to the document that carries each, with the grep or case behind the check. Three corpus rows added (`A-11` capabilities, `A-12` reason, `X-17` EPIPE), two reworded — allowed only because no code and no result exist yet, and the close-out says so | [audit](../audits/2026-11-01-tzim-v0.16.1.md) |
| 2026-11-02 00:39 | manager | The assignment still told the coder to build "SQLite state" with `shell=False`, in the file he is meant to work from, one day after D-35. Fixed, with the file state layer named explicitly | [ASSIGNMENT](ASSIGNMENT-AT-MVP-DRAFT.md) §5 |
| 2026-11-01 20:40 | manager | Tightened the gate: **D-25** — no implementation start until the severity-A drift is closed. Charter §3 is satisfied; the contract is not. Three parts of the fix are the owner's: [Q11](QUESTIONS-TO-OWNER.md) (`--capabilities`), Q12 (`reason_code`), Q13 (version window) | [DECISIONS](DECISIONS.md) D-25 |

**Governance note, recorded rather than glossed over:** the admin merge is a precedent. Charter §2.2 forbids the manager
from self-merging and from self-approval, and this merge did both in effect. It was done because the owner asked for the
transfer in this session and the platform offers no other path for a sole-owner repository (a PR cannot be approved by its
author, including an administrator). If that is not the intended standing rule, the fix is one of: require a second
reviewer identity for `main`, drop the review requirement for documentation-only pull requests, or keep using `--admin`
deliberately and say so. Owner action, listed in [OWNER-ACTIONS](OWNER-ACTIONS.md) §6a.

## State at the end of 2026-10-29

| Item | State |
| --- | --- |
| Issues | #1…#9 (MVP), #10 (assignment, not started) |
| Owner decisions | charter v0.2 §16: D-01…D-05, D-18, D-19 adopted |
| Manager defaults in force | D-07 (update source), D-09 (hash cap), D-15 (dependencies) |
| Blocking | **B-01** — precondition §3.2: `tzim` PR #84 unfinished |
| Open owner actions | close `tzim#86`; attach labels; repo description; spec home (Q9); B-01 decision; optional branch protection |
| Product code | none, by design |
| Documentation on `main` | **yes** — merge commit `1328919` (PR #11, admin merge under branch protection) |

## Template for new entries

```markdown
| <HH:MM> | <owner/manager/coder/upstream> | <what happened, one sentence> | <artifact or link> |
```

Rules: one line per event; a decision goes to [DECISIONS](DECISIONS.md) and is *referenced* here, not restated; a mistake
is recorded with its root cause, not smoothed over; nothing is back-dated.
