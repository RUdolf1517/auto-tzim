# Review — 514b06f

Reviewer: manager (this session) · Date: 2026-11-02
Scope: **AT-01 only** (#1). AT-02…AT-09 are not started and are not reviewed here.
Specification: `SPEC.md` 0.1 (sha256 `250d0db1…f660c`) · `MANAGER-SPEC.md` 0.2 (sha256 `7e798361…43f1`)
Gates: [GATES.md](../GATES.md) — at this stage only the exit-code rows that AT-01 touches (AC-8), nothing else.
Corpus: [FIXTURE-CORPUS.md](../FIXTURE-CORPUS.md) — not exercised; AT-08 owns it.

**Verdict: NOT READY — findings F-01…F-05.** The six acceptance points of AT-01 all pass. The findings are not about
those points being false; they are about the skeleton carrying five things that cost ten minutes now and a migration
later.

## 1. Binding

| Check | Result | Evidence |
| --- | --- | --- |
| claimed head SHA == PR head | **pass** — `514b06fbb26ed285e8482128b2cd8fc77bb9ed31` | `gh pr view 21 --json headRefOid` |
| SHA reachable, branch based on current `main` | **pass** — merge-base is `origin/main` (`2cd2225`), 10 commits, linear, no merge commits | `git log --oneline origin/main..514b06f` |
| commits labelled `AT-01` | **pass** — all ten | same |
| tree contains what the PR claims and nothing else | **pass** — 12 files: 4 manifests/config, 3 sources, workflow, script, lockfile, `.gitignore`, `README` | `gh pr diff 21 --name-only` |
| gate documents untouched | **pass** — no change under `docs/manager/` | same |
| coder performed no merge or release | **pass** — PR open, no tag, no crates.io claim | `gh pr view`, `gh api .../releases` |

## 2. CI

Run **41439760612**, `CI`, head `514b06f`, event `push`, **all 8 jobs success** (independently verified by API):

| Job | ubuntu-latest | macos-latest |
| --- | --- | --- |
| build | success | success |
| test (incl. package, install, CLI contract) | success | success |
| deny | success | success |
| pinned-dependencies | success | success |

Skipped / cancelled / neutral jobs: **none** at the head SHA.

Two environment warnings appeared in the run and are recorded as F-09: `Node.js 20 is deprecated` for
`actions/checkout@v4`, and *"`ubuntu-latest` will migrate to Ubuntu 26 beginning October 19, 2026"*.

## 3. Row reconciliation

Not applicable at this stage: no corpus, no artefact bundle, no per-case rows (AT-08). Nothing is claimed about them and
nothing here should be read as covering them.

## 4. Acceptance matrix — the six AT-01 points

| # | Point | Verdict | Mark | Evidence |
| --- | --- | --- | --- | --- |
| 1 | clean checkout builds; binary is named `auto-tzim` | **pass** | independently verified | `ci.yml:36` `cargo install --locked --path crates/auto-tzim --root …` then `"$RUNNER_TEMP/auto-tzim-install/bin/auto-tzim" --version`; the job is green at the head SHA, so the installed binary existed under that name on both platforms |
| 2 | `--help` and `--version` exit `0` on both platforms | **pass** | independently verified | check-run annotation `CLI output` carries the full help text and `auto-tzim 0.1.0` from the run at `514b06f`; job green on both |
| 3 | unknown command exits `2` | **pass** | independently verified + source-inspected | annotations `Unknown argument unknown` and `Unknown argument --unknown` both read `exit=2 stdout=<empty>`; `main.rs:9` `Cli::parse()` and no positional arguments ⇒ clap's usage error path, exit `2` |
| 4 | both crates are publishable | **pass, with a caveat** | independently verified | `cargo package --locked` for both crates is a step in the green job. Caveat is F-05: the binary crate has no dependency on the core crate, so packaging is trivially easy today |
| 5 | an unpinned dependency fails CI | **pass** | independently verified + tool-executed | red run **41439451071** at `7149d7c`: `pinned-dependencies (ubuntu-latest)` failure. The message the coder could not read, I reproduced by running the script myself (see §5) |
| 6 | nothing beyond help/version exists | **pass** | source-inspected | `--help` lists `-h/--help` and `-V/--version` only; `main.rs` is nine lines; `lib.rs` is one constant; no file is opened anywhere in the tree |

The negative controls are real, and better than the handoff claimed: the deny red run failed with **both** mechanisms —
`error[banned]: crate 'rusqlite = 0.31.0' is explicitly banned` **and** `error[not-allowed]` for `libsqlite3-sys`,
`once_cell`, `pkg-config` — which shows the allowlist is deny-by-default, not a courtesy list.

## 5. Controls I ran myself

The coder's remaining blocker was "cannot read CI logs" (the run logs are served from
`productionresultssa5.blob.core.windows.net`, which this environment cannot reach — I confirmed the same failure, so the
blocker was real and not an excuse). Two independent routes closed it:

