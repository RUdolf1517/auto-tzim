# Pre-flight: launch preconditions (charter §3)

Date: **2026-10-29**, revised after the owner's charter **v0.2** (which decided findings P-01…P-03) and after the issue
set was created. Scope: the six preconditions that must be confirmed before the first implementation assignment.
Evidence is labelled per the [legend](README.md#evidence-legend).

## Summary

| # | Precondition | State | Blocks |
| --- | --- | --- | --- |
| P1 | Owner authorised starting `auto-tzim` as a separate project | **met** — charter v0.2 states the first managerial result is an issue set, a full assignment and an acceptance plan; the owner instructed that work directly | nothing |
| P2 | The coder finished the previously assigned `tzim` set | **not met** — PR #84 open with two failing jobs, seven issues open | the **start** of implementation (B-01); not the preparation |
| P3 | Separate repository exists, URL/owner known | **met** | — |
| P4 | `SPEC.md` copied as a fixed-version document | **met**, plus `MANAGER-SPEC.md` v0.2 and a conflict resolution page | — |
| P5 | Initial scope limited to MVP / v0.1 | **met** — nine issues, non-goals stated in each | — |
| P6 | No changes in `tzim`; missing machine interfaces become an issue plus a fail-closed adapter | **met** — charter §16 P-01 accepts the restricted adapter; upstream request filed as `RUdolf1517/tzim#86` | — |

The deliverable the charter asks for is complete: nine issues, one consolidated assignment, one pre-registered acceptance
matrix. The assignment is **filed but not started**, because P2 is a fact about another repository and cannot be
resolved from here.

---

## P1 — owner authorisation · **met**

Charter v0.2 is an authorisation to run the project as manager: it fixes the issue roadmap (§4), the workflow (§5), the
invariants (§6), the evidence standard (§7), the gates (§8) and the release rules (§14), and it decides P-01…P-03 (§16).
The owner's accompanying instruction was to produce the issue set, one full coder assignment and a verifiable acceptance
plan for the MVP — which is what this repository now contains.

What was created under that authorisation: issues [#1](https://github.com/RUdolf1517/auto-tzim/issues/1) …
[#9](https://github.com/RUdolf1517/auto-tzim/issues/9) (AT-01…AT-09), the assignment
[#10](https://github.com/RUdolf1517/auto-tzim/issues/10), and one upstream request
[RUdolf1517/tzim#86](https://github.com/RUdolf1517/tzim/issues/86). No branch protection, no tags, no releases, no
repository settings: those remain the owner's.

## P2 — the previous `tzim` set is finished · **not met**

Charter §3.2 forbids using a new project as a reason to drop unfinished work. At the time of this pre-flight:

- **PR #84 is open** (`Extend residual FITS acceptance coverage for #78`, branch `arena/99418c32-tzim`, state `OPEN`,
  `MERGEABLE`, updated 2026-10-29T19:26:30Z) — `tool-executed`: `gh pr view 84`.
- **Two required checks on it fail** — `tool-executed`: `gh pr checks 84`:
  `measure the frozen 21-PDF set` ([run 40939520316](https://github.com/RUdolf1517/tzim/actions/runs/40939520316/job/123679769049))
  and `re-run the published measurements` ([run 40939520233](https://github.com/RUdolf1517/tzim/actions/runs/40939520233/job/123678877629)).
- **Seven issues are open**: #57, #59, #78, #79, #80, #82, #83 — `tool-executed`: `gh issue list`.

**Effect:** implementation, tests and fixtures do not start. The blocker is filed in the charter §11 format as **B-01**
in [issue #10](https://github.com/RUdolf1517/auto-tzim/issues/10) and in
[the assignment](ASSIGNMENT-AT-MVP-DRAFT.md#blocker-b-01-charter-10-format): *merge or explicitly park PR #84, then
comment "start" on #10.*

**What is not blocked:** the issue set, the acceptance matrix, the risk register, the documentation and this pre-flight.

## P3 — separate repository · **met**

`gh repo view RUdolf1517/auto-tzim` (`tool-executed`): public, MIT, default branch `main`, issues enabled, created
2026-10-29T19:22:18Z, description empty. Labels `v0.1`, `manager`, `assignment`, `upstream` created for this project.
Still the owner's to set: the repository description, topics, and branch protection.

## P4 — specification copies with a fixed version · **met**

- `SPEC.md` — technical specification **0.1**, 419 lines, SHA-256 `250d0db1…f660c`, from `tzim` branch
  `arena/b3d4085f-tzim`, commit `efc31756`.
- `MANAGER-SPEC.md` — charter **0.2**, 352 lines, SHA-256 `7e798361…43f1`, supplied by the owner on 2026-10-29 with the
  decisions on P-01…P-03.

Both are recorded in [PROVENANCE.md](../spec/PROVENANCE.md). Because the charter's §16 overrides four points of the older
technical specification, the resolutions are written down once in [CONFLICTS.md](../spec/CONFLICTS.md) so that no coder
has to choose per pull request.

## P5 — scope is MVP / v0.1 · **met**

Nine issues, each with scope, acceptance criteria, required tests, non-goals and required artifacts
([ROADMAP-ISSUES.md](ROADMAP-ISSUES.md)). Later stages are sketched only and explicitly excluded from the MVP pull
request. The charter's changed AT-04 definition — a restricted adapter with a read-only candidate plan — is what the
issues say, not the older technical specification's planner.

## P6 — no `tzim` changes; gaps become an issue and a fail-closed adapter · **met**

- Charter §16 P-01 accepts the missing capability/reason API as a **limitation**, not a blocker: the default set is
  delegated without `--formats`, `transforms` stays empty, nothing is parsed from prose, `skipped_by_tzim` stays generic,
  and an unknown contract is `incompatible_tzim` (exit `3`).
- The upstream request is filed as [RUdolf1517/tzim#86](https://github.com/RUdolf1517/tzim/issues/86) — a description of
  a future `--capabilities --json` and a structured reason code, explicitly not a pull request and not a dependency.
- No change to `tzim` code, tests, CI or settings was made or requested in this project.

## What the manager did not do

- Did not start implementation, write product code, tests, fixtures or CI (charter §2.2).
- Did not edit issue #10 after creation (the integration returns 403 for edits and comments); the one wrong reference in
  its body is corrected in [ROADMAP-ISSUES.md](ROADMAP-ISSUES.md) and [the assignment](ASSIGNMENT-AT-MVP-DRAFT.md)
  instead of being hidden.
- Did not run the `tzim` binary: release assets are unreachable from this session and no Rust toolchain exists here.
  Every `tzim` claim is `source-inspected` against `d4e4597`; behavioural confirmation belongs to AT-08 in CI.
- Did not touch `tzim`'s branches, PRs or settings beyond filing the one requested upstream issue.
