# Problem notes

Three questions were raised in [QUESTIONS-TO-OWNER.md](../QUESTIONS-TO-OWNER.md) on 2026-10-29. The first answer was
**do not decide, do not act — describe the problem in detail**, so this directory was written as that description. The
owner's charter **v0.2** then decided all three (§16). The notes are kept unchanged as the analysis behind the decisions,
with a resolution header on each page.

| Note | Subject | Resolution (charter v0.2) | Decisions |
| --- | --- | --- | --- |
| [P-01](P-01-machine-interface.md) | `tzim` exposes no machine-readable capability list and no structured skip/refusal reason | restricted adapter; default set delegated; generic `skipped_by_tzim`; upstream request `tzim#85` | D-01, D-02, D-05, D-22 |
| [P-02](P-02-dry-run-is-not-write-free.md) | A `tzim` dry run writes and deletes a real candidate in the destination directory | scan/dry-run is a local read-only planner; `tzim --dry-run` forbidden in the MVP; apply bound to a plan | D-04, D-20, D-23 |
| [P-03](P-03-exit-codes.md) | `SPEC.md` §8.2 has no exit code for a run refused before processing, and the two vocabularies collide on `4` | own contract with `6`/`7`/`8`, `4` unused, signals, fixed order and precedence | D-03, D-18, D-19 |

Each note follows the same shape: what exists, what is missing, what it breaks in specification terms, what the options
cost, what evidence would settle it, and what was **not** done. They stop short of a recommendation strong enough to be
mistaken for a decision — which is why the decisions, when they came, came from the owner and are recorded in
[DECISIONS.md](../DECISIONS.md).

Why these three and not others: all three were found by reading `tzim` at `d4e4597` before any `auto-tzim` code
existed ([study](../TZIM-STUDY-2026-10-29.md)). They are cheap to describe now and expensive to discover after the
MVP is written, because each one silently decides what a "safe result" means.
