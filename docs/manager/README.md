# Manager index and status board

Role: project manager, reviewer and release gatekeeper for `auto-tzim`. Not a coder; no product code, tests,
generators or CI implementation are written here (see [MANAGER-SPEC.md §2](../../MANAGER-SPEC.md)).

Date of this board: **2026-10-29**.

## Where the project stands

| Item | State |
| --- | --- |
| Repository | `RUdolf1517/auto-tzim`, public, MIT, initial commit `dc184df` |
| Specification copies | done — [`SPEC.md`](../../SPEC.md) v0.1 and [`MANAGER-SPEC.md`](../../MANAGER-SPEC.md), provenance recorded |
| Launch preconditions (charter §3) | **2 of 6 not met** — see [pre-flight](PREFLIGHT-2026-10-29.md) |
| GitHub issues for AT-01…AT-09 | **not created yet** — blocked on owner authorisation (Q1) |
| Coder assignment | **not issued** — draft only, see [assignment draft](ASSIGNMENT-AT-MVP-DRAFT.md) |
| Product code in this repository | none, by design |
| Decisions awaiting owner | D-01, D-03, D-04, D-05, D-09, D-10, D-13 (see [decisions](DECISIONS.md)) |
| Open questions | [QUESTIONS-TO-OWNER.md](QUESTIONS-TO-OWNER.md) — 4 blocking, 6 secondary |
| Unresolved Critical/High risks | [risk register](RISK-REGISTER.md) — all controls specified, none implemented yet |
| Release readiness | not applicable; nothing is releasable and nothing claims to be |

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
| [ASSIGNMENT-AT-MVP-DRAFT.md](ASSIGNMENT-AT-MVP-DRAFT.md) | Charter §11 assignment for the MVP, **draft, not issued** |

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
