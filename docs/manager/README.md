# Manager index and status board

Role: project manager, reviewer and release gatekeeper for `auto-tzim`. Not a coder; no product code, tests,
generators or CI implementation are written here (see [MANAGER-SPEC.md §2](../../MANAGER-SPEC.md)).

Date of this board: **2026-10-29** (updated after the owner's charter v0.2).

## Where the project stands

| Item | State |
| --- | --- |
| Repository | `RUdolf1517/auto-tzim`, public, MIT, initial commit `dc184df` |
| Specification copies | `SPEC.md` v0.1 (from `tzim`), `MANAGER-SPEC.md` **v0.2** (owner, 2026-10-29) — [provenance](../spec/PROVENANCE.md), [conflicts resolved](../spec/CONFLICTS.md) |
| Charter v0.2 findings | P-01/P-02/P-03 **decided by the owner** (charter §16); recorded as D-01…D-05, D-18, D-19 |
| GitHub issues AT-01…AT-09 | **created** — [#1](https://github.com/RUdolf1517/auto-tzim/issues/1) … [#9](https://github.com/RUdolf1517/auto-tzim/issues/9) |
| Consolidated assignment | **prepared and filed** as [#10](https://github.com/RUdolf1517/auto-tzim/issues/10); start blocked by B-01 (charter §3.2) |
| Upstream request | [RUdolf1517/tzim#86](https://github.com/RUdolf1517/tzim/issues/86) — capabilities + structured reasons; not a dependency |
| Acceptance matrix | [GATES.md](GATES.md) — 13 acceptance rows, 19 invariants, pre-registered |
| Risk register | [RISK-REGISTER.md](RISK-REGISTER.md) — 26 rows, four of them promoted to charter requirements in v0.2 |
| Product documentation | 16 documents under `docs/`, all marked "specification, no implementation" |
| Product code in this repository | none, by design |
| Decisions awaiting owner | D-07 (update source), D-09 (hash cap), D-15 (dependency pin) — manager defaults in force |
| Release readiness | not applicable; nothing is releasable and nothing claims to be |
| Known GitHub-permission limits | (1) #10's body cites `tzim#85`; the correct number is **#86**. (2) The labels `v0.1`, `manager`, `assignment`, `upstream` were created but cannot be attached to issues. The integration here may create issues and labels, but every edit, comment and label-attach call returns HTTP 403 "Resource not accessible by integration". Both corrections are recorded in the repository ([ROADMAP-ISSUES.md](ROADMAP-ISSUES.md)) and each needs one manual action: edit one line in #10, attach `v0.1` to #1…#9 and `assignment`+`manager` to #10 |

## Documents

| Document | Purpose |
| --- | --- |
| [PREFLIGHT-2026-10-29.md](PREFLIGHT-2026-10-29.md) | Charter §3 preconditions, each with evidence and its blocking effect |
| [TZIM-STUDY-2026-10-29.md](TZIM-STUDY-2026-10-29.md) | Independent study of `tzim`: the contract we may rely on, and the machine-interface gaps |
| [DECISIONS.md](DECISIONS.md) | Decision log; proposals are marked as such and are not requirements until approved |
| [QUESTIONS-TO-OWNER.md](QUESTIONS-TO-OWNER.md) | What only the owner can answer, with a recommended default for each |
| [RISK-REGISTER.md](RISK-REGISTER.md) | Risks, levels, the control that closes each, and the evidence that proves it |
| [ROADMAP-ISSUES.md](ROADMAP-ISSUES.md) | AT-01…AT-09 with acceptance criteria and required evidence; later stages sketched only |
| [GATES.md](GATES.md) | Pre-registered MVP acceptance matrix: specification clause → behavioural evidence |
| [problems/](problems/README.md) | The three problem notes P-01…P-03, now **decided** — kept as the analysis behind D-02, D-04 and D-18/D-19 |
| [ASSIGNMENT-AT-MVP-DRAFT.md](ASSIGNMENT-AT-MVP-DRAFT.md) | Charter §11 assignment for the MVP — **prepared, filed as #10, not started** |

Product documentation lives outside this directory, in `docs/`: [architecture](../architecture.md), [cli](../cli.md),
[config](../config.md), [safety](../safety.md), [guarantees](../guarantees.md), [formats](../formats.md),
[state](../state.md), [privacy](../privacy.md), [update](../update.md), [testing](../testing.md),
[roadmap](../roadmap.md), [releases](../releases.md), [dependencies](../dependencies.md),
[limitations](../limitations.md), [recovery](../recovery.md).

## Evidence legend

Every factual claim in these documents is labelled, per charter §12, as one of:

- **independently verified** — the manager re-derived it from raw material (checksums, JSON rows, aggregates,
  CI logs at a named SHA);
- **source-inspected** — read in source code or documentation at a named commit, not executed;
- **tool-executed** — produced by a command the manager ran, with the command recorded;
- **coder-reported** — asserted by whoever wrote the code, not yet checked.

Claims in the pre-flight and study documents are `source-inspected` or `tool-executed`; nothing in this
repository is `coder-reported` yet, because no code exists.

## Environment limits of the manager session (declared, not a blocker for documentation)

- `tzim` could **not be executed** here: the release assets are served from
  `release-assets.githubusercontent.com`, which this session cannot reach, and no Rust toolchain is available.
  Every `tzim` claim is therefore `source-inspected` against the tree at `d4e4597`, not `tool-executed`.
  Behavioural confirmation of the pinned binary is an MVP task (AT-08) and a CI gate.
- The local interpreter is Python 3.11.2; the project targets 3.12+. That is an analysis detail only — CI must
  pin its own interpreter.
