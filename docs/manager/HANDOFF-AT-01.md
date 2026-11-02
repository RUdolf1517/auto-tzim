# Brief — AT-01: the first commit of code

Written 2026-11-02, at `main` = `95325af`. This is the coder's brief for **AT-01 only**. It is not the handoff
[format](HANDOFF-TEMPLATE.md) and does not replace it: the coder fills that in when the work is done.

Read [ASSIGNMENT-AT-MVP-DRAFT](ASSIGNMENT-AT-MVP-DRAFT.md) for the whole MVP. **For this task, read it for context and
build only what is below.** The assignment covers nine issues; delivering more than AT-01 is a review failure, not
generosity.

## 1. What AT-01 is for

AT-01 settles two things that are expensive to change later and cheap to prove now: **the shape of the workspace** and
**the pipelines that will be trusted for every later stage**. It deliberately contains no behaviour. A skeleton that
touches no user file is the only change in this project that can be reviewed without risking anybody's data, so it is
also the right place to discover that CI, the toolchain pin or the crate layout is wrong.

## 2. Build exactly this

| Item | Requirement |
| --- | --- |
| Workspace | `Cargo.toml` at the repository root, `resolver = "2"`, two members and no more |
| Library crate | `crates/auto-tzim-core`, package name **`auto-tzim-core`**. Empty of behaviour: a `lib.rs` with the version constant and nothing that reads a file |
| Binary crate | `crates/auto-tzim`, package name **`auto-tzim`** (D-36 — the name is published one day, so it is fixed now), `[[bin]] name = "auto-tzim"` |
| Toolchain | `rust-toolchain.toml` pinning an exact stable version; the same version in CI and in the README |
| Lockfile | `Cargo.lock` committed. A library workspace still commits it: reviewers need the exact tree `cargo deny` looked at |
| Deny config | `deny.toml` with the D-35 refusals encoded: no crate that compiles C, no HTTP/network crate, licences limited to what the project accepts |
| CI | One workflow, `ubuntu-latest` + `macos-latest`, the pinned toolchain, jobs for build, test, `cargo deny`, and a **pinned-dependency check** |
| CLI surface | `--help` (lists only what exists: `--help`, `--version`) and `--version` (one line). No subcommands yet — they arrive with their issues |
| Exit codes | `0` for help and version; `2` for an unknown command or flag, with a message on stderr and nothing on stdout |

**Not in this task, at all:** config, discovery, planner, adapter, runner, state, `--json`, `--mode`, any `.rs` file that
opens a file, any crate beyond the two, any workflow beyond the one, the `tui` and `qt` crates (D-34 — they are v0.2 and
v0.4 and creating their directories now is scope).

## 3. Six acceptance points

Each needs its evidence in the handoff. A point without evidence is `not-run`, and `not-run` is not `passed`.

| # | Point | Evidence required |
| --- | --- | --- |
| 1 | A clean checkout builds and the binary is named `auto-tzim` | `cargo install --path crates/auto-tzim` in a fresh clone, then `auto-tzim --version` |
| 2 | `--help` and `--version` exit `0` on both platforms | CI run IDs at the final head SHA, one per platform, status and SHA filled in |
| 3 | An unknown command exits `2` | the invocation, its stdout, its stderr and its exit status, side by side |
| 4 | Both crates are **publishable** | `cargo package` per crate succeeds. This is a metadata property: **do not publish**, and do not claim a crates.io release |
| 5 | **An unpinned dependency fails CI** | the negative control, described in §4 |
| 6 | Nothing beyond help/version exists | the `--help` output, reviewed: no command is advertised that is not implemented |

## 4. The negative control is the deliverable

Point 5 is the one that decides whether the rest of this project's CI means anything. A job that cannot fail is
decoration, and the failure mode is silent: it stays green forever and nobody notices until a real regression ships.

Do it in this order, and put all three steps in the pull request:

1. add a dependency with an open range (`serde = "*"` or similar) to the core crate;
2. push, and **link the red run** — its run ID, the job name, the failing command and its message;
3. revert in a separate commit, and link the green run that follows.

The same rule applies to the deny job: if `deny.toml` forbids C-compiling crates, prove it once by adding `rusqlite`
in a throwaway commit and showing the job reject it. Record the run ID. This is the only way a future reader knows the
rule was ever live.

## 5. Two traps that are already in the documents

- **The crate name is not the old name.** If any file you touch says `auto-tzim-cli`, it is stale; the rename is D-36 and
  it cannot be revisited after publication. Same for `auto-tzim_tui`, which does not exist yet.
- **Do not "fix" the gate documents.** `GATES.md`, the corpus and the decision log are pre-registered: a coder that
  changes an acceptance row has changed the test it is measured by. If a row seems wrong or unsatisfiable, that is a
  finding to report in the pull request, not an edit.

## 6. What the pull request must contain

The [handoff block](HANDOFF-TEMPLATE.md) with every line filled, `remaining blocker: none`, and
`Merge/Release performed by coder: no`. Commits labelled `AT-01: …`. One pull request. The reviewer runs the acceptance
points independently and does not take the summary as evidence.

## 7. Why the start waited, in one paragraph

The MVP was blocked for three days, and the reason was never caution for its own sake: the `tzim` contract moved twice
under the documentation — `--capabilities` and a stable `reason_code` appeared, the accepted version window shifted, and
the default format set changed — and building against the old contract would have produced code that passed our tests
and failed against every binary a user can install. That is worse than a delay, because it looks finished. The
[readiness pass](READINESS-2026-11-02.md) is the last of that work: twelve contradictions found by reading the assignment
against the decisions, all fixed. Nothing in the repository blocks AT-01 now.
