# Readiness pass — the assignment against D-01…D-36

Written 2026-11-02, at `main` = `95325af`, the day the owner authorised the start. Purpose: read the assignment and the
issue set **line by line** against the decision log before a coder touches either, because a contradiction found here
costs an hour and the same contradiction found in review costs a rewrite.

Nothing was executed except greps and `gh` reads. Every claim below names the artifact it is about.

## 1. Method

1. Read [ASSIGNMENT-AT-MVP-DRAFT](ASSIGNMENT-AT-MVP-DRAFT.md) end to end, then the AT-01…AT-09 entries of
   [ROADMAP-ISSUES](ROADMAP-ISSUES.md), then [GATES](GATES.md) AC-1…AC-17 and INV-1…INV-23.
2. For each decision D-26…D-36 — the ones adopted after the assignment was written — grepped the whole tree for the
   vocabulary it replaced: `SQLite`, `rusqlite`, `shell=False`, `stdin=DEVNULL`, `auto-tzim-cli`, `3.12`, `3.13`,
   `import smoke`, `one --version`, `zero invocations`, `$?`.
3. Read the nine live issue bodies through the API, since those are what a coder actually opens.
4. Fixed what is mine to fix in this commit. Everything else is either an owner action or a decision.

## 2. Findings

Severity: **A** — a coder would build the wrong thing or fail a gate they cannot satisfy; **B** — a wrong statement in a
document that decides acceptance; **C** — cosmetic or ambiguous wording.

| # | Sev | What was wrong | Where | State |
| --- | --- | --- | --- | --- |
| R-01 | A | The status block said the start is **blocked by charter §3.2** and that `tzim` PR #84 is still unfinished. It merged on 2026-10-30. A coder reading the first paragraph would wait for a gate that closed a day earlier | assignment, header | fixed |
| R-02 | A | The blocker block at the end still described **B-01 as live**, with 2026-10-29 observations (`run 40939520316`, "both jobs fail"). Charter §10 wants `remaining blocker: none`, not a resolved blocker dressed as an open one | assignment, §B-01 | fixed — marked cleared, text kept as history |
| R-03 | A | **The invocation counter contradicted the capability probe.** D-26 adds `tzim --capabilities` at start, so a scan makes **two** flag-only spawns; six documents still demanded "exactly one `--version`", and three others said "zero invocations" for a second run, which is impossible once there is a probe at start | GATES AC-2, AC-6 · guarantees · safety · testing · EVIDENCE-PROTOCOL · corpus `R-14` · ROADMAP AT-05, AT-08 | fixed — one two-bucket definition, see §3 |
| R-04 | A | **AC-2's positive control** said an apply "must show `tzim` invocations with a path", while INV-20 fails any run whose argv names a file inside the root. Read together, apply could never pass both | GATES AC-2 | fixed — the control now names **staged copies** |
| R-05 | A | **AT-01 named the wrong crate** (`auto-tzim-cli`, renamed by D-36), carried an orphan fragment ("and\n3.13,") left from the Python text, and asked for an "Import smoke" test in a Rust workspace. This is the text of issue #1, the first thing the coder opens | ROADMAP AT-01 | fixed here; issue body still stale — see §4 |
| R-06 | A | **AT-09's acceptance described the pre-D-26/27 world**: "capabilities only as version + binary hash" and "`skipped_by_tzim` is generic". Both are now false, and they are the criteria the documentation is signed off against | ROADMAP AT-09 | fixed |
| R-07 | B | **D-34's artifact table mapped the `auto-tzim` artifact to crate `auto-tzim-cli`** — contradicting D-36 in the same file, twenty lines above the paragraph explaining the rename | DECISIONS D-34 | fixed |
| R-08 | B | The **CI matrix template said toolchain `3.12`** (a Python version), and Q8 recommended "one job per stage on 3.12 only". A coder filling in the template reproduces the wrong matrix | REVIEW-TEMPLATE §2, QUESTIONS Q8 | fixed |
| R-09 | B | The state layer appeared in **both #2 and #5** with no split, so two issues could each build a different half | assignment items 2 and 5 | fixed — #2 owns the three primitives, #5 owns the records |
| R-10 | C | `$?` in the required artifacts and in AT-06's acceptance. Bash-only, in documents the owner pastes into **fish** — the shell where `$?` does not exist and `echo $status` is the check | assignment, ROADMAP AT-06 | fixed — "exit status" |
| R-11 | C | The review order started at `#2`, skipping the issue that creates the repository | assignment | fixed |
| R-12 | C | "Both crates publishable" read like AT-01 must **publish** to crates.io. Publication is a release action (D-36); what AT-01 can prove is `cargo package` plus `cargo install --path` producing a binary named `auto-tzim` | assignment item 1, ROADMAP AT-01 | fixed |

