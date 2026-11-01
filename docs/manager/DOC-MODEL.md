# The documentation model, and how it maps onto `tzim`'s

Checked on **2026-11-01** against `RUdolf1517/tzim` at `f605c2ea` (v0.16.1). The point of the comparison is not symmetry
for its own sake: both repositories are kept by the same person, so a reader who knows one should not have to learn a
second filing system. Where we differ, the table says why.

## Same model, same places

| `tzim` | here | State |
| --- | --- | --- |
| `README.md` — what it is, install, usage, documentation index | [README.md](../../README.md) | match |
| `LICENSE` (MIT) | [LICENSE](../../LICENSE) | match |
| `CHANGELOG.md` — Keep a Changelog, one entry per release | [CHANGELOG.md](../../CHANGELOG.md) | match |
| `CONTRIBUTING.md` — how to build, test, submit | [CONTRIBUTING.md](../../CONTRIBUTING.md) | match |
| `SECURITY.md` — private reporting, scope, disclosure | [SECURITY.md](../../SECURITY.md) | **added 2026-11-01** |
| `THIRD-PARTY.md` — flat licence list for packagers | [THIRD-PARTY.md](../../THIRD-PARTY.md) | **added 2026-11-01** |
| `.github/CODEOWNERS` | [.github/CODEOWNERS](../../.github/CODEOWNERS) | **added 2026-11-01** |
| `docs/cli.md` | [cli](../cli.md) | match — ours documents our CLI, not `tzim`'s |
| `docs/safety.md` | [safety](../safety.md) | match |
| `docs/guarantees.md` | [guarantees](../guarantees.md) | match |
| `docs/formats.md` | [formats](../formats.md) | match |
| `docs/dependencies.md` | [dependencies](../dependencies.md) | match |
| `docs/testing.md` | [testing](../testing.md) | match |
| `docs/releases.md` | [releases](../releases.md) | match |
| `docs/roadmap.md` | [roadmap](../roadmap.md) | match |
| `docs/audits/` — frozen records of what was true at a commit | [audits](../audits/README.md) | **added 2026-11-01**, first entry is the 2026-11-01 contract audit |

## Deferred, with the condition that would make us adopt it

| `tzim` | Why not yet | Adopt when |
| --- | --- | --- |
| `handoff/` — per-iteration evidence, committed | The layout already exists as a contract ([EVIDENCE-PROTOCOL](EVIDENCE-PROTOCOL.md), [testing-harness](../testing-harness.md) §6) and nothing has been produced yet. An empty directory is a promise, and a promise is not evidence | the first real evidence bundle exists (AT-08) |
| `.github/ISSUE_TEMPLATE/` — bug, feature, format request | There is no released software and no external reporter; the issue set here is manager-generated work items | v0.1 is published (AT-09) |
| `man/tzim.1` | We have no command to document | AT-06 defines the CLI |
| `completions/`, `packaging/`, `Formula/` | Distribution work, explicitly outside the MVP | after v0.1 |

## Deliberately not copied

| `tzim` | Why it would be wrong here |
| --- | --- |
| `docs/benchmarks.md`, `docs/measurement-status.md`, `docs/size-budget.md` | Those measure a codec. This project measures almost nothing and adds almost nothing; the honest statement is in [architecture](../architecture.md) and [guarantees](../guarantees.md) — we do not compete with `tzim` on bytes or speed, and inventing a benchmark page would invite comparisons we cannot support |
| `docs/{dicom,fits,flac,pdf}-audit.md` | Format admission is `tzim`'s decision, not ours. We have no format opinions to audit; we have a delegation policy to verify |
| `src/`, `tests/`, `fuzz/`, `.github/workflows/` | Real, but they belong to the coder (AT-01…AT-08), not to this documentation pass |

## Ours, with no `tzim` equivalent

| Document | Why it exists only here |
| --- | --- |
| [architecture](../architecture.md), [cli](../cli.md), [config](../config.md), [state](../state.md), [recovery](../recovery.md), [update](../update.md), [privacy](../privacy.md) | We are an orchestrator: we have state, history, folders, a plan, a lock and one network request. `tzim` has none of those problems |
| [limitations](../limitations.md), [faq](../faq.md) | The failure modes of a tool that moves files are different from the failure modes of one that rewrites them |
| [testing-harness](../testing-harness.md) | We test *against* a binary, so the fake binary, its invocation log and the case runner are part of the contract. `tzim` does not need a fake of itself |
| `docs/spec/` — pinned copies with provenance and SHA-256 | The contract was written in another repository; a copy without provenance is a rumour |
| `docs/manager/` — pre-flight, decisions, gates, corpus, review protocol, status log | The project is run under a written charter. Without these, the charter would be the only record, and a 350-line document is a poor audit trail |
| `MANAGER-SPEC.md`, `SPEC.md` | The pinned contract itself |

## Rules that carry over unchanged

From `tzim`'s documentation practice, adopted here because they are the reason its documents can be trusted:

1. **English**, everywhere, including audits.
2. **State the limits in the document that makes the claim.** "Not executed" is written down, not implied away.
3. **One claim, one place.** A fact lives in one document; the others link to it.
4. **Changelog per release**, in Keep a Changelog form, with the reason and the evidence, not just the change.
5. **Audits are frozen.** The product document changes when the finding is fixed; the audit keeps saying what it said.
6. **The documentation index is the front door.** `README.md` lists every document with one line about its contents, so
   nobody has to guess a filename.

## Rule that exists only here

Every product document opens with a **status line** — *"Status: specification, no implementation"* — until AT-09 declares
the release ready. `tzim` does not need this because its documents describe a working binary. Ours would otherwise read as
a description of software that does not exist, which is the most misleading thing documentation can do.

## Outstanding

The 2026-11-01 audit found [ten drift items](../audits/2026-11-01-tzim-v0.16.1.md) between these documents and `tzim`
v0.16.1. They are contract corrections, not filing-system problems: this model is now aligned; the content of six
documents inside it is not.
