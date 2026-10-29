# Changelog

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), versions follow [Semantic
Versioning](https://semver.org/spec/v2.0.0.html).

Nothing has been released. There is no version of `auto-tzim` that can be installed, and no tag in this repository.

## [Unreleased]

### Added

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