## 3. The counter, defined once

GATES AC-2 now defines the vocabulary the rest of the documents use, so a coder cannot satisfy one row by breaking
another:

| Bucket | What it is | Scan / dry run | Apply |
| --- | --- | --- | --- |
| **flag-only probe** | `--version`, `--capabilities`; no path, no file read | exactly 2, in that order | exactly 2 |
| **path invocation** | an argument naming a file or directory | **0** | ≥1, naming a **staged copy** under `AUTO_TZIM_HOME/staging/<run_id>/` |

Consequences now stated in each place: a second run over an unchanged corpus makes **zero per-file spawns** and still
probes twice; "invocation counter shows one `--version`" is gone from all six documents.

## 4. The one thing still stale, and it is the coder's copy

Issue bodies **#2…#9 have not been touched since 2026-10-29** (`updatedAt` unchanged), and they contradict the current
design in different ways:

| Issue | What it still says |
| --- | --- |
| #1 | "Python package `auto_tzim`", `pyproject.toml`, `requires-python >=3.12` |
| #4 | "`tzim --version` only", fake versions starting at `0.13.9` — no `--capabilities`, no D-30 window |
| #5 | "SQLite state with transactions", "SQLite dumps per scenario" |
| #6 | `update-check` described as a command of this release |

`#10` now carries the current assignment, but a coder working issue by issue reads **#1 for AT-01** — so this is not
cosmetic. Issue writes return `403` to this session, so regenerating the nine bodies is the owner's. One loop, no clone,
no working directory (`awk` and `gh` only):

```fish
for n in 1 2 3 4 5 6 7 8 9
  set id (printf "AT-%02d" $n)
  gh api "repos/RUdolf1517/auto-tzim/contents/docs/manager/ROADMAP-ISSUES.md?ref=main" -H "Accept: application/vnd.github.raw" \
    | awk -v id="### $id" '$0 ~ "^" id {f=1; next} f && /^#/ {f=0} f' \
    | gh issue edit $n --repo RUdolf1517/auto-tzim --body-file -
end
```

Verify (the count must be zero, and the dates must be today):

```fish
for n in 1 4 5 6
  printf "#%s python/sqlite hits: " $n
  gh issue view $n --repo RUdolf1517/auto-tzim --json body --jq .body | grep -ci -E "pyproject|SQLite|0\.13\.9"
end
gh issue view 1 --repo RUdolf1517/auto-tzim --json updatedAt --jq .updatedAt
```

## 5. Start order, and what "AT-01 done" means

**AT-01 only.** It is a skeleton and nothing else: two crates, pinned toolchain, CI on both platforms, `--help` and
`--version` with exit `0`, an unknown command with exit `2`. No config, no discovery, no runner, no state.

| Done when | Evidence |
| --- | --- |
| A clean checkout builds and the binary is named `auto-tzim` | `cargo install --path crates/auto-tzim` then `auto-tzim --version` |
| Both platforms run it | CI run IDs at the final head SHA, `ubuntu-latest` and `macos-latest` |
| An unknown command exits `2` | the command, its stdout/stderr and its exit status in the handoff |
| Both crates are publishable | `cargo package` per crate; **not** a crates.io publication |
| An unpinned dependency fails CI | the negative control: pin one to `*`, watch the job go red, revert |
| Nothing beyond help/version exists | `--help` output reviewed: no command that does not exist yet |

The negative control is the part that gets skipped, so it is named here: a CI job that cannot fail is decoration.

## 6. The brief exists, so the start is a handover rather than a conversation

[HANDOFF-AT-01](HANDOFF-AT-01.md) is written and ready: what to build, what not to, the six acceptance points with the
evidence each needs, and the negative control (an unpinned dependency must turn CI red, with the run ID recorded) as a
deliverable rather than a footnote. A coder can be pointed at it without reading this file first.

## 7. What this pass did not do

- No code, no `Cargo.toml`, no workflow — the manager does not write product code or CI, ever.
- No verification of anything at runtime: there is nothing to run.
- No change to `SPEC.md` or `MANAGER-SPEC.md`.
- The nine issue bodies — §4, owner action.

Twelve findings, all fixed in the repository or handed over with a command. What remains is one loop on the owner's
side and then the coder's first commit.
