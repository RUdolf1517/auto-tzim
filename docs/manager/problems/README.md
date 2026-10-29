# Problem notes

Three questions were raised in [QUESTIONS-TO-OWNER.md](../QUESTIONS-TO-OWNER.md) on 2026-10-29. The owner's answer
to all three was the same: **do not decide, do not act — describe the problem in detail.** This directory is that
description. It is analysis, not a decision log: nothing here is adopted, nothing here is filed upstream, and no
implementation depends on it yet.

| Note | Subject | Blocks | Owner answer |
| --- | --- | --- | --- |
| [P-01](P-01-machine-interface.md) | `tzim` exposes no machine-readable capability list and no structured skip/refusal reason | AT-04 design, AT-08 test design | describe only |
| [P-02](P-02-dry-run-is-not-write-free.md) | A `tzim` dry run writes and deletes a real candidate in the destination directory | AT-03/AT-04 planner, documentation of invariant 2 | describe only |
| [P-03](P-03-exit-codes.md) | `SPEC.md` §8.2 has no exit code for a run refused before processing, and the two exit-code vocabularies collide on `4` | AT-06 | describe only |

Each note follows the same shape: what exists, what is missing, what it breaks in `SPEC.md` terms, what the options
cost, what evidence would settle it, and what was **not** done. The notes deliberately stop short of a recommendation
strong enough to be mistaken for a decision; where a default is named it is labelled *manager's default, unapproved*.

Why these three and not others: all three were found by reading `tzim` at `d4e4597` before any `auto-tzim` code
existed ([study](../TZIM-STUDY-2026-10-29.md)). They are cheap to describe now and expensive to discover after the
MVP is written, because each one silently decides what a "safe result" means.