| Control | Command | Result |
| --- | --- | --- |
| read CI output without logs | `gh api repos/…/check-runs/<id>/annotations` | **works** — the CLI contract evidence and the deny error text are in annotations; the coder had already been reading these, so the blocker applied only to raw logs |
| pin checker, as shipped | `python3 scripts/check-pins.py` on the branch manifests | exit `0`, `All declared dependencies are exactly pinned` |
| pin checker, `serde = "*"` | same, with the dependency added | exit `1`, `crates/auto-tzim-core/Cargo.toml: serde: dependency version must be exact (=X.Y.Z), got '*'` — the exact text the handoff could not confirm |
| pin checker, unprefixed `"4.5.20"` | same | exit `1` |
| pin checker, `"=1.2.3"` | same | exit `0` |
| pin checker, `workspace = true` | same | exit `0` (skipped, as designed) |
| pin checker, nested `[target.'cfg(unix)'.dependencies.bar]` | same | exit `1` — recursion into nested tables works |
| zero tests in the tree | `git grep -n '#\[test\]\|#\[cfg(test)\]' 514b06f -- '*.rs'` | **no hits** — see F-02 |

**Caveat on my own evidence:** there is no Rust toolchain in this environment (`cargo: command not found`), so nothing
was compiled or executed here. Everything about the binary's behaviour is read from CI at the head SHA, not reproduced
locally. That is why the annotations matter and why the marks say "independently verified" rather than "reproduced".

## 6. Findings

