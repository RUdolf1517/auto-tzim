# Changelog

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), versions follow [Semantic
Versioning](https://semver.org/spec/v2.0.0.html).

Nothing has been released. There is no version of `auto-tzim` that can be installed, and no tag in this repository.

## [Unreleased]

### Fixed — the leftovers of the sweep, and the audit closed (2026-11-02)

A re-read of every document against the decisions, instead of trusting the sweep that produced them. It found four
things that were still wrong, and they are worth naming because each one is the kind of error that survives review:

- **The v0.1 network claim was alive in nine files.** [README](README.md), [SECURITY](SECURITY.md),
  [guarantees](docs/guarantees.md) (A2 and A6), [architecture](docs/architecture.md), [faq](docs/faq.md),
  [CONTRIBUTING](CONTRIBUTING.md) and [cli](docs/cli.md) §2.11 still described the single `update-check` request that
  [D-28](docs/manager/DECISIONS.md) removed, and [config](docs/config.md) still said a key was "used only by
  `update-check`". `update-check` is now documented as a command that **does not exist in v0.1** and exits `2` if typed,
  which is what AC-9 and corpus `W-02` already asserted. The documents disagreed with the gate that tests them; the gate
  was right.
- **Python idioms had survived in five acceptance rows.** `shell=False`, `stdin=DEVNULL`, `import urllib.request`,
  `socket.socket`, `os.replace` and "no `pty` import" were still the evidence demanded by AC-5, INV-8, INV-14 and INV-15
  after [D-33](docs/manager/DECISIONS.md). They now demand Rust: `std::process::Command` with one `.arg()` per item, no
  `sh -c`, `Stdio::null()`, a source check that keeps `std::net` behind one test guard, and a planted
  `TcpStream::connect` as the control that proves the check can fail.
- **The assignment still told the coder to build SQLite.** Step 5 of
  [ASSIGNMENT-AT-MVP-DRAFT](docs/manager/ASSIGNMENT-AT-MVP-DRAFT.md) said "SQLite state" and `shell=False` — in the file
  the coder is meant to work from, one day after [D-35](docs/manager/DECISIONS.md) removed the database. Fixed, and the
  D-35 pointer is now explicit.
- **`reason_code` was decided but never written down.** [D-27](docs/manager/DECISIONS.md) surfaced it as `reason`;
  `cli.md` did not mention the field at all. It is now in §2.8 and §3, and corpus `A-12` gives an unknown code a case.

### Added — the 2026-11-01 audit is closed, with the check behind each finding

[docs/audits/2026-11-01-tzim-v0.16.1.md](docs/audits/2026-11-01-tzim-v0.16.1.md) carries `Status: closed` and a
close-out table mapping all ten findings to the document that now carries each one. The finding text is unchanged, as
that folder's rule requires. The close-out says what it does not claim: **nothing was executed**, so it closes actions,
not behaviour — the pinned binary in AT-08 is the first thing that can test the contract.

Three corpus rows were added the same day — `A-11` (`--capabilities`: good document, wrong capability schema, wrong
result schema, flag rejected, missing key), `A-12` (an unknown `reason_code` passed through, never mapped) and `X-17`
(`EPIPE` does not replace an earned exit `1`) — and two were reworded (`A-01` now takes its version from the pin file,
`R-01` now names the prompting child it stands for). Adding rows to a frozen corpus is allowed here for one reason only:
**there is no code and no result yet**, so nothing is being edited *after* a result is known. The reason is written in
the commit and in the close-out.

### Not observed — the two owner actions, measured rather than accepted

The owner reported §6a (drop the required review) and §6b (refresh issue #10) done on 2026-11-02. Measured rather than
accepted: issue #10's `updatedAt` is still `2026-10-29T20:13:58Z` and its body is byte-identical to the stale 122-line
copy (the file is 170); `main` is
still `protected: true`, and pull request #16 still reports `BLOCKED` with no status checks configured, which leaves a
required review as the only thing that can block it. Both are recorded in [OWNER-ACTIONS](docs/manager/OWNER-ACTIONS.md)
as **not observed**, with the three commands that settle each — `CLEAN` on a checkless pull request, and a changed
`updatedAt`, are the two signatures of success.

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

### Added — pre-registration completed

- [FIXTURE-CORPUS.md](docs/manager/FIXTURE-CORPUS.md): the corpus frozen before implementation — ~110 cases by ID
  (`N-*` names, `K-*` kinds, `S-*` stability, `P-*` policy, `A-*` adapter, `R-*` runner, `L-*` locks, `X-*` exit codes,
  `W-*`/`D-*` network and docs) with the expected outcome of each and the gate row it proves.
- [EVIDENCE-PROTOCOL.md](docs/manager/EVIDENCE-PROTOCOL.md): how a handoff is verified — SHA binding, CI runs, artifact
  checksums, row reconciliation with `jq`, byte evidence, the read-only-plan proof, the executed exit-code matrix, six
  negative controls the reviewer performs personally, and the four claim marks.

### Changed — no C in the build, and state is plain files

`rusqlite` is gone ([D-35](docs/manager/DECISIONS.md)); state and history are JSONL files and atomically replaced JSON
under `AUTO_TZIM_HOME`. The trade is stated rather than implied: no SQL queries and no cross-file transactions, in exchange
for a build with no C in it and a history a user can read with `cat`. The three primitives that replace a database —
`atomic_write`, `append_line`, `create_exclusive` — and the single-writer rule they rest on are in
[state.md](docs/state.md) §2. Recovery, retention, schema versioning and the exit code for corrupt state are unchanged; the
fixture cases `L-04`/`L-06` now describe a malformed JSONL line instead of a corrupt database.

### Changed — `cargo install`, and the crate keeps its hyphen

[D-36](docs/manager/DECISIONS.md): distribution is GitHub releases + Homebrew + `cargo install auto-tzim`. The CLI crate is
renamed from `auto-tzim-cli` to `auto-tzim` so that the published name produces the right binary — a crate name is forever,
so it is decided before publication rather than after. A `cargo`-installed binary is updated by asking `cargo`, not by
replacing the file, because cargo keeps its own bookkeeping in `.crates.toml`.

### Changed — the implementation language is Rust

The owner's decision, 2026-11-01 ([D-33](docs/manager/DECISIONS.md)): `auto-tzim` is a Rust workspace, not a Python
package. `SPEC.md` §4.1 recommends Python 3.12+, and it is **not edited** — it is pinned with a recorded SHA-256, so the
override is recorded as a decision and the conflict is filed as [C-5](docs/spec/CONFLICTS.md). Every document that
described the Python stack was rewritten the same day: dependencies, releases, testing harness, contributing,
architecture, third-party notices, and the acceptance rows that asserted "no `urllib` import".

What the change buys, concretely: installation is a static binary with no interpreter and no environment to corrupt;
self-update becomes the same atomic `rename` the `tzim` repository already uses, instead of surgery on `site-packages`;
and the zero-network invariant stops being a test and becomes a property of the artifact — there is no HTTP client in
`std`, so an outbound request would have to appear as a dependency in `Cargo.lock`, where CI sees it.

What it costs: the Qt GUI, which loses mature `PySide6` and gains `cxx-qt`. [D-34](docs/manager/DECISIONS.md) answers that
by splitting the artifact rather than by pretending the cost is small — three front doors (CLI, TUI, Qt), three binaries,
three installs, one core crate.

### Changed — the roadmap is versioned

[roadmap.md](docs/roadmap.md) rewritten around versions instead of features: each one states what a user gets, the new
risk it introduces, what is deliberately out of scope, the gate that has to be passed, and what would stop it. v1.0 is
defined as the point where the contract stops moving rather than as a feature release, and v0.2+ carries no issue set yet —
deliberately, because a breakdown written before v0.1 ships describes software nobody has used.
`update-check` moved to v0.2 in [update.md](docs/update.md) with D-28, and the manager page gained the versions-to-issues
mapping.

### Changed — four product decisions, and what they cost

- **The adapter reads `tzim --capabilities`** ([D-26](docs/manager/DECISIONS.md)) and fails closed on an unexpected schema
  pair. Cost: a scan is now two spawns, so the harness counting rule changed with it rather than failing a correct build.
- **Refusal reasons are surfaced** ([D-27](docs/manager/DECISIONS.md)): a `reason` field beside the verbatim `note`; the
  prose is still never parsed.
- **`update-check` leaves the MVP** ([D-28](docs/manager/DECISIONS.md)): v0.1 makes no network request at all, `urllib`
  leaves the dependencies, exit `5` becomes reserved, and charter §17.5 is overridden without editing the pinned `SPEC.md`.
- **Two operating modes** ([D-29](docs/manager/DECISIONS.md)): `confirm` (default — a proposal, then install on approval)
  and `auto` (no prompt). One install path, staged copies, a journaled swap, and a rule that an original is deleted only
  after its verified replacement is in place. Invariant 12 is rewritten with the single approval gate inside it; invariant
  16 stays true and gains the ordering guarantee beside it.

New acceptance rows AC-14…AC-17, invariants 20…23, and corpus cases `M-01`…`M-14` carry the mechanics.

### Added — the repository files `tzim` has, and we did not

- [SECURITY.md](SECURITY.md): private reporting, and a scope written for a tool whose job is to move files — a write to
  the source folder during a scan is a vulnerability, a `tzim` format bug is not.
- [OWNERS.md](OWNERS.md) and [.github/CODEOWNERS](.github/CODEOWNERS): who decides what, which role may write which path,
  and why `CODEOWNERS` is a declaration and not an enforced rule while the only identity able to approve is the author.
- [THIRD-PARTY.md](THIRD-PARTY.md): the licence list for packagers — one runtime dependency, and `tzim` invoked rather
  than linked.
- [docs/audits/](docs/audits/README.md): the frozen-audit convention, adopted from `tzim`, with its first entry.
- [docs/manager/DOC-MODEL.md](docs/manager/DOC-MODEL.md): every document here mapped onto `tzim`'s, with the reason for
  each difference and for each omission.

### Found — the contract moved under the documentation

- [docs/audits/2026-11-01-tzim-v0.16.1.md](docs/audits/2026-11-01-tzim-v0.16.1.md): `tzim` v0.16.0 shipped
  `--capabilities`, a stable `reason_code` and JSON result schema 2; v0.16.1 is the current release. Ten findings, six
  severity A, each with the affected `file:line`. Six documents carry a banner pointing at it.
- **B-01 is cleared**: `tzim` PR #84 merged on 2026-10-30, the upstream tracker has no open issues or pull requests, and
  three releases shipped. Charter §3.2 is satisfied — the start is now gated by the drift, not by the blocker
  ([D-25](docs/manager/DECISIONS.md)).

### Changed — documentation now lives on `main`

- The complete documentation set was transferred to `main` by pull request
  [#11](https://github.com/RUdolf1517/auto-tzim/pull/11) (merge commit `1328919`, 44 files, markdown only). `main` is a
  protected branch with a required review and GitHub forbids approving one's own pull request, so the merge used
  `--admin` on the owner's instruction; the governance consequence is written down in
  [docs/manager/OWNER-ACTIONS.md](docs/manager/OWNER-ACTIONS.md) §6a rather than left implicit.

### Added — the missing half of the records

- [testing-harness.md](docs/testing-harness.md): the harness contract the review protocol already relied on — the fake
  `tzim` scenario switchboard, the invocation-log schema (the proof that a scan is read-only), `run-case <ID>`, the pinned
  binary fetcher with checksum enforcement, and the evidence bundle layout.
- [HANDOFF-TEMPLATE.md](docs/manager/HANDOFF-TEMPLATE.md) and [REVIEW-TEMPLATE.md](docs/manager/REVIEW-TEMPLATE.md): charter
  §12 as fill-in forms, plus the four claim marks and the single allowed verdict.
- [UPSTREAM.md](docs/manager/UPSTREAM.md): the `tzim` items that affect this project (`#85` canonical, `#86` my duplicate,
  `#83` prompt contract, `#82` JPEG default, PR #84), what we rely on, and how a change would become visible.
- [STATUS-LOG.md](docs/manager/STATUS-LOG.md): dated record of every action and decision on 2026-10-29, including the
  duplicate-issue mistake and its root cause.
- [docs/faq.md](docs/faq.md): short answers with pointers, for users and for the next reviewer.

### Fixed — a duplicate upstream issue of my own making

- `tzim#86` (filed by this project) duplicates `tzim#85`, which already described the same capabilities /
  structured-reasons request and was created twelve minutes earlier. #85 is canonical; #86 should be closed. The
  reference in assignment #10 to `#85` was correct and my earlier "correction" of it was the error. Root cause: the
  upstream issue was filed without re-listing the open issues first.

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
