# Review — ebfae55

Reviewer: manager (this session) · Date: 2026-11-02
Scope: **AT-01 only** (#1), round two. Specification: `SPEC.md` 0.1 (`250d0db1…f660c`) · `MANAGER-SPEC.md` 0.2 (`7e798361…43f1`)
Previous review: [REVIEW-514b06f](REVIEW-514b06f.md) — ten findings, verdict NOT READY.

**Verdict: NOT READY — four findings open (F-11, F-12, F-03, F-06) and three carried over (F-07, F-08, F-09).**
Four of the ten findings from round one are fixed and verified. The branch is red at its head, and the coder is right that
calling it green would be false — the question is what the *accepted* evidence is, and that is now written down rather
than improvised: [D-39](../DECISIONS.md).

## 1. Binding

| Check | Result | Evidence |
| --- | --- | --- |
| claimed head SHA == PR head | **pass** — `ebfae55110d23aff3f2f01d86481eca72bc629d0` | `gh pr view 21 --json headRefOid` |
| commits labelled `AT-01` | **pass** — three new commits, each labelled | `git log --oneline 514b06f..ebfae55` |
| branch contains the current `main` | **no — three commits behind** (`354166c` is not an ancestor) | `git merge-base --is-ancestor` |
| tree contains only what the handoff claims | **pass** — six files changed: workflow, `Cargo.lock`, two manifests, `main.rs`, `tests/cli.rs` | `git diff --name-only 514b06f..ebfae55` |
| gate documents untouched | **pass** — `DECISIONS.md` not edited by the coder, as instructed | same |
| coder performed no merge or release | **pass** | `gh pr view 21` — open; no tag |

Being behind `main` is not a finding: the three commits that landed (`REVIEW-514b06f`, `D-38`, `D-39`…`D-41`) are
documentation, and a rebase would only add noise to the review. It matters for one thing — the coder has not yet read
[D-39](../DECISIONS.md), which answers the question in the handoff.

## 2. CI

Run **41441125967** at `ebfae55`, event `push`:

| Job | ubuntu-latest | macos-latest |
| --- | --- | --- |
| build | success | success |
| **test** | **failure** | **failure** |
| deny | success | success |
| pinned-dependencies | success | success |

The failure is one step and one cause, read from the job annotation rather than from the (unreachable) raw log:

```text
Packaging auto-tzim v0.1.0
    Updating crates.io index
error: failed to prepare local package for uploading
Caused by: no matching package named `auto-tzim-core` found
  location searched: registry `crates-io`
  required by package `auto-tzim v0.1.0`
```

Everything **before** that step passed on both platforms, which is how the fixes below are verified: `cargo fmt --all
--check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, the CLI contract and
`cargo install --path` all precede the failing step in the same job. Sixteen check runs at the head, all from two events
pushing the same SHA — F-08, still open.

## 3. Round-one findings

| ID | State | How it was verified |
| --- | --- | --- |
| F-01 fmt + clippy are gates | **fixed** | both are steps in the `test` job (`ci.yml:28-29`); `rust-toolchain.toml` now carries `components = ["rustfmt", "clippy"]`; the job ran past them on both platforms |
| F-02 zero tests | **fixed** | `crates/auto-tzim/tests/cli.rs` — one integration test using `env!("CARGO_BIN_EXE_auto-tzim")`, asserting the version string exactly, `2` for an unknown argument, empty stdout, non-empty stderr. `cargo test` ran before the failing step, so it passed |
| F-04 unpinned actions | **fixed** | every `uses:` now carries a 40-hex SHA with a version comment. `dtolnay/rust-toolchain@683b588…` is a frozen commit of `master`, which is what "pinned" means; I accept it |
| F-05 crates unrelated | **fixed** | `auto-tzim-core = { path = "../auto-tzim-core", version = "=0.1.0" }`; `#[command(version = auto_tzim_core::VERSION)]` — the binary now takes its version from the core, which is the first real expression of D-34's "the core owns" in the dependency graph |
| F-03 MSRV | **open, now answered** | [D-40](../DECISIONS.md): declare `rust-version = "1.82"` in both manifests, equal to the pinned channel. My earlier recommendation of 1.88 is withdrawn, with the reason in the decision |
| F-06 Python pin script | **open, now answered** | [D-41](../DECISIONS.md): CI-only tooling may be Python, with three binding conditions — a self-test in CI, a mention in `dependencies.md` §2a, and a roadmap item to fold it into the Rust harness at AT-08 |
| F-07 debug notices | **open, one part fixed** | the `deny` `::error` stays, correctly. The two CLI `::notice`s are now redundant *because* F-02 landed — a test asserts what a notice only announces |
| F-08 double matrix | **open** | `on: [push, pull_request]`, 16 check runs for 8 jobs |
| F-09 moving runner labels | **open** | `ubuntu-latest`/`macos-latest` unchanged since round one |
| F-10 `disable_help_subcommand` | **accepted as is** | harmless; noted for AT-06 |

## 4. Findings

| ID | Severity | Issue | What is wrong | Required change | State |
| --- | --- | --- | --- | --- | --- |
| F-11 | High | AT-01 (#1) | **The head is red, so the acceptance evidence does not exist yet.** The cause is not a defect — it is the dependency from F-05 meeting an unpublished crate — but a pull request whose required job fails cannot be accepted, and the fix is not "try harder": it is to record what is verified and what is deferred | adopt the four checks of [D-39](../DECISIONS.md) exactly: `cargo package -p auto-tzim-core` (full), `cargo package --no-verify -p auto-tzim` **plus** an assertion on the normalised manifest inside the `.crate`, and a guard step that runs `cargo package -p auto-tzim` and asserts it fails with `no matching package named`. Green at the new head | open |
| F-12 | High | AT-01 (#1) | **One step mixes a hard gate with a known-red one.** `cargo install --path` (a real acceptance point) sits in the same shell block as `cargo package -p auto-tzim` (expected red until release). Today the block fails for the second reason, and tomorrow an install regression would be indistinguishable from the expected failure — which is precisely how a real defect gets waved through as "the packaging thing again" | split them: `install` stays a hard step that must be green; the package check becomes the D-39 guard with its asserted message. A failed install must never be able to blame packaging | open |
| F-03 | Medium | AT-01 (#1) | MSRV undeclared in both manifests | add `rust-version = "1.82"` to `crates/auto-tzim-core/Cargo.toml` and `crates/auto-tzim/Cargo.toml` ([D-40](../DECISIONS.md)) | open |
| F-06 | Low | AT-01 (#1) | the pin checker has no control of its own | add a CI step that runs the script against two fixture manifests and asserts exit `1` on `"*"` and exit `0` on `"=1.2.3"` ([D-41](../DECISIONS.md)); name the script in `dependencies.md` §2a — already done by the manager | open |
| F-13 | Low | AT-01 (#1) | **the test does not assert `--help`.** Acceptance point 2 covers `--help` and `--version`; the test covers the second and the unknown-argument path only, so `--help` is asserted only by the shell step — which F-07 will delete | add `--help` to `tests/cli.rs`: exit `0`, stdout non-empty, stderr empty | open |
| F-14 | Low | AT-01 (#1) | branch is three commits behind `main` | rebase before the final handoff, so the re-review runs against the decisions that now bind it | open |

## 5. Answers the coder asked for

The handoff asks one question — *how to verify packaging before publication without publishing* — and it is the right
question, asked before doing something irreversible. The answer is [D-39](../DECISIONS.md), summarised:

1. **`cargo package -p auto-tzim-core`** — full verification. It works today because the core is a leaf; keep it as a
   hard green step.
2. **`cargo package --no-verify -p auto-tzim`** — produces `target/package/auto-tzim-0.1.0.crate` without building it.
   Works today. Then assert the manifest the user would actually download:

   ```sh
   tar -xzOf target/package/auto-tzim-0.1.0.crate auto-tzim-0.1.0/Cargo.toml > /tmp/packaged-Cargo.toml
   grep -q 'auto-tzim-core' /tmp/packaged-Cargo.toml
   ! grep -q 'path *= *"\.\./auto-tzim-core"' /tmp/packaged-Cargo.toml
   ```

   The second half is the point: it proves cargo stripped the `path` and kept the version — the manifest is
   registry-correct.
3. **A guard that asserts the expected failure.** Run `cargo package -p auto-tzim` and require it to fail *with
   `no matching package named`*. This is the row that keeps the other two honest: when the core is eventually published
   the guard goes green and someone must look, instead of a permanently red step being ignored for months.
4. **At release:** core first, then `cargo package -p auto-tzim` for real. Recorded in [releases](../../releases.md) §2.

What I will **not** accept: declaring `Remaining blocker: none` while a required job is red (the coder did not do this —
correct), or leaving the red step unasserted so that "packaging again" becomes the standard excuse.

## 6. Verdict

**NOT READY.** F-11 and F-12 are the work; F-03 and F-06 are two lines each, and the answers are now decisions rather
than questions to me. F-07, F-08, F-09, F-13, F-14 ride along. Re-review at the new head SHA.

Signed: manager (this session) · Date: 2026-11-02
Merge performed by reviewer: **no**
