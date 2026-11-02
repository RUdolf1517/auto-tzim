# Review — 38332aa

Reviewer: manager (this session) · Date: 2026-11-02
Scope: **AT-01 only** (#1), round three.
Previous: [REVIEW-514b06f](REVIEW-514b06f.md), [REVIEW-ebfae55](REVIEW-ebfae55.md) — both NOT READY.

**Verdict: NOT READY — one deletion, then PASS.** Everything the coder was asked to fix is fixed, and the one thing that
is not fixed cannot be fixed: the packaging checks I ordered are physically impossible before publication. That is my
error, not the coder's, and it is now withdrawn by [D-42](../DECISIONS.md).

## 1. Binding

| Check | Result | Evidence |
| --- | --- | --- |
| claimed head SHA == PR head | **pass** — `38332aadd171d103011976f97da98fb685c64e58` | `gh pr view 21 --json headRefOid` |
| branch contains `main` | **pass** — `2d98275` is an ancestor; the coder merged rather than rebased, as instructed, so review history survives | `git log --oneline` |
| commits labelled `AT-01` | **pass** | three commits |
| gate documents untouched | **pass** — no change under `docs/manager/` or to the spec copies | `git diff --name-only 2d98275..38332aa` |
| nothing published | **pass** — no tag, no registry credential, no release | `gh api .../releases` |

## 2. CI

Run **41442119906** at `38332aa`: `build`, `deny`, `pinned-dependencies` **green on both platforms**; `test` **red on both**,
at the `Package CLI without verification` step.

The coder's claim is exact and I reproduce the reading from the annotation: on Cargo 1.82, `cargo package --no-verify -p
auto-tzim` fails with the same `no matching package named auto-tzim-core found`. `--no-verify` skips the build, not the
preparation step that resolves dependencies against the registry. **D-39's middle row was therefore impossible**, and
with it the whole scheme.

## 3. Findings from round two

| ID | State | Verified |
| --- | --- | --- |
| F-03 MSRV | **fixed** | `rust-version = "1.82"` in both manifests |
| F-06 pin-checker self-test | **fixed** | `scripts/test-check-pins.py` builds fixture manifests in a temp dir and asserts `*` → exit 1, `=1.2.3` → exit 0; it runs as a step of `pinned-dependencies`. It is a better test than I asked for: it exercises the real checker rather than a copy |
| F-07 debug notices | **fixed** | the CLI `::notice` calls are gone; the `deny` `::error` stays, correctly |
| F-08 double matrix | **fixed** | `on.push.branches: [main]` + `pull_request` |
| F-09 moving runners | **fixed** | `ubuntu-24.04`, `macos-15` |
| F-12 install mixed with packaging | **fixed** | `Install CLI` is now its own hard step and it passed on both platforms before the failing one |
| F-13 `--help` untested | **fixed** | `tests/cli.rs` asserts exit `0`, non-empty stdout, empty stderr |
| F-14 branch behind `main` | **fixed** | merged, not rebased |
| F-11 packaging evidence | **withdrawn** | see §4 |

## 4. The finding I created, and withdraw

I told the coder to prove "both crates are packagable" with `cargo package`. That command does not only build a tarball;
it resolves the crate's dependencies **against the registry**, so in a workspace whose CLI depends on an unpublished
sibling it fails by construction. My workaround (`--no-verify`) does not work on 1.82 either, and my "guard that asserts
the expected failure" was a step whose only job was to be red — which is exactly the ritual I have been criticising all
week.

Three review rounds, two red CI runs and one useless step later, the requirement is withdrawn rather than negotiated:
[D-42](../DECISIONS.md).

- **Nothing in this project publishes.** Publication is a release action (D-36), the owner's, and AT-01 needs none of it.
- **"Publishable" means the metadata is there.** That is a manifest property, assertable in one line, and it is what
  ROADMAP-AT-01 always meant before I reinterpreted it.
- **Packaging is verified at release**, in the order core → CLI, from `docs/releases.md` §2, by the owner, once.

## 5. What is left

One change, and it is a deletion:

1. remove `Package core`, `Package CLI without verification` and `Guard expected unpublished core failure` from
   `.github/workflows/ci.yml`;
2. keep `Install CLI` as a hard step (already correct);
3. if the coder wants the metadata check to be mechanical, one line in `pinned-dependencies` is enough —
   `cargo metadata --no-deps --format-version 1` and assert the seven fields. Optional: I will accept it either way,
   because the manifests are visible in review.

Then the head is green and AT-01 is `PASS`.

**Why this is not scope creep in reverse.** The coder asked twice rather than improvising, and both times the answer
involved me being wrong. That is the process working: the failure was loud, cheap, and recorded instead of shipped as a
permanently red step with a comment explaining why nobody should worry about it.

Signed: manager (this session) · Date: 2026-11-02
Merge performed by reviewer: **no**
