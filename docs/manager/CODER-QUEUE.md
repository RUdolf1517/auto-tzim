# The coder's queue — maximum work, minimum waiting

Written 2026-11-02 at `main` = `6c55db2`. The owner asked for the coder to be given as much work as possible. This page is
that: every stage that can be worked on now, in the order they must be done, with the evidence each one is judged by — so
the coder never waits for a manager to write the next brief.

**Read this page first, then the brief for the stage you are on.** The brief is the contract; this page is the schedule.

## 0. Rust. The implementation language is Rust, and it is not negotiable

This is repeated at the top because it has already been argued once and it will not be argued again:

| Fact | Where it comes from |
| --- | --- |
| The product is **Rust**: a workspace with `auto-tzim-core` (library) and `auto-tzim` (binary) | [D-33](DECISIONS.md), adopted by the owner 2026-11-01 |
| `SPEC.md` §4.1 still recommends Python and **must never be edited** — the override lives in the decision log and in [CONFLICTS C-5](../spec/CONFLICTS.md) | D-33 |
| Toolchain **1.89.0**, pinned in `rust-toolchain.toml`, in both manifests (`rust-version`) and in CI. Do not use a standard-library API that arrived after it | [D-40](DECISIONS.md), [D-53](DECISIONS.md) |
| **No Python in the product.** Two exceptions exist and both are CI-only: `scripts/check-pins.py` and its self-test | [D-41](DECISIONS.md) |
| The fake `tzim`, the case runner and the pinned-binary fetcher are **one Rust binary**, `auto-tzim-harness` — not a shell script, not Python | [dependencies](../dependencies.md) §2a, [testing-harness](../testing-harness.md) |
| No crate that compiles C, no crate that can open a socket, every version `=X.Y.Z`, every crate on the allowlist | [D-35](DECISIONS.md), [D-38](DECISIONS.md) |
| A new dependency is **two commits** — the dependency, then the `deny.toml` allowlist entry — and `cargo tree -e features` in the pull request | [dependencies](../dependencies.md) §2a |

Mechanics that follow from Rust, and are review items:

- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --locked` —
  all three already run in CI on both platforms. They are not advisory.
- **No `unsafe`, no `libc`, no wrapper crate for things `std` does.** The file lock is `std::fs::File::try_lock` — that is
  why the MSRV is 1.89 ([D-53](DECISIONS.md)). `unsafe` and `libc` still need a decision first, and nothing in v0.1 needs them.
- **`std::process::Command` lives in `runner.rs` and nowhere else.** `std::fs` writes live in `config.rs`, `state.rs` and
  `staging.rs`. A `use std::process` in `discovery.rs` is a review finding even if the code is correct, because the
  boundary is what makes the promise checkable.
- Errors are **typed enums** in the core, not `anyhow`/`thiserror` — adding one is a dependency decision, not a
  convenience. The CLI layer maps a typed error to an exit code ([D-18](DECISIONS.md), [D-19](DECISIONS.md)); the core
  never calls `std::process::exit`.
- `unwrap()`/`expect()` on anything derived from user input (a path, a config value, a subprocess result) is a defect. On a
  value the type system proves infallible it is fine.
- Paths are `PathBuf` internally and are rendered with `to_string_lossy()` **for display only**; a non-UTF-8 name must never
  panic (corpus `N-17`).
- `std::os::unix::fs::MetadataExt` (`dev`, `ino`, `nlink`, `mtime_nsec`) is the identity source. Both target platforms are
  Unix; gate with `cfg(unix)` and fail to compile elsewhere rather than silently doing something else.

## 1. How work is delivered

| Rule | Detail |
| --- | --- |
| **One stage, one pull request** | A PR that delivers AT-03 and half of AT-04 is returned unread. The stages are sized so that each is reviewable in one sitting |
| **Commits labelled** | `AT-0N: …`, one idea per commit, dependencies as their own commits |
| **The handoff block** | [HANDOFF-TEMPLATE](HANDOFF-TEMPLATE.md), every line filled, pasted into the PR |
| **Negative controls** | Every stage names the check that would fail if the feature were broken, and shows it going red **on purpose**, with the run id |
| **You do not merge** | `Merge/Release performed by coder: no`. The owner merges; the manager reviews |
| **You do not edit the gate documents** | `GATES.md`, `DECISIONS.md`, the corpus expectations, the acceptance rows — and that includes the `HANDOFF-*.md` briefs you work from, which are the manager's copy of the contract. If one is wrong, that is a finding in the PR, not an edit: an edit to the acceptance rows makes the review compare the code against a contract the coder just rewrote |

## 2. The queue

Nothing here is optional and nothing here is out of order. **A stage may not start before the previous one is accepted** —
merged is not enough; the review has to have said PASS. That is the cost of the gate, and it is the reason a stage is never
half-built on top of an unaccepted one.

| # | Stage | Brief | State |
| --- | --- | --- | --- |
| 1 | **AT-02** — config, atomic storage, folder CRUD, **Part B** (the macOS queue), **F-18** (the metadata check) | [HANDOFF-AT-02](HANDOFF-AT-02.md) | **ACCEPTED 2026-11-05 at `d0ac17e`**, merged `70ab3d0`, run 42087393439 green in `main` |
| 2 | **AT-03** — safe discovery, stability, and the read-only candidate plan | [HANDOFF-AT-03](HANDOFF-AT-03.md) · [KICKOFF](KICKOFF-AT-03.md) | **started 2026-11-06 — start here** |
| 3 | **AT-04** — restricted `tzim` adapter, policy, planner | expand from [ROADMAP](ROADMAP-ISSUES.md) §AT-04 into a brief before starting; the harness's fake `tzim` is built here | queued |
| 4 | **AT-05** — runner, state, history, cancellation, staging and the crash-safe install | queued; also owns `state/last-compression.json` and the `--time` cooldown ([D-47](DECISIONS.md)) | queued |
| 5 | **AT-06** — CLI and the exit-code contract, including the path-first one-shot form and the batch rules | queued | queued |
| 6 | **AT-08** — verification with the fake and the real `tzim`, the evidence bundle | queued; completes the harness started in AT-04 | queued |

`AT-07` (update-check) is v0.2 and must not be built — `update-check` does not exist in v0.1 and invoking it exits `2`
([D-28](DECISIONS.md)). `AT-09` is release readiness, and it is the owner's, not a stage the coder starts.

## 3. What you may do without asking

Everything below is inside the assignment. Do it, put it in the PR, and do not wait:

- add **tests** and corpus cases — new IDs are always allowed (the corpus forbids removals and expectation changes, never
  additions);
- refactor within a stage you are already working on, as long as no documented behaviour changes;
- fix anything in CI that is red or flaky, with the run ids as evidence;
- write rustdoc on public items in `auto-tzim-core` — the crate is the contract for the TUI and Qt front doors later;
- improve error messages, as long as the documented reason codes and exit codes stay exactly as they are;
- append to the corpus an adversarial case you discovered while testing, even if it is not in the list.

## 4. What needs a decision first — stop and ask

A one-line question in the pull request is enough; the manager answers in the decision log. Asking costs an hour. Guessing
costs a rewrite, and in this project it can cost a user's file.

- a **new dependency**, a new feature flag on an existing one, or anything that needs an allowlist entry;
- a **new CLI flag, subcommand, config field or output field** — the CLI, `config.md`, `cli.md` and `state.md` are contract;
- a **new status value, reason code or exit code**;
- anything that touches a **gate document**, a corpus expectation, or an acceptance row;
- **`unsafe`**, `libc`, a thread pool, async, or a signal handler beyond `TERM`/`KILL` on a child;
- anything that would make two of our processes touch one file;
- anything that writes inside a user's folder before the install step.

## 5. If a stage runs longer than a week

Stop and write one paragraph in the pull request: what is left, what is blocking, what you tried. A stage that quietly
becomes three weeks is a stage whose design was wrong, and the paragraph is how that gets found before it becomes a rewrite.
The manager's job is to unblock, not to be surprised.

## 6. Why the order is what it is

Config first, because every later stage reads it and a schema mistake is expensive to undo. Discovery second, because it is
the stage that decides what a user's folder even contains, and it must be provably read-only before anything writes. The
adapter, runner and CLI after that, because they are the parts whose failures are loud. Verification last, because a fake
`tzim` written before there is anything to call it is a test of nothing.

The parts that can overlap — the harness's fake `tzim` with AT-04's adapter, the corpus cases with AT-03's discovery — are
allowed to overlap, and the briefs say so where it applies.
