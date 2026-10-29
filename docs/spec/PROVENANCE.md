# Provenance of the specification copies

The two documents in the repository root are **verbatim copies**, not paraphrases. They are the contract; this file
records where they came from so that a later change can be detected instead of argued about.

Retrieved: **2026-10-29**

| File in this repository | Version | Source | Lines | SHA-256 |
| --- | --- | --- | --- | --- |
| `SPEC.md` | 0.1 | `RUdolf1517/tzim`, `handoff/auto-tzim/SPEC.md`, branch `arena/b3d4085f-tzim`, commit `efc317561d0ec07f4ed6cac3fb90ae643d9c5101` | 419 | `250d0db1c705cdc714142078a67592bed355ec19e569b0627daa5d8dff1f660c` |
| `MANAGER-SPEC.md` | 0.2 | owner message of 2026-10-29 in this project (charter text supplied directly, "учтены P-01/P-02/P-03") | 352 | `7e798361b46c74898c2532448039dba914127c7be94c8b9f76373ddc8fe343f1` |

Version history of the charter copy inside this repository:

| Version | How it arrived | State |
| --- | --- | --- |
| 0.1 | copied from `tzim` `handoff/auto-tzim/MANAGER-SPEC.md` at `efc31756` | superseded by 0.2 |
| 0.2 | supplied by the owner in this project on 2026-10-29; adds invariants 18–19, rewrites invariant 2 and 11, adds §16 (decisions on P-01/P-02/P-03) and the exit-code contract | current |

The technical specification (`SPEC.md`) is still **0.1** and still comes from the pushed branch `arena/b3d4085f-tzim` in
`tzim`; it is not merged into `tzim` `main` (`main` was `d4e45974b32fc2bb696eee346892846fe3e0a5d1` on the same date).
The charter's §16 changes several things the older technical specification states (invariant 2, the empty `transforms`
allowlist, the exit-code table of its §8.2). Where they disagree, the charter wins for *process and acceptance*, and the
technical specification wins for *product behaviour* except in the three places §16 names explicitly. Those three
divergences are recorded as decisions D-02, D-04 and D-18/D-19 and are listed in
[CONFLICTS.md](CONFLICTS.md).

Retrieval command for the technical specification (reproducible):

```sh
gh api -H "Accept: application/vnd.github.raw" \
  repos/RUdolf1517/tzim/contents/handoff/auto-tzim/SPEC.md?ref=arena/b3d4085f-tzim > SPEC.md
sha256sum SPEC.md
```

## Rules for changing these copies

1. The copies are not edited in place. A specification change is a change in the source, followed by a new copy with a
   bumped version line and a new row in the table above.
2. Any implementation or review that cites "the spec" cites the version and the SHA-256 above. A statement that cannot
   be traced to one of these two files, or to a decision in [`docs/manager/DECISIONS.md`](../manager/DECISIONS.md)
   that the owner approved, is not a requirement.
3. Discrepancies between the copies and observed `tzim` behaviour are recorded in
   [`docs/manager/TZIM-STUDY-2026-10-29.md`](../manager/TZIM-STUDY-2026-10-29.md); they are not silently "fixed" in the
   copy.
4. The manager does not own either text. Proposals to change them live in
   [`docs/manager/QUESTIONS-TO-OWNER.md`](../manager/QUESTIONS-TO-OWNER.md) and stay proposals until the owner answers.
