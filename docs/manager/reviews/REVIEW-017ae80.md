# Review — 017ae80

Reviewer: manager (this session) · Date: 2026-11-02
Scope: **AT-01 (#1)**. Previous: [REVIEW-514b06f](REVIEW-514b06f.md), [REVIEW-ebfae55](REVIEW-ebfae55.md),
[REVIEW-38332aa](REVIEW-38332aa.md) — all NOT READY.

**Verdict: PASS — AT-01 is complete at `017ae80`.**

Reviewed **after** the merge, and that is stated rather than glossed: the owner merged #21 as `fc17b9b` at
2026-11-02T12:19:03Z, **35 seconds after** the run finished green (12:18:32Z). The order was correct — green, then merge —
but this report is a post-merge review, so it cannot gate the merge it describes. Its value is the record, and the fact
that AT-01's evidence is now fixed at a SHA somebody can re-check.

## 1. Binding

| Check | Result | Evidence |
| --- | --- | --- |
| claimed head SHA == merged head | **pass** — `017ae80a0fa69c7dbd52ae27afc1e85203ece2f6` | `gh pr view 21` |
| merged only after green | **pass** — run updated `12:18:32Z`, merged `12:19:03Z` | run metadata vs `mergedAt` |
| the change since the last review is the deletion I asked for | **pass** — one file, 25 lines, all removals | `git diff 38332aa..017ae80` |
| `Install CLI` untouched | **pass** | the diff has no other hunk |
| gate documents untouched by the coder | **pass** | `git diff --name-only 2d98275..017ae80` |
| nothing published | **pass** — no tag, no release | `gh api .../releases` → 0 |

## 2. CI

Run **41442840838** at `017ae80`: **8 of 8 jobs green** on `ubuntu-24.04` and `macos-15` — build, test, deny,
pinned-dependencies. The head SHA produces **8 check runs, not 16**: F-08 is fixed and verified by the number itself.

Post-merge, `main` at `fc17b9b` also has 8 check runs, all success.

The macOS jobs were still queued when the coder reported; the run finished green twenty seconds later. Reporting a
half-queued run as done would have been a claim without evidence — the coder did not, and said exactly which jobs were
still running. That is the behaviour the process is supposed to produce.

## 3. The six acceptance points

| # | Point | Verdict | Evidence |
| --- | --- | --- | --- |
| 1 | clean checkout builds; binary named `auto-tzim` | **pass** | `Install CLI` step: `cargo install --locked --path crates/auto-tzim` then the installed `auto-tzim --version`, green on both platforms |
| 2 | `--help` and `--version` exit `0` on both platforms | **pass** | asserted in `tests/cli.rs` (exit status, non-empty help, exact `auto-tzim <version>\n`, empty stderr) and by the shell contract step; both ran green |
| 3 | unknown command exits `2`, empty stdout | **pass** | `tests/cli.rs` asserts `Some(2)`, empty stdout, non-empty stderr; the shell step asserts the same for `unknown` and `--unknown` |
| 4 | registry metadata present (D-42, not `cargo package`) | **pass** | both manifests carry `name`, `version`, `edition`, `rust-version`, `license`, `description`, `repository` — read, not assumed |
| 5 | an unpinned dependency fails CI | **pass** | red `41439451071` at `7149d7c`; the checker now also has a **self-test**, whose control I ran myself (§4) |
| 6 | nothing beyond help/version | **pass** | `--help` lists `-h/--help`, `-V/--version`; `main.rs` is nine lines; `lib.rs` is one constant; no file, socket or process API anywhere in `src/` (`git grep` for `std::net`, `TcpStream`, `std::process`, `std::fs`, `File::`, `OpenOptions` → no hits) |

No `cargo package` step remains in the workflow, and `main` confirms it: `grep -n "Package\|no-verify\|Guard"` over the
merged `ci.yml` returns nothing.

## 4. Controls I ran myself

No Rust toolchain exists in this environment, so nothing Rust was compiled or executed here — that is stated plainly
because it bounds every mark above. What could be executed, was:

| Control | Result |
| --- | --- |
| `python3 scripts/test-check-pins.py` on a clean export of `main` | `Pin checker self-test passed`, exit `0` |
| `python3 scripts/check-pins.py` on the same tree | `All declared dependencies are exactly pinned`, exit `0` |
| **break the checker, re-run the self-test** | checker accepted `"*"` (wrongly, exit `0`) and the self-test **failed**: `'*': expected 1, got 0` — so the self-test is not decoration; it observes the real defect |
| restore and re-run | green again |

The break I first attempted (`False and … or …`) silently did nothing because of Python operator precedence, and the
self-test correctly stayed green — a nice accident that shows the test is checking behaviour rather than keywords. The
second attempt replaced the condition outright and the test went red.

## 5. Findings

| ID | Severity | State |
| --- | --- | --- |
| F-01…F-14 | — | **closed** — all fixed and verified across the three rounds; F-11 withdrawn with [D-42](../DECISIONS.md) |
| F-15 | Low, follow-up for AT-02…AT-06 | `disable_help_subcommand = true` stays until subcommands exist; re-check when they do |
| F-16 | Low, follow-up for AT-08 | the pin checker is Python by [D-41](../DECISIONS.md), with a self-test; fold it into the Rust harness when the harness exists |
| F-18 | Low, fixed in AT-02's scope | [D-42](../DECISIONS.md) promised that the manifests' registry metadata would be **asserted** — "by the pin checker's companion script, or by `cargo metadata --no-deps` in one line" — and no such step exists in `ci.yml`. Acceptance point 4 passed because I read the four manifests, so nothing prevents the next commit from deleting `repository`. The brief for AT-02 makes the two-line check part of that stage |
| F-17 | Low, housekeeping | `actions/checkout@v4` still emits a Node.js 20 deprecation warning in every run; harmless now, will break eventually |

None of the three blocks acceptance: they are named so the next stage does not rediscover them.

## 6. What PASS means here, and what it does not

PASS means: **AT-01's scope, at `017ae80`, has no unresolved Critical/High finding, and every acceptance point was
reproduced by the reviewer** — from CI status, from job annotations, and from source, with the one Rust-free control run
locally.

It does **not** mean:

- that anything about the product works — there is no product yet, only `--help` and `--version`;
- that AT-02 may start automatically. Each stage starts on the owner's word; this report recommends, it does not schedule;
- that the branch-protection obligations are met. When AT-01's CI is the CI, required status checks come back **with the
  real context names** ([D-24](../DECISIONS.md)) — see §7.

## 7. One thing the merge changes back

`main` currently has **no required status checks** ([D-24](../DECISIONS.md), repaired on 2026-11-02 by deleting the
placeholder `test` context that nothing could report). Now that a real CI exists, that placeholder can be restored
honestly — with the **actual** context names this workflow produces:

```text
build (ubuntu-24.04) · build (macos-15) · test (ubuntu-24.04) · test (macos-15)
deny (ubuntu-24.04) · deny (macos-15) · pinned-dependencies (ubuntu-24.04) · pinned-dependencies (macos-15)
```

That is an owner action (`gh api -X PATCH .../branches/main/protection/required_status_checks`), and it is the last piece
of D-24 that the codebase can now support. Doing it before AT-02 means the second stage is the first one merged through a
real gate — which is worth more than doing it before the tenth.

Signed: manager (this session) · Date: 2026-11-02
Merge performed by reviewer: **no** (the owner merged #21; this review is post-merge and says so)
