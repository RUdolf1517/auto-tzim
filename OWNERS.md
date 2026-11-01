# Owners and decision rights

Who may decide what, in a project where most of the writing is done by agents and one person holds the consequences.
The normative text is [MANAGER-SPEC.md](MANAGER-SPEC.md) §2 (powers and prohibitions); this page is the short version
that does not require reading a 350-line charter, plus the file-level ownership that the charter leaves implicit.

**Current state:** one human owner, no second maintainer, no bot identity. That single fact shapes everything below —
several controls that would normally be enforced by the platform (an independent approval, a second pair of eyes on a
merge) are enforced here by evidence instead. See [OWNER-ACTIONS](docs/manager/OWNER-ACTIONS.md) §6a and
[D-24](docs/manager/DECISIONS.md).

## Roles

| Role | Who | Does | May not |
| --- | --- | --- | --- |
| **Owner** | [@RUdolf1517](https://github.com/RUdolf1517) | Holds the product contract, safety invariants and the release; answers the questions that change scope; merges to `main`; authorizes a release | — |
| **Manager** | an agent, acting under the charter | Plans, writes documentation, issues and acceptance criteria; reviews the coder's work independently; records evidence and decisions | Write product code, tests, fixtures, generators or CI; fix the coder's code by hand; merge, close issues or release without explicit authorization; relax an invariant for convenience |
| **Coder** | an agent, assigned one consolidated set of issues | Writes the implementation, tests, fixtures, harness and CI; hands over with evidence | Decide its own acceptance; expand the scope of the assignment; touch another role's files (below) without being asked |

The coder and the manager are not the same instance and do not share a session. The manager does not vouch for the
coder's claims: every claim in a hand-over is marked *independently verified*, *source-inspected*, *tool-executed* or
*coder-reported*, and the first three are the only ones that can close an acceptance row
([EVIDENCE-PROTOCOL](docs/manager/EVIDENCE-PROTOCOL.md)).

## Decision rights

| Decision | Owner | Manager proposes | Coder decides | Notes |
| --- | --- | --- | --- | --- |
| Product contract, scope, defaults | **decides** | proposes, with consequences | — | A contract change needs a written answer, recorded in [DECISIONS](docs/manager/DECISIONS.md) |
| Safety invariants (read-only scan, never replace the original, no PTY, no hidden processing) | **decides** | proposes | may not touch | Not relaxed for convenience, a deadline, or a failing test |
| Adding a runtime dependency | **decides** | proposes, with the alternative it replaces | — | [Dependencies](docs/dependencies.md) §5 |
| Merge to `main` | **authorizes** | recommends only after all gates | opens the pull request | Interim rule for documentation-only pull requests: [D-24](docs/manager/DECISIONS.md) |
| Release, tag, published artifacts | **authorizes** | prepares and verifies the package | — | Never implied by a green CI run |
| Creating, editing, closing issues | **authorizes closure** | creates and maintains the set | comments on its own | An issue is closed by evidence, not by finishing a branch |
| Filings in the `tzim` repository | **authorizes** | drafts | — | Our bugs stay here; theirs go there ([UPSTREAM](docs/manager/UPSTREAM.md)) |
| Security reports | **decides** | triages, proposes severity | writes the fix and the test | [SECURITY.md](SECURITY.md) |
| This page and the charter | **decides** | drafts amendments | — | Ownership changes are recorded below, never assumed |

## File ownership

Write access follows the role, not the convenience of whoever is editing:

| Path | Written by | Why |
| --- | --- | --- |
| `MANAGER-SPEC.md`, `SPEC.md`, `docs/spec/**` | owner (manager drafts the change) | Pinned contract copies; a silently edited copy is a second source of truth |
| `docs/**` (except `docs/manager/**`) | manager | Product documentation, reviewed by the owner before a release |
| `docs/manager/**` | manager | Plans, decisions, evidence, reviews |
| `SECURITY.md`, `OWNERS.md`, `.github/CODEOWNERS`, `LICENSE` | owner (manager drafts), nobody else | Governance and reporting channels |
| `README.md`, `CONTRIBUTING.md`, `CHANGELOG.md`, `THIRD-PARTY.md` | manager commits; owner may edit directly | Public face of the project |
| `src/**`, `tests/**`, `.github/workflows/**`, `Cargo.toml`/`Cargo.lock`, `deny.toml`, packaging | coder only | Product code and the machinery that proves it |
| `handoff/**`, `docs/audits/**` | whoever produced the record, then frozen | History, not documentation: it must not be edited to match a later outcome |

A file listed as the coder's may still be *reviewed* by the manager — reviewing is reading and running, never rewriting.
If the manager finds a defect, the defect becomes a finding, and the finding becomes the coder's next commit.

## Escalation

The manager stops and asks the owner when a decision would change the product contract, relax a safety invariant, add a
dependency, authorize a merge or a release, or close an issue. Everything else — ordering work, writing criteria,
rejecting a hand-over, filing a bug in our own tracker — is the manager's job and is expected to happen without a
question.

An answer from the owner is recorded in [DECISIONS](docs/manager/DECISIONS.md) the same day, with the question that
produced it. An undocumented decision is treated as a suggestion the next time it comes up.

## Platform enforcement, and why it is thin

`.github/CODEOWNERS` names the owner for every path, mirroring the `tzim` repository. It is **declarative only**: no
rule currently requires a code-owner review, because the only identity able to approve a pull request is the one that
authors it, and GitHub does not allow self-approval at any permission level. Turning that requirement on would not add a
reviewer; it would only make every merge an administrator bypass. The reason, the three options and the owner's
decision to defer are recorded in [OWNER-ACTIONS](docs/manager/OWNER-ACTIONS.md) §6a and [D-24](docs/manager/DECISIONS.md).

## Change history

| Date | Change |
| --- | --- |
| 2026-11-01 | Page created. Roles, decision rights and path ownership written down; `CODEOWNERS` added as a declaration, not an enforced rule. |
