# Audits

Independent audits this project asked for, and the evidence behind them. The convention is taken from the `tzim`
repository ([tzim docs/audits](https://github.com/RUdolf1517/tzim/tree/main/docs/audits)), because two projects kept by the
same person should not invent two ways to record what was true at a commit.

## What belongs here

A record of **what was true at one commit**, what was checked, and what was found — not documentation of the product. A
conclusion that survives the audit is copied into the document it belongs to ([guarantees](../guarantees.md),
[safety](../safety.md), [formats](../formats.md)) and into [CHANGELOG](../../CHANGELOG.md). The audit keeps saying what it
said, even after the thing it found has been fixed.

Typical audits for this project: a check of the `tzim` contract against a new release, a review of the pinned binary's
observable behaviour, a safety-invariant review of a candidate build.

## Rules

- **Name:** `YYYY-MM-DD-<subject>.md` — the date the audit was written and what it audited. When it audits an upstream
  release, the version goes in the name (`2026-11-01-tzim-v0.16.1.md`), because that is the thing that will move under us.
- **Language:** English, like the rest of `docs/`.
- **State the baseline.** The exact commit (ours) and the exact version/commit (theirs), and every `file:line` reference
  pointing at the final one.
- **State the limits.** What could not be run, and therefore which claims are about source rather than behaviour.
  "Not executed" is a finding about the audit, and it is written down.
- **Frozen.** Once its findings are filed as issues, the file is not edited. The fix, its test and its CHANGELOG entry
  are the record of the fix; the audit is the record of the problem.
- **Close it.** A finished audit carries `Status: closed` in the header and a map from each finding to the issue or the
  document that now carries it, so the paper trail ends somewhere actionable.
- **No self-audit.** An audit of work performed by an agent is performed by a different agent, or by the owner. The
  manager reviewing the coder is the normal case; a coder auditing its own implementation is not an audit.

## Current contents

| File | Audits | Status |
| --- | --- | --- |
| [2026-11-01-tzim-v0.16.1.md](2026-11-01-tzim-v0.16.1.md) | every `tzim` contract fact this project relies on, against `tzim` v0.16.1 (`f605c2ea`) | open — the drift it found is not yet fixed |

Earlier upstream review lives in [docs/manager/TZIM-STUDY-2026-10-29.md](../manager/TZIM-STUDY-2026-10-29.md). It is a
manager record rather than an audit entry — it was written before this convention existed, and it is preserved as written
rather than retro-fitted into the folder.
