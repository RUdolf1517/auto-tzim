# Pre-flight: launch preconditions (charter §3)

Date: **2026-10-29**. Scope: the six preconditions that must be confirmed before the first implementation
assignment. Evidence is labelled per the [legend](README.md#evidence-legend).

## Summary

| # | Precondition | State | Blocks |
| --- | --- | --- | --- |
| P1 | Owner authorised starting `auto-tzim` as a separate project | **unverified** | creating issues, issuing the assignment |
| P2 | Coder finished the previously assigned `tzim` set | **not met** | issuing an assignment (charter §3.2) |
| P3 | Separate repository exists, URL/owner known | **met** | — |
| P4 | `SPEC.md` copied into the repository as a fixed-version document | **met (this turn)** | — (canonical home still open, Q10) |
| P5 | Initial scope limited to MVP / v0.1 (spec §17) | **met as a declaration** | — (no assignment yet) |
| P6 | No changes in the `tzim` repository; missing machine interfaces become an issue plus a fail-closed adapter | **met, with two gaps recorded** | AT-04 design (decisions D-01, D-02) |

Net: the documentation work is complete and defensible, but **the MVP assignment is not issued**, because P1 is
unverified and P2 is factually unmet. This is the intended behaviour of the charter, not a delay caused by the
manager.

---

## P1 — owner authorisation · **unverified**

The repository `RUdolf1517/auto-tzim` exists (created 2026-10-29T19:22:18Z, public, MIT, default branch `main`,
issues enabled), and this session works on branch `arena/5e301052-auto-tzim`. A created repository is not by
itself the "explicit permission to start" that charter §3.1 asks for, and the charter also says the owner is the
only one who can change the product contract. So the authorisation is treated as **unverified** until the owner
answers Q1 in [QUESTIONS-TO-OWNER.md](QUESTIONS-TO-OWNER.md).

**What it blocks:** creating the AT-01…AT-09 GitHub issues and issuing the assignment.
**What it does not block:** documentation, the `tzim` study, risk register, gates and the assignment draft —
all produced here.

## P2 — the previous `tzim` set is finished · **not met**

Charter §3.2 forbids using a new project as an excuse to drop unfinished work. At the time of this pre-flight:

- **PR #84 is open** — `Extend residual FITS acceptance coverage for #78`, branch `arena/99418c32-tzim`, state
  `OPEN`, `MERGEABLE`, updated 2026-10-29T19:26:30Z (`tool-executed`: `gh pr view 84`).
- **Two required checks on that PR are failing** (`tool-executed`: `gh pr checks 84`):
  `measure the frozen 21-PDF set`
  ([run 40939520316](https://github.com/RUdolf1517/tzim/actions/runs/40939520316/job/123679769049)) and
  `re-run the published measurements`
  ([run 40939520233](https://github.com/RUdolf1517/tzim/actions/runs/40939520233/job/123678877629)). The other
  checks listed at that moment pass.
- **Seven issues are open** in `tzim`: #57, #59, #78, #79, #80, #82, #83 (`tool-executed`: `gh issue list`).

This is not a manager's judgement about whether those failures matter for `tzim`; it is only the observation
that the set is not finished, which is exactly what P2 asks about. The owner decides whether #78/#84 are
"finished enough" to start `auto-tzim` in parallel, or whether `auto-tzim` waits.

**What it blocks:** the assignment. Work in this repository stays documentation-only until the owner answers Q1.

## P3 — separate repository · **met**

`gh repo view RUdolf1517/auto-tzim` (`tool-executed`): public, MIT, default branch `main`, issues enabled,
created 2026-10-29T19:22:18Z, description empty. Local clone: initial commit `dc184df`, working branch
`arena/5e301052-auto-tzim`. No `.github/` directory, no workflows, no package, no tags, no releases.

Note for the owner: the repository description is empty while `tzim`'s is a full sentence. Setting it is a
GitHub-settings action the manager does not perform without rights (charter §3, last paragraph).

## P4 — `SPEC.md` copied as a fixed-version document · **met**

`SPEC.md` (419 lines, SHA-256 `250d0db1…f660c`) and `MANAGER-SPEC.md` (341 lines, SHA-256 `25a7d3cd…1959b`) are
in the repository root as byte-identical copies, with source ref, commit `efc31756` and checksums recorded in
[docs/spec/PROVENANCE.md](../spec/PROVENANCE.md) (`tool-executed`: `sha256sum`).

One caveat that needs an owner decision rather than a manager workaround: in `tzim`, both documents live on the
pushed branch `arena/b3d4085f-tzim` and are **not merged into `tzim` `main`** (`main` = `d4e4597`). The versioned
copy therefore has an unmerged home. Q10 proposes merging that docs-only commit before the first implementation
PR, so that "specification version 0.1" means one thing in both repositories.

## P5 — initial scope is MVP / v0.1 · **met as a declaration**

[ROADMAP-ISSUES.md](ROADMAP-ISSUES.md) defines exactly AT-01…AT-09 for MVP, derived from spec §17, and states the
explicit non-goals (scheduler, TUI, Qt, automatic update, backup, cloud sync, own codecs). Later-stage work is
described only as a sketch and is excluded from the MVP assignment. Nothing in this repository implements
anything yet, so the scope is currently a declaration, and the first PR is where it is tested.

## P6 — no changes in `tzim`; gaps become issues plus a fail-closed adapter · **met, with two gaps**

Nothing in this repository asks for a change to `tzim` code. The study
([TZIM-STUDY-2026-10-29.md](TZIM-STUDY-2026-10-29.md)) found two places where the machine interface is weaker than
the specification assumes:

- **G-1** no machine-readable capability/format list (`--info` and `--list-formats` are human text; the spec
  forbids parsing human output);
- **G-2** no structured skip/refusal reason code — only a free-text `note`.

Both are handled fail-closed inside `auto-tzim` (decisions D-01 and D-02) and would be *improved* by two upstream
issues in `tzim`, which the manager can file but does not implement. Charter §3.6 allows exactly this: an issue
plus a fail-closed adapter, no core change.

A third item is a compatibility condition rather than a gap: `tzim` issue #83 proposes interactive confirmation
for opt-in format handling. As soon as a future `tzim` can prompt, charter invariant 12 (no prompt in
scheduler/non-TTY/JSON mode) needs a documented non-TTY refusal contract on the `tzim` side. Until then the
adapter pins a version range that has no prompts at all (`source-inspected`: no `stdin` reference exists anywhere
in `tzim` `src/` at `d4e4597`).

## What the manager did not do

- Did not create GitHub issues, tags, releases or branch protection (waits for P1).
- Did not write product code, tests, CI or fixtures (charter §2.2).
- Did not modify either specification copy, and did not "fix" the discrepancies found in `tzim` documentation.
- Did not run the `tzim` binary: the release assets are unreachable from this session and no Rust toolchain
  exists here. That limitation is declared and is compensated by requiring the pinned binary in AT-08's CI job.
