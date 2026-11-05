# Review — d0ac17e (AT-02, PR #34)

Reviewer: manager (this session) · Date: 2026-11-05
Scope: **AT-02 (#2)** — the D-56 / F-22–F-25 work on top of [b1b6d5a](REVIEW-b1b6d5a.md).
Head: `d0ac17ef983c717fd451bbe6ba48db51bc3c481b`, 3 files changed since `b1b6d5a` (+202 −50), 11 files vs the PR base.

**Verdict: PASS — AT-02 is complete at `d0ac17e`.** Two Low findings, listed in §4, neither of which blocks acceptance and
both cheaper to fix in AT-03 than to hold the stage for. The merge is the owner's word, not mine: PR #34 is product code.

## 1. F-22 is fixed, and the fix is the split I asked for, not a deletion

| Kind | Where it lives now | Evidence |
| --- | --- | --- |
| structure — schema, enums, names, duplicate name/root, absolute and non-empty root, `forbidden_root`, `state_inside_root`, `cloud_sync_root`, `not_in_v01`, ranges, formats, `tzim_path` **shape** | `validate_document` + `validate_config`, still called by **both** `load()` and `save()` | `validate_config` line 603 calls `validate_root_structure`; `save()` line 781 calls `validate_config` |
| environment — root exists / is a directory / is not a symlink / canonicalises; `tzim_path` exists, is a regular file, is executable | `validate_environment()` → `validate_root()`, called from the `Config::Validate` arm and from `folder add` | `main.rs` lines 135 and 165 |

The behaviour that was the defect, for a profile whose disk is gone and a `tzim_path` that was uninstalled:

| Command | At `b1b6d5a` | At `d0ac17e` |
| --- | --- | --- |
| `config validate` | exit `2`, `root_missing` | exit `2`, `root_missing` **and** `tzim_path_missing` — unchanged, correct |
| `folder list` / `show` | exit `2` | exit `0` |
| `folder edit` / `enable` / `disable` / `remove` | exit `2` | exit `0` |
| `folder add` of a **new** valid folder while another is unavailable | exit `2` | exit `0` |
| `folder add` of a path that does not exist | exit `2`, `root_missing` | exit `2`, `root_missing` |

The acceptance test `unavailable_resources_do_not_block_profile_management` walks exactly this table in one run, in the
right order: add → make both resources vanish → validate is red → five management commands are green → add another folder →
adding a missing root is still refused → remove. It also asserts the two error codes appear **together** in one
`config validate`, which is the aggregate behaviour `cli.md` §2.2 promises.

## 2. The negative control is real, and it failed for the right reason

`e86c5fe` adds two lines to `validate_config`:

```rust
// Negative control: intentionally reproduce F-22.
errors.extend(validate_environment(&config));
```

That is the most honest possible control — it puts the old trap back exactly where it was, without touching the tests. Run
**42046344048** at `e86c5fe` is red, and I read the annotation rather than the summary: `test (ubuntu-24.04)` job
127310206029 failed with

```
test unavailable_resources_do_not_block_profile_management ... FAILED
panicked at crates/auto-tzim/tests/config_cli.rs:974
test result: FAILED. 33 passed; 1 failed
```

and the same on macOS. Line 974 is the `folder list` / `folder show` / `disable` / `edit` / `enable` loop — the first thing
that tries to manage a profile whose disk is gone. So the test fails **when the trap returns** and passes when it does not,
which is the only property that makes a green run mean anything. `d0ac17e` removes the two lines; the final head contains no
trace of the control.

Note the shape of this evidence, because it is better than the earlier six: the control is one line of production code, the
red run is on **both** platforms, and the failing assertion is the acceptance point itself — not a proxy for it.

## 3. F-23, F-24, F-25

| # | Asked for | Result |
| --- | --- | --- |
| F-23 | no `expect` panic in `main.rs` | **fixed** — five sites now `ok_or_else(|| invalid("internal error: mutating command without config lock"))`. Still not a panic, and it names the condition. See F-26 below for the code it returns |
| F-24 | `tzim_path_not_executable` must not be reused for unrelated errors | **fixed** — non-file now reports `tzim_path_not_executable: not a regular file`; `tzim_path_missing` carries the OS error; a directory is no longer called "not executable" without saying why |
| F-25 | state the Windows scope in the code | **fixed** — `// v0.1 supports Unix platforms only; Windows is outside its stated scope.` next to the `std::os::unix` import |

## 4. Two Low findings, not blockers

**F-26 (Low, contract) — the internal-error branch returns exit `2`, the contract says `7`.** `cli.md` §4 assigns `7` to
"internal error: unreadable/corrupt state, unexpected exception, child output our guard cannot attribute to a file", and the
precedence table puts `7` above `2`. The unreachable branch in `main.rs` reports `ConfigError::Invalid`, i.e. `2`. It is
unreachable today, which is exactly why it is cheap to make right: add an `Internal` variant that maps to `7`. Not a reason
to hold AT-02 — a wrong code that nothing can trigger today still becomes true the first time something can.

**F-27 (Low, documentation) — `structural_path` is a filesystem function wearing a structural name.** It calls
`canonicalize_with_missing` first, so "structural" validation does `lstat`/`canonicalize` on existing ancestors, to catch two
names for the same directory. That is a good reason, and the fallback to a lexical form is what keeps an unplugged disk from
failing. The name is what misleads: a later reader will assume "structure" means "no I/O" and put something expensive behind
it, or worse, assume a `load()` never touches the disk. One comment in [D-56](../DECISIONS.md) — "structure means *no
requirement on the environment*, not *no syscalls*" — closes it.

## 5. Everything else, re-verified rather than assumed

| Item | Result | Evidence |
| --- | --- | --- |
| final head green on both platforms | **pass** — 5 of 5 jobs, Rust 1.89 | run **42046562705** at `d0ac17e`; macOS queued 14:36:00 → started 14:36:04 (**4 s**) |
| the acceptance evidence is regenerated **at this head**, not carried over | **pass** | notices on both `test` jobs: identity row, corrupt-config SHA `12f3de65…`, edit SHA/stat pairs, twelve refusal rows, one- and three-error rows |
| F-21, the lock split, the six earlier controls, D-54 items 1–6 | **pass, unchanged** | diff `b1b6d5a..d0ac17e` touches three files and none of that logic |
| structure was not weakened by the split | **pass** | `structural_validation_still_rejects_invalid_unavailable_profiles`: two profiles with the same **missing** root are still `duplicate_root` on `folder list`; a relative `tzim_path` still exits `2` |
| a corrupt config is still never overwritten | **pass** | structure is still checked in `save()`, and the eight-command test still passes at this head |
| no `unsafe`, no `libc`, no spawn in product code | **pass** | `git show d0ac17e:crates/*/src/*` greps clean |
| the PR body was rewritten for this head | **pass** | head SHA, both run IDs, the control's red run, the D-56 section, the per-acceptance map |

## 6. What happens next, in order

1. **#38 first** ([D-56](../DECISIONS.md), this review and the previous one): PR #34 implements D-56, so the decision
   document should land before or with the code, not after it.
2. **#34 then** — AT-02 accepted at `d0ac17e`. Product code, so the merge is the owner's word.
3. F-26 and F-27 in AT-03's first commit, alongside whatever AT-03 needs.
4. R-27 stays open: this run took 4 s, three of the earlier ones took 2–6 s, and one took 15 minutes and never arrived.
