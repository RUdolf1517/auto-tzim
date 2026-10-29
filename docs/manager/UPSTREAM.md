# Upstream: what we need from `tzim`, and where it stands

Date: **2026-10-29**. `auto-tzim` changes nothing in `tzim` (charter §3.6). This page tracks the other direction: the
issues and pull requests in `RUdolf1517/tzim` that affect us, what we assume about each, and what would change here if it
landed.

State was read with `gh` on 2026-10-29 and is **a snapshot, not a subscription** — re-read it before quoting it.

## 1. Open items that matter to us

| Item | State | What it is | Our relation to it |
| --- | --- | --- | --- |
| `tzim#85` | OPEN | `feat(cli): add versioned machine-readable capabilities and structured result reason codes` — created 19:59Z | **Canonical** upstream request for the interface our adapter lacks (study G-1, G-2). Not a dependency; nothing here waits for it |
| `tzim#86` | OPEN | My duplicate of #85, filed 20:11Z without re-listing the open issues | **To be closed as a duplicate** ([OWNER-ACTIONS](OWNER-ACTIONS.md) §1). Its only unique content is the concrete JSON sketches |
| `tzim#83` | OPEN | `feat(cli): add interactive confirmation for opt-in format handling without a force bypass` | **Compatibility condition.** The day a `tzim` can prompt, our invariant 12 (no prompt, no PTY) needs a documented non-TTY refusal on their side. Until then the adapter pins a prompt-free range |
| `tzim#82` | OPEN | `feat(jpeg): ship V1 verification and enable baseline JPEG by default` | Would change the *default* format set of a future binary. Our rule already survives it: we never display a default set, we delegate by omitting `--formats` (D-11) |
| `tzim#78`, `#59`, `#57` | OPEN | FITS/PDF measurement and evidence work | Not ours; relevant only because their completion is precondition §3.2 (blocker B-01) |
| `tzim` PR #84 | OPEN, MERGEABLE, updated 20:18Z | `Extend residual FITS acceptance coverage for #78` | **Blocks the MVP start** under charter §3.2, not our design |
| `tzim` release `v0.14.0` | published | The artifact our real-binary suite pins | Pinned by version **and** SHA-256; `latest` is forbidden (D-13) |

## 2. What we rely on, and how we would notice it changing

| We rely on | Where it is written | How a change becomes visible here |
| --- | --- | --- |
| `tzim --version` prints `tzim <semver>`, exit `0` | `src/main.rs:22` | `A-02`…`A-06` cases fail with `incompatible_tzim` (exit `3`) |
| the `--json` result stream: one `file` row per file, one `summary` with `schema: 1` | `docs/cli.md` §4 | `A-07`…`A-09` cases: missing schema, unknown status, extra field |
| exit codes `0/1/2/3/4` with the documented meanings | `docs/cli.md` §5 | `R-10` (verification failure) and `R-11` (usage) cases |
| `skipped` is generic, the reason lives in a free-text `note` | `docs/cli.md` §4 | `D-02` mapping test; we never classify from the note |
| no prompt, no stdin read in the current binary | `grep -rn stdin src/` → no hits at `d4e4597` | `R-01` (blocked stdin) and `R-02` (no TTY) |
| no network from the binary | `docs/releases.md` §1 | `W-01`: resolver disabled, all commands green |

A change to any of these is not "handled by updating docs": it fails a case, the job goes red, and the adapter entry is
revised in a reviewed change with its own evidence.

## 3. Compatibility matrix

Deliberately empty until a binary has actually been tested against the suite. A range that was never run is a claim, not a
matrix.

| `auto-tzim` | `tzim` versions tested | Artifact (target) | SHA-256 of tested binary | Adapter |
| --- | --- | --- | --- | --- |
| — | none yet | — | — | — |

Filling a row requires: the pinned artifact hash, the T1–T10 contract cases green, and the `A-*` adapter cases green on
that binary. Two rows for two builds of the same version are allowed and expected — the FITS and `jpeg-v1` features prove
that a version string is not a capability statement.

## 4. If #85 lands

What we would do, in order — and what we would **not** do:

1. Read the new interface as a contract, not as a convenience: `version`, `json_schema`, `formats`, `features`, `opt_ins`
   (including `confirmation_required`).
2. Replace the version-keyed adapter table with a capability table, keeping the fail-closed rule: an unknown field or an
   unparsable document is still `incompatible_tzim`.
3. Keep `--version` as the identity probe; add a `--capabilities --json` probe to `doctor`.
4. Delete nothing from the corpus that the interface makes easier — the `A-*` cases stay, because a future binary can
   regress.
5. **Not** start displaying "the default set" as a promise; it remains a delegation (D-11).
6. **Not** enable `transforms` on the strength of an interface alone: charter §6-11 also requires an installed build that
   has the handler and a real-binary test.

## 5. If #83 lands

Charter invariant 12 survives only if `tzim` documents, and we can test, that a non-TTY invocation **refuses** with a
documented status and exit code instead of waiting. Until then:

- the adapter range stays prompt-free;
- `R-01` (a child that blocks on stdin is killed by our timeout) stays in the corpus as a defence in depth, not as a
  substitute for their contract;
- a `tzim` that prompts with `stdin=/dev/null` would fail `R-01` in CI, which is exactly the intended outcome.

## 6. Rules for filing anything upstream

Learned the hard way this week (I filed a duplicate):

1. **List the open issues immediately before filing** (`gh issue list --repo RUdolf1517/tzim --state open`), and read the
   two or three closest titles.
2. One request per issue, with the concrete shape (fields, example output) and an explicit "not a dependency" line.
3. No pull request, no branch, no code change from this project without the owner's instruction.
4. Record the number in this file and in [DECISIONS](DECISIONS.md) D-22 in the same turn it is created.