| ID | Severity | Issue | What is wrong | Evidence | Required change | State |
| --- | --- | --- | --- | --- | --- | --- |
| F-01 | Medium | AT-01 (#1) | **Two documented CI gates do not exist.** `docs/dependencies.md:52` says "`cargo fmt --check`, `clippy -D warnings` and `cargo deny check` are CI gates"; `CONTRIBUTING.md:74` tells contributors to run both. The workflow runs `cargo deny` and neither of the other two, and `rust-toolchain.toml` uses `profile = "minimal"`, so those components are not even installed | `ci.yml:5-53` — no `fmt`, no `clippy` job; `rust-toolchain.toml:3` | add `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` as steps or a job; add the components to the toolchain profile so they exist | open |
| F-02 | Medium | AT-01 (#1) | **`cargo test` runs zero tests.** There is no `#[test]` anywhere in the tree, so the step prints `running 0 tests` and succeeds by doing nothing. The CLI contract is asserted by shell in the same job, which is fine as evidence but is not a test anyone can run locally with `cargo test` | `git grep '#[test]'` → 0 hits; `ci.yml:28` | add one integration test (`crates/auto-tzim/tests/cli.rs`) using `env!("CARGO_BIN_EXE_auto-tzim")`: `--version` prints `auto-tzim <CARGO_PKG_VERSION>`, an unknown argument exits `2` with empty stdout. Keep the shell evidence if you like; the test is what makes the job meaningful | open |
| F-03 | Medium | AT-01 (#1) | **The MSRV is not declared where `cargo install` can see it.** Neither manifest has `rust-version`, while `docs/releases.md:34` states "The crate declares a minimum supported Rust version" and D-36 promises the pin is stated. `rust-toolchain.toml` is not shipped in the published crate, so a user with an older compiler gets an opaque dependency error instead of a clean refusal. Separately, **1.82.0 is older than `tzim`'s own `rust-version = "1.88"`**, and nothing in the repository explains the number | `crates/*/Cargo.toml` — no `rust-version`; `rust-toolchain.toml:2`; `docs/releases.md:34`; `tzim` `Cargo.toml:5` | add `rust-version = "…"` to both manifests. The number itself is the owner's call — I recommend **1.88** to match `tzim`, and the same channel in `rust-toolchain.toml`, CI and README. If 1.82 is deliberate, say why in DECISIONS | open |
| F-04 | High | AT-01 (#1) | **The CI executes code from a moving branch.** `dtolnay/rust-toolchain@master` runs whatever `master` points at when the job starts, with our `GITHUB_TOKEN` in the runner; `taiki-e/install-action@cargo-deny` is a moving tag. In a project that refuses a C library it cannot audit (D-35), an unpinned third-party action is the same objection with worse consequences | `ci.yml:13, 25, 60` (`@master`), `ci.yml:63` (`@cargo-deny`) | pin every action to a full commit SHA with a trailing comment naming the version; `dtolnay/rust-toolchain` supports version refs (`@1.82.0` or `@stable`) and a SHA is better still. `actions/checkout@v4` may stay on the major tag if you prefer, but say so | open |
| F-05 | Medium | AT-01 (#1) | **The two crates are unrelated.** The binary does not depend on `auto-tzim-core`; it takes its version from clap and the core's `VERSION` constant is unused. So D-34's "the core owns all behaviour" is not yet expressed in the dependency graph, and "publishable" is trivially true — the moment the real dependency is added, `cargo package -p auto-tzim` will fail until the core is published, which is a decision nobody has taken | `crates/auto-tzim/Cargo.toml:13-14` (no core dependency); `main.rs:5` (`#[command(version)]`); `lib.rs:2` (`VERSION` unused) | add `auto-tzim-core = { path = "../auto-tzim-core", version = "0.1.0" }` and take the version from it (either `#[command(version = auto_tzim_core::VERSION)]` or print it). Record the packaging order (core first, then binary) in `docs/releases.md` | open |
| F-06 | Low | AT-01 (#1) | **The pin check is Python**, so CI needs `python3 ≥ 3.11` (`tomllib`) on both runners, and `docs/testing-harness.md:26` says this project's tooling is a Rust binary, not a Python script. The script also has no test of its own — the only thing that ever exercised it was the one-shot red run | `scripts/check-pins.py:1-6`; `ci.yml:80` | keep it, replace it, or move it to a Rust `xtask` — but decide in writing. I verified the script's behaviour (§5); if it stays, that verification belongs in the repo as a test. This is a manager/owner call, not a defect the coder must guess about | open |
| F-07 | Low | AT-01 (#1) | **Debug scaffolding is now permanent CI.** The `::notice`/`::error` emitters with hand-rolled `%`/newline escaping exist because raw logs were unreadable from the coder's environment. The `deny` one earns its place; echoing the entire `--help` text into a notice does not | `ci.yml:44, 51, 71` | keep the `deny` `::error` (it is what made the control verifiable); drop the two CLI notices once F-02 lands, because a test asserts what a notice only announces | open |
| F-08 | Low | AT-01 (#1) | **Every branch push runs the whole matrix twice** — `on: [push, pull_request]`, and both events fire for a branch in a PR. The head SHA produced **16 check runs for 8 jobs** | `ci.yml:2`; `gh api …/commits/<sha>/check-runs` → `total_count: 16` | `push: branches: [main]` plus `pull_request` | open |
| F-09 | Low | AT-01 (#1) | **Moving runner labels.** `ubuntu-latest` and `macos-latest` change under us — the run itself warns that `ubuntu-latest` becomes Ubuntu 26 on 2026-10-19. For a project that pins everything else, the runner image is the one unpinned input left | run annotations; `ci.yml:9, 21, 56, 76` | pin `ubuntu-24.04` and a specific `macos-…` image, and note both in the README's build section | open |
| F-10 | Low | AT-01 (#1) | `disable_help_subcommand = true` is a no-op while there are no subcommands, and will need revisiting when they arrive | `main.rs:5` | drop it, or keep it and note in AT-06 that subcommands must be re-checked against it | open |

## 7. Claims re-marked

| Claim | Author's mark | Reviewer's mark | Note |
| --- | --- | --- | --- |
| head SHA, branch, ten labelled commits | stated | **independently verified** | API |
| 8 CI jobs green at the head SHA, both platforms | stated | **independently verified** | run 41439760612 |
| `cargo install --locked --path` then `--version` = `0` | "0 in fresh CI checkout" | **independently verified** | the exact command sequence is in the green job; nothing was compiled locally (no toolchain here) |
| `--help`/`--version` = `0`, text shown | stated | **independently verified** | check-run annotation at the head SHA |
| unknown argument = `2`, empty stdout, stderr non-empty | stated | **independently verified** | annotations for both `unknown` and `--unknown` |
| `cargo package` both crates = `0` | stated | **independently verified, with F-05** | green step; the crates are independent |
| pin control red 41439451071, exact message unverified | "text not verified" | **tool-executed by me** | message reproduced locally: `serde: dependency version must be exact (=X.Y.Z), got '*'` |
| deny control red 41439637128, `error[banned] rusqlite` | stated | **independently verified** | annotation contains the `banned` and `not-allowed` errors |
| "Remaining blocker: cannot read CI logs" | blocker | **closed** | logs are blocked here too; annotations are the supported route, and the script's behaviour was reproduced offline |
| "Tests added: CI workflow — CLI contract; no negative control for the CLI test" | honest gap | **confirmed gap** | F-02: there is no test at all, so there is nothing to control |

## 8. Answers to the coder's three questions

**1. Is a fresh CI checkout enough for the install acceptance, or is a separately created clone required?**
Enough for AT-01, and I am not going to invent a ritual. `cargo install --locked --path` in a checkout runs the same
resolve-build-place sequence as `cargo install auto-tzim`, and it ran on both platforms at the head SHA. What it does
**not** prove: installation from the published crate (impossible before publication) and installation without a
toolchain on the machine (D-36 accepts that `cargo install` needs a compiler). One sentence saying exactly that belongs
in the AT-01 handoff, not a second checkout.

**2. Are clap's automatic `-h` and `-V` aliases acceptable?**
Yes. They are options of the one command that exists, not new commands, and disabling them would be a deviation from
every Rust CLI a user has met. Keep `--help` and `--version` as the documented spellings; `-h`/`-V` are ordinary
aliases of the same exit-`0` path.

**3. Is a deny-by-default allowlist enough for the "no C, no network" policy?**
Yes as the mechanism, with a condition and a record. The condition: `cargo-deny` cannot classify behaviour, so the
allowlist is only as good as the review that admits an entry — that review is the policy, and the red run proves the
allowlist actually refuses (it rejected `libsqlite3-sys`, `once_cell` and `pkg-config` on the way in). The record: this
is now [D-38](../DECISIONS.md) — exact pins, deny-by-default allowlist, an explicit deny list as the second net, and
"a new dependency is a deliberately two-step change". Two consequences the coder should carry forward: a **feature** flip
on an already-allowed crate is not caught by the allowlist, so dependency changes get `cargo tree -e features` in the PR;
and `docs/dependencies.md` must describe this mechanism instead of describing only `cargo deny`.

## 9. What would make this PASS

F-01…F-05, in a single push, with the same report format. F-06, F-03's number and F-04's `checkout@v4` exception need one
word from the owner; the rest are nits that can ride along. Re-review happens at the new head SHA — the rows that depend
on it are re-run, not argued.

Signed: manager (this session) · Date: 2026-11-02
Merge performed by reviewer: **no**
