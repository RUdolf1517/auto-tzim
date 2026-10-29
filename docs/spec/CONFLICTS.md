# Specification conflicts, and which text wins

Date: 2026-10-29. Two documents define this project: the technical specification `SPEC.md` (version 0.1) and the
manager charter `MANAGER-SPEC.md` (version 0.2, whose §16 decided findings P-01…P-03). They were written at different
times, so in four places they do not say the same thing. A coder must not have to guess, and a reviewer must not have to
pick a side per pull request, so the resolution is written down here once.

Rule of precedence, taken from the charter itself: **for process, evidence and acceptance the charter wins; for product
behaviour the technical specification wins, except where charter §16 explicitly overrides it.** Where §16 overrides, the
decision is recorded in [DECISIONS.md](../manager/DECISIONS.md) and cited in the row.

| # | Subject | `SPEC.md` (0.1) says | `MANAGER-SPEC.md` (0.2) says | Resolution |
| --- | --- | --- | --- | --- |
| C-1 | What a dry run is | §8.1: a first run requires a preceding `scan`/dry-run; nothing forbids invoking `tzim` in that preview | Invariant 2 + §16 P-02: `auto-tzim` scan/dry-run is a **local read-only candidate planner**; it does not run file-processing `tzim`, does not pass `--dry-run`, writes nothing in the target root; only `tzim --version` is allowed | **Charter wins.** `tzim --dry-run` is forbidden in the MVP. Decision D-04. This also closes the "exact numbers" question: the MVP plan reports *candidates*, not predicted savings |
| C-2 | Exit codes | §8.2: `0/1/2/3/4/5` (4 = lock) | §16 P-03: add `6` (preflight/policy refusal), `7` (internal), `8` (lock); `4` unused; `130/143` for signals; precedence `130/143 > 7 > 2 > 8 > 3 > 6 > 5 > 1 > 0`; raw `tzim` codes only in history | **Charter wins.** Decision D-18/D-19. The technical specification's §8.2 table is superseded, and `docs/cli.md` §4 is the operative table |
| C-3 | `transforms` in the MVP | §6.3: `transforms` is an opt-in field; the MVP may support one | Invariant 11 + §16 P-01: the field must be **empty** in the MVP; confirmation-required transformations do not run | **Charter wins.** Decision D-05: any non-empty `transforms` is a validation error |
| C-4 | Capability discovery | §7.1: record supported formats/opt-ins "if available"; a version-specific adapter is allowed; §2.1 forbids parsing human output | Invariant 18 + §16 P-01: nothing is inferred from `--info`/`--list-formats`/`note`; default set is delegated to the binary without `--formats`; unknown contract ⇒ `incompatible_tzim` | **Charter wins, and it is stricter.** Decision D-01: the only capability call is `tzim --version` |

Two smaller differences are not conflicts but additions the technical specification does not cover; they are decided in
[the decision log](../manager/DECISIONS.md) and flagged there as manager decisions with owner notice:

- **D-16** the shape of our own `--json` output (the technical specification requires JSON, not its field names);
- **D-07** the single `update-check` endpoint (the technical specification requires "only an officially documented
  source", not which one).

## What this means for the coder

1. Read the charter's §6 invariants first, then this page, then `docs/cli.md` §4 (exit codes) and §2.6 (dry run). Those
   three places are where the older text would mislead.
2. If a task seems to require invoking `tzim` on a target path outside an apply run, it is not a task: it is a
   contradiction to raise with the manager before writing code.
3. A pull request that reintroduces `tzim --dry-run`, or that reports `0` for a refused run, is returned as **Critical**
   (charter §9), not discussed as a preference.

## When this page changes

Only when a specification copy changes. Each new copy is recorded in [PROVENANCE.md](PROVENANCE.md), and a conflict that
a new version resolves disappears from the table with the row noted as closed in the commit message.
