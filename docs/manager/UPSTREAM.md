# Upstream: what we need from `tzim`, and where it stands

**Read on 2026-11-01 at `f605c2ea` (v0.16.1).** `auto-tzim` changes nothing in `tzim` (charter §3.6). This page tracks the
other direction: the issues, pull requests and releases in `RUdolf1517/tzim` that affect us, what we assume about each, and
what changes here when it moves.

A snapshot, not a subscription — re-read it with `gh` before quoting it. Every revision of this page carries a read date
for the same reason.

## 1. State of the upstream tracker

**Zero open pull requests. Zero open issues.** The set the coder was assigned is finished, and three releases shipped in
the days after it:

| Release | Published | What it means for us |
| --- | --- | --- |
| [`v0.15.0`](https://github.com/RUdolf1517/tzim/releases/tag/v0.15.0) | 2026-10-30 | opt-in FLAC (#79) and opt-in PDF (#80) handlers; the #57/#59 measurement work resolved as *refused after measurement* / *threshold not applied*, not as admission |
| [`v0.16.0`](https://github.com/RUdolf1517/tzim/releases/tag/v0.16.0) | 2026-11-01 | **`--capabilities`, JSON result schema 2 and `reason_code`** (#85); Windows per-user installer; upstream `SECURITY.md` |
| [`v0.16.1`](https://github.com/RUdolf1517/tzim/releases/tag/v0.16.1) | 2026-11-01 | performance and verification internals; **exit-code precedence documented and enforced** (#109) |

### Closed items that used to be on this page

| Item | Outcome |
| --- | --- |
| PR #84 — residual FITS acceptance coverage | **merged** 2026-10-30T14:55:59Z. This was blocker **B-01**; charter §3.2 is now satisfied |
| `tzim#85` — capabilities + structured reasons | **closed as implemented**, PR #91, merge `503d1eca`. Shipped in v0.16.0 |
| `tzim#86` — my duplicate of #85 | **closed**. The duplication mistake is recorded in [STATUS-LOG](STATUS-LOG.md) and was never repeated |
| `tzim#83` — interactive confirmation | **shipped**: a default-No prompt for `--fits-tile` on a single file on a TTY, never for non-TTY, `--json`, `--dry-run` or batch runs |
| `tzim#82` — JPEG by default with V1 verification | **shipped**, with build-aware wording in `--list-formats` |
| `tzim#78`, `#57`, `#59` — FITS/PDF measurement | closed with measured, pre-registered outcomes; several are refusals, and upstream says so plainly |

**The consequence:** four things this project's documents describe as missing or future now exist. The full list, with
`file:line` and severity, is the [2026-11-01 audit](../audits/2026-11-01-tzim-v0.16.1.md). It is not a footnote — it is the
reason the MVP should not start until the documents are corrected.

## 2. What we rely on, and how we would notice it changing

Updated for v0.16.1. The third column is the point: a dependency is only trustworthy if something goes red when it moves.

| We rely on | Where it is written | How a change becomes visible here |
| --- | --- | --- |
| `--version` prints `tzim <semver>`, exit `0` | `src/main.rs`, `docs/cli.md` §2 | `A-02`…`A-06`: unparsable or unknown version ⇒ `incompatible_tzim` (exit `3`) |
| `--capabilities` prints one JSON object, `capabilities_schema_version: 1`, `result_schema_version: 2` | `docs/cli.md` §"capabilities document" | `A-10`: missing document, unknown schema version or unknown opt-in policy ⇒ fail closed |
| the `--json` result stream: one `file` row per file, one `summary` with `schema: 2`, `reason_code` on non-optimizing rows | `docs/cli.md` §4 | `A-07`…`A-09`: missing schema, unknown status, unknown schema version |
| exit codes `0/1/2/3/4`; precedence `2 → 4 → 3 → 1 → 0`; `EPIPE` preserves an earned code; missing input is `1` | `docs/cli.md` §5, CHANGELOG v0.16.1 #109 | `R-10`, `R-11`, `X-01`…`X-16` |
| a refusal is `skipped` (or `unsupported`) with a stable `reason_code` and a human `note` | `docs/cli.md` §4 | `D-02` mapping; we never classify from the note |
| no prompt, no stdin read on any non-TTY path | `docs/cli.md` §"FITS tile confirmation" | `R-01`, `R-02`: a child waiting on stdin is killed |
| no network from the binary | `docs/releases.md` §1 | `W-01`: resolver disabled, all commands green |
| release artifacts: stable unversioned URLs plus `SHA256SUMS` | `docs/releases.md` §3 | `pinned.py`: a missing or mismatched checksum fails the run (exit `5`), never a fallback to `latest` |

## 3. Compatibility matrix

Deliberately empty until a binary has actually been tested against the suite. A range that was never run is a claim, not a
matrix.

| `auto-tzim` | `tzim` versions tested | Artifact (target) | SHA-256 of tested binary | Adapter |
| --- | --- | --- | --- | --- |
| — | none yet | — | — | — |

Filling a row requires: the pinned artifact hash, the contract cases green, and the `A-*` adapter cases green on that
binary. Two rows for two builds of the same version are expected — the FITS and `jpeg-v1` features prove that a version
string is not a capability statement. **No binary has been executed by this project yet**; every claim on this page is a
source claim, and the audit says so in its limits section.

## 4. The interface landed — what happens now

`tzim#85` was filed by this project on 2026-10-29 and shipped two days later. The plan when it lands was written before it
did, which is the only reason this page does not have to be improvised now:

1. Read the document as a contract: `build_features`, `compiled_formats`, `default_enabled_formats`, `selection_flags`,
   `handler_opt_ins`, `transformations`, `confirmation_policies`.
2. Keep the fail-closed rule: an unknown `capabilities_schema_version`, an unknown `result_schema_version`, or an unknown
   opt-in policy is still `incompatible_tzim`. Unknown *fields* may be ignored only where our behaviour does not depend on
   them.
3. Keep `--version` as the identity probe and add a `--capabilities` probe to `doctor`.
4. Delete nothing from the corpus that the interface makes easier — `A-*` cases stay, because a future binary can regress.
5. **Do not** start displaying "the default set" as a promise of what will be optimized. It is now *observable* rather than
   a guess, which is a different and better thing — but the owner has to decide whether we consume it (audit DR-05).
6. **Do not** enable `transforms` on the strength of an interface alone: charter §6-11 also requires an installed build
   with the handler and a real-binary test.

## 5. Rules for filing anything upstream

Learned the hard way in October (I filed a duplicate of #85 and had to close it):

1. **List the open issues immediately before filing** (`gh issue list --repo RUdolf1517/tzim --state open`), and read the
   two or three closest titles.
2. One request per issue, with the concrete shape (fields, example output) and an explicit "not a dependency" line.
3. No pull request, no branch, no code change from this project without the owner's instruction.
4. Record the number here and in [DECISIONS](DECISIONS.md) in the same turn it is created.
5. Check the tracker is idle before assuming it: as of 2026-11-01 it is empty, so a new issue would be the only open one.
