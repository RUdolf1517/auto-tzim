# Changelog

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), versions follow [Semantic
Versioning](https://semver.org/spec/v2.0.0.html).

Nothing has been released. There is no version of `auto-tzim` that can be installed, and no tag in this repository.

## [Unreleased]

### Added — charter v0.2 and the MVP issue set (2026-10-29)

- `MANAGER-SPEC.md` updated to the owner's **version 0.2**: nineteen invariants (invariant 2 rewritten around a read-only
  candidate plan, 11 around an empty `transforms`, new 18 on fabricated capabilities and 19 on exit codes), and §16
  with the decisions on P-01/P-02/P-03.
- [docs/spec/CONFLICTS.md](docs/spec/CONFLICTS.md): the four places where the older technical specification and the
  charter disagree, with the resolution.
- Nine MVP issues ([#1](https://github.com/RUdolf1517/auto-tzim/issues/1)…[#9](https://github.com/RUdolf1517/auto-tzim/issues/9)),
  the consolidated assignment ([#10](https://github.com/RUdolf1517/auto-tzim/issues/10), prepared and blocked on charter
  §3.2), and one upstream request
  ([RUdolf1517/tzim#86](https://github.com/RUdolf1517/tzim/issues/86)) for a capabilities / structured-reasons API.
- Decision log revised: D-01…D-05, D-18, D-19 (owner, charter v0.2) and D-20…D-23 (plan binding, no PTY, upstream issue,
  candidate plan is not a promise).
- Acceptance matrix revised: 13 acceptance rows and 19 invariants, including the read-only scan proof by invocation
  counter, the `6/7/8/130/143` exit-code matrix and the precedence pairs.
- Product documentation updated for the new contract: scan is a local read-only planner, `tzim --dry-run` is forbidden,
  `skipped_by_tzim` replaces `skipped_refused`, and the exit codes are `0/1/2/3/5/6/7/8/130/143` with `4` unused.

### Added — documentation set

- Specification copies (`SPEC.md` v0.1, `MANAGER-SPEC.md`) with provenance and checksums
  ([docs/spec/PROVENANCE.md](docs/spec/PROVENANCE.md)).
- Manager documentation: launch pre-flight against the charter's six preconditions, an independent study of `tzim`
  (contract surface, machine-interface gaps, behaviour that changes the safety design), a decision log with unapproved
  proposals, questions to the owner, a risk register, the issue breakdown AT-01…AT-09, a pre-registered acceptance
  matrix, and a draft MVP assignment.
- Problem notes P-01…P-03: the missing machine interface in `tzim`, the fact that a `tzim` dry run writes a temporary
  candidate into the destination directory, and the gap/collision in the exit-code tables.
- Product documentation set: architecture, CLI reference, configuration reference, safety model, guarantees, formats,
  state/history/locks, privacy, updates, testing strategy, roadmap, releases, dependencies, limitations, recovery, and
  contributing rules.

### Not implemented

- Everything else. `auto-tzim` v0.1 has not been started: there is no package, no CLI, no test, no CI and no release.
  The owner's decision of 2026-10-29 was to produce the complete documentation first.

## [0.1.0] — not started

Reserved for the MVP described in [`docs/roadmap.md`](docs/roadmap.md) and accepted against
[`docs/manager/GATES.md`](docs/manager/GATES.md).
