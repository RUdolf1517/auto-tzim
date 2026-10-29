# Review template (charter §7, §12, §15)

Copy this file for every review: `docs/manager/reviews/REVIEW-<head-sha>.md`. The repository keeps review reports as
project records; a review that exists only in a chat is not a record.

Rules that the template enforces:

1. **One review per head SHA.** Any later change invalidates the rows it could affect; they are re-run, not argued.
2. **The author does not review their own work** and the reviewer does not merge (charter §2.2, §14).
3. **Every claim gets a mark**, and only the four marks below. An unmarked claim is `coder-reported` by default.
4. **Every finding carries an issue id, a severity per charter §9, and a raw evidence pointer.** A finding without those
   three is an opinion and blocks nothing.
5. **Exactly one verdict** at the end, from the two allowed strings.

---

```markdown
# Review — <head SHA>

Reviewer: <name/role>
Date: <YYYY-MM-DD>
Scope: AT-01 … AT-09 (#1 … #9), assignment #10
Specification: SPEC.md 0.1 (sha256 250d0db1…) · MANAGER-SPEC.md 0.2 (sha256 7e798361…)
Gates: GATES.md (13 acceptance rows, 19 invariants) · Corpus: FIXTURE-CORPUS.md (111 cases)

## 1. Binding

| Check | Result | Evidence |
| --- | --- | --- |
| claimed head SHA == PR head | | `gh pr view <PR> --json headRefOid` |
| SHA reachable from the PR | | |
| commits labelled `AT-0N` | | |
| no uncommitted/unpushed work claimed as done | | |

## 2. CI

| Job | Platform | Python | Status | Run id | Head SHA |
| --- | --- | --- | --- | --- | --- |
| test | ubuntu-latest | 3.12 | | | |
| test | macos-latest | 3.12 | | | |
| … | | | | | |

Skipped / cancelled / neutral jobs: <none | list — each is a failed row>

## 3. Row reconciliation

| Quantity | Claimed | Recomputed | Match |
| --- | --- | --- | --- |
| corpus cases run | | | |
| cases skipped (+ reason) | | | |
| status counts | | `jq -r .status … \| sort \| uniq -c` | |
| artifacts and checksums | | `sha256sum -c` | |

## 4. Acceptance matrix

| Row | Verdict | Mark | Evidence |
| --- | --- | --- | --- |
| AC-1 deterministic candidate plan | | | |
| AC-2 scan is read-only (invocation counter + unchanged root + read-only mount) | | | |
| AC-3 only computed argv | | | |
| AC-4 no non-candidate file modified | | | |
| AC-5 no shell / no prompt / no PTY / no root | | | |
| AC-6 no retry loop | | | |
| AC-7 recovery after cancel and crash | | | |
| AC-8 exit codes, `4` never, precedence | | | |
| AC-9 update check notifies only | | | |
| AC-10 docs match behaviour | | | |
| AC-11 preflight refusals return 6 | | | |
| AC-12 plan binding | | | |
| AC-13 fail-closed adapter, no prose | | | |
| INV-1 … INV-19 | | | (one line per invariant that applies to this stage) |

## 5. Controls I ran myself

| Control | Command | Result |
| --- | --- | --- |
| break one expectation, suite must fail | | |
| block the resolver, all commands but update-check work | | |
| hold the lock ⇒ exit 8 | | |
| touch a planned file ⇒ exit 6 | | |
| rename a fixture to `--json` | | |
| five random corpus cases re-run | | |

## 6. Findings

| ID | Severity | Issue | What is wrong | Evidence | Required change | State |
| --- | --- | --- | --- | --- | --- | --- |
| F-01 | Critical/High/Medium/Low | AT-0N (#) | | file:line, case id, row | | open/closed |

Critical findings stop the review; High findings stop acceptance. Neither may be downgraded because the fix is hard or the
release is late (charter §9).

## 7. Claims re-marked

| Claim | Author's mark | Reviewer's mark | Note |
| --- | --- | --- | --- |
| | | | |

## 8. Verdict

PASS — ready for owner merge/release decision
   |  NOT READY — findings: F-0N, F-0N

Signed: <reviewer>  ·  Date: <YYYY-MM-DD>
Merge performed by reviewer: no
```

---

## What "PASS" does not mean

`PASS` means *this scope, at this SHA, with this evidence, has no unresolved Critical/High finding and every gate row is
satisfied*. It does not mean the product is released, that the owner has merged, or that a later stage may start: each of
those is a separate decision, and the merge itself is the owner's (charter §14).

## Where the review lives

| Artifact | Location |
| --- | --- |
| the review report | `docs/manager/reviews/REVIEW-<sha>.md` |
| the evidence bundle it cites | `evidence/<sha>/` (not committed if large; checksums always are) |
| the findings that survive the review | new or updated issues, each linked from the report |
| the verdict, one line | the [status log](STATUS-LOG.md) |
