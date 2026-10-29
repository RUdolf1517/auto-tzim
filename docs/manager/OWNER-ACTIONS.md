# Owner actions: commands I cannot run myself

Date: **2026-10-29**. This page exists because the GitHub integration in this session can **create** issues and labels
but every edit, comment, label-attach and pull-request call returns:

```text
HTTP 403  Resource not accessible by integration
```

Reproduced for: `gh issue edit`, `gh issue comment`, `gh api -X PATCH …/issues/10`, `gh api -X POST …/labels`.
Reads and creations work, so everything below is a *small* action with a *known* target, not a workaround for missing
evidence.

Every command is copy-paste ready from an interactive shell in this repository, run with your own credentials. I am not
asking for a token and none is needed here: these are your permissions, not mine.

After running them, nothing else in the project changes — no acceptance row depends on labels, and the one wrong
reference inside #10 is already corrected in the repository copy of the assignment.

---

## 1. Fix the one wrong reference in issue #10

**Why:** the body says the upstream request is `RUdolf1517/tzim#85`; the issue I actually filed is **#86**. #85 is a
different, existing issue in `tzim`. The repository copy
([ASSIGNMENT-AT-MVP-DRAFT.md](ASSIGNMENT-AT-MVP-DRAFT.md), [ROADMAP-ISSUES.md](ROADMAP-ISSUES.md)) already carries the
correct number; only the GitHub body is wrong.

```sh
# what it does: reads the current body, replaces #85 with #86 in the one line that
# refers to the upstream request, writes it back. Idempotent — running it twice is harmless.
gh issue view 10 --repo RUdolf1517/auto-tzim --json body -q .body \
  | sed 's|RUdolf1517/tzim#85|RUdolf1517/tzim#86|' > /tmp/i10.md \
  && gh issue edit 10 --repo RUdolf1517/auto-tzim --body-file /tmp/i10.md
```

Verify:

```sh
gh issue view 10 --repo RUdolf1517/auto-tzim --json body -q .body | grep -c 'tzim#86'   # expect 1
gh issue view 10 --repo RUdolf1517/auto-tzim --json body -q .body | grep -c 'tzim#85'   # expect 0
```

## 2. Attach the labels that already exist

**Why:** the labels `v0.1`, `manager`, `assignment`, `upstream` were created in the repository; attaching them failed
with the same 403. Without them the issue list has no stage grouping, which matters once #11 does not exist yet and
everything is in one flat list.

```sh
# AT-01 … AT-09 are the MVP stage
for n in 1 2 3 4 5 6 7 8 9; do
  gh issue edit "$n" --repo RUdolf1517/auto-tzim --add-label v0.1
done

# the consolidated assignment is both a manager record and an assignment
gh issue edit 10 --repo RUdolf1517/auto-tzim --add-label assignment --add-label manager

# the upstream request lives in tzim, not here; `upstream` stays unused in this repository
gh issue edit 86 --repo RUdolf1517/tzim --add-label enhancement
```

Verify:

```sh
gh issue list --repo RUdolf1517/auto-tzim --limit 20 --json number,labels \
  --jq '.[] | "#\(.number) [\([.labels[].name] | join(","))]"'
```

Expected: `#1…#9` carry `v0.1`; `#10` carries `assignment,manager`.

## 3. Repository description (optional, one line)

`gh repo view` reports an empty description, while `tzim`'s is a full sentence. A one-line description makes the two
repositories consistent in search results and in the organisation's repository list.

```sh
gh repo edit RUdolf1517/auto-tzim \
  --description "Safe orchestration for tzim: explicit folders, read-only candidate plans, no codecs of its own" \
  --add-topic tzim --add-topic file-optimization --add-topic orchestration
```

Verify:

```sh
gh repo view RUdolf1517/auto-tzim --json description,repositoryTopics --jq '{description, topics: [.repositoryTopics[].name]}'
```

## 4. Canonical home of the specification (Q9, optional)

**Why:** `SPEC.md` and `MANAGER-SPEC.md` live only on the pushed branch `arena/b3d4085f-tzim` (commit `efc31756`) in
`tzim`, not on `main`. Until that docs-only commit is merged, "specification version 0.1" resolves to a branch that may
disappear. This repository keeps a verbatim copy with checksums
([provenance](../spec/PROVENANCE.md)), so nothing is at risk — but the canonical home should not be a branch.

```sh
# docs-only commit; check the diff is two markdown files before merging
gh pr create --repo RUdolf1517/tzim \
  --base main --head arena/b3d4085f-tzim \
  --title "docs: add auto-tzim technical and manager specifications" \
  --body "Docs-only: handoff/auto-tzim/SPEC.md (v0.1) and MANAGER-SPEC.md (v0.1). No code, no CI change."

gh pr diff <PR-number> --repo RUdolf1517/tzim --name-only   # expect exactly two paths under handoff/auto-tzim/
gh pr merge <PR-number> --repo RUdolf1517/tzim --merge
```

Note: the charter copy in `tzim` would still be **v0.1** after that merge. The owner's v0.2 text lives in this repository
([MANAGER-SPEC.md](../../MANAGER-SPEC.md)); if the canonical home should carry v0.2 as well, that is a second docs-only
commit in `tzim`, and I would then update
[PROVENANCE.md](../spec/PROVENANCE.md) with the new source commit.

## 5. Clearing blocker B-01 (not a command, a decision)

The MVP cannot start until the previous `tzim` set is resolved (charter §3.2, blocker **B-01** in #10). Two shapes:

```sh
# a) finish it: the failing jobs on PR #84 are measurement jobs, not code jobs
gh pr checks 84 --repo RUdolf1517/tzim
# ... fix or regenerate what they measure, then:
gh pr merge 84 --repo RUdolf1517/tzim --merge

# b) park it explicitly, and say which issues are deferred
gh issue comment 84 --repo RUdolf1517/tzim --body "Parked: auto-tzim MVP starts; the two measurement jobs are deferred to <date/issue>."
```

Then, and only then:

```sh
gh issue comment 10 --repo RUdolf1517/auto-tzim \
  --body "start — B-01 cleared: <merge link or park comment>. MVP assignment authorised."
```

## 6. Branch protection (optional, later)

Not needed for the MVP (one coder, one PR, owner merges). If you want it enforced by the platform rather than by the
charter, the minimum that matches §5 and §14 is: require a pull request, require the CI checks, forbid force-pushes, and
do not allow the author to merge their own PR.

```sh
gh api -X PUT repos/RUdolf1517/auto-tzim/branches/main/protection \
  -F required_status_checks[strict]=true \
  -F 'required_status_checks[contexts][]=test' \
  -F enforce_admins=false \
  -F required_pull_request_reviews[required_approving_review_count]=1 \
  -F restrictions=null
```

Careful: the check name `test` must match the CI job name that AT-01 (#1) will introduce; set this **after** the first
workflow lands, or the rule will block every pull request.

---

## Summary of what changes when

| Action | Needed for | Blocking? |
| --- | --- | --- |
| 1. Fix #85 → #86 in #10 | accuracy of the record | no |
| 2. Attach labels | grouping the issue list | no |
| 3. Repository description | consistency with `tzim` | no |
| 4. Merge the docs commit in `tzim` | canonical spec home (Q9) | no — a checksummed copy is here |
| 5. Clear B-01 | **the MVP start** | **yes** |
| 6. Branch protection | platform-enforced merge rules | no |

If any command fails with 403 as well, the token lacks issue/pull-request write scope: that is a GitHub connection
setting for this conversation, not something to work around in the repository.
