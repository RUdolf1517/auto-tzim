# Provenance of the specification copies

The two documents in the repository root are **verbatim copies**, not paraphrases. They are the contract; this
file records where they came from so that a later change can be detected instead of argued about.

Retrieved: **2026-10-29**

| File in this repository | Source repository | Source ref | Source commit | Lines | SHA-256 |
| --- | --- | --- | --- | --- | --- |
| `SPEC.md` | `RUdolf1517/tzim` | branch `arena/b3d4085f-tzim` | `efc317561d0ec07f4ed6cac3fb90ae643d9c5101` | 419 | `250d0db1c705cdc714142078a67592bed355ec19e569b0627daa5d8dff1f660c` |
| `MANAGER-SPEC.md` | `RUdolf1517/tzim` | branch `arena/b3d4085f-tzim` | `efc317561d0ec07f4ed6cac3fb90ae643d9c5101` | 341 | `25a7d3cd1262510da9719852c6131808b71c380d829cdfab04bc01ed1ba1959b` |

Both files live in the source repository under `handoff/auto-tzim/`. The commit that introduced them is
`docs: add auto-tzim technical and manager specifications` (`efc31756`), which is on a pushed branch and **not
merged into `tzim` `main`** (`main` was at `d4e45974b32fc2bb696eee346892846fe3e0a5d1` on the same date). The
technical specification states its own version: **0.1**.

Retrieval command (reproducible):

```sh
gh api -H "Accept: application/vnd.github.raw" \
  repos/RUdolf1517/tzim/contents/handoff/auto-tzim/SPEC.md?ref=arena/b3d4085f-tzim > SPEC.md
gh api -H "Accept: application/vnd.github.raw" \
  repos/RUdolf1517/tzim/contents/handoff/auto-tzim/MANAGER-SPEC.md?ref=arena/b3d4085f-tzim > MANAGER-SPEC.md
sha256sum SPEC.md MANAGER-SPEC.md
```

## Rules for changing these copies

1. The copies are not edited in place. A specification change is a change in the source repository, followed by
   a new copy with a bumped version line and a new row in the table above.
2. Any implementation or review that cites "the spec" cites the SHA-256 above. A statement that cannot be
   traced to one of these two files, or to a decision in
   [`docs/manager/DECISIONS.md`](../manager/DECISIONS.md) that the owner approved, is not a requirement.
3. Discrepancies between the copies and observed `tzim` behaviour are recorded in
   [`docs/manager/TZIM-STUDY-2026-10-29.md`](../manager/TZIM-STUDY-2026-10-29.md); they are not silently
   "fixed" in the copy.
4. The manager does not own the text of `SPEC.md`. Proposals to change it (for example the exit-code gap in
   decision D-03) are raised in [`docs/manager/QUESTIONS-TO-OWNER.md`](../manager/QUESTIONS-TO-OWNER.md) and
   stay proposals until the owner approves them.
