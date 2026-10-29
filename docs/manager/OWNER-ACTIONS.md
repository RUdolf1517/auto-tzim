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

## 1. Close my duplicate upstream issue

**Why:** I filed `RUdolf1517/tzim#86` for a capabilities/structured-reasons API **without re-listing the open issues
first**. `RUdolf1517/tzim#85` already covered exactly that request, was created twelve minutes earlier, and is the better
text. #85 is canonical; my #86 is noise in someone else's tracker.

Nothing in `auto-tzim` depends on the number: #10 cites **#85**, which was correct all along — my earlier "correction" of
that reference was the mistake, and it has been reverted in the repository copy of the assignment.

```sh
gh issue close 86 --repo RUdolf1517/tzim --reason "duplicate" \
  --comment "Duplicate of #85, which says the same thing better. Filed by the auto-tzim project by mistake without checking the open list first. Nothing here depends on this issue: auto-tzim#10 references #85. Sorry for the noise."
```

If you would rather keep the extra detail from #86 (the concrete JSON sketches), the alternative is to paste those two
code blocks into #85 as a comment and then close #86 the same way.

Verify:

```sh
gh issue view 86 --repo RUdolf1517/tzim --json state --jq .state      # expect "CLOSED"
gh issue view 85 --repo RUdolf1517/tzim --json state --jq .state      # expect "OPEN"
gh issue view 10 --repo RUdolf1517/auto-tzim --json body -q .body | grep -c 'tzim#85'   # expect 1, no edit needed
```

## 2. Attach the labels that already exist

**Why:** the labels `v0.1`, `manager`, `assignment`, `upstream` were created in the repository; attaching them failed
with the same 403. Without them the issue list has no stage grouping.

`gh issue edit` accepts **one** issue number (`{<number> | <url>}`), so there is no comma list and no variadic form. Run
one line per issue — no `for` loop, no `$n`, no inline comments. This block is syntactically valid in bash and zsh
(checked with `bash -n`):

```sh
gh issue edit 1 --repo RUdolf1517/auto-tzim --add-label "v0.1"
gh issue edit 2 --repo RUdolf1517/auto-tzim --add-label "v0.1"
gh issue edit 3 --repo RUdolf1517/auto-tzim --add-label "v0.1"
gh issue edit 4 --repo RUdolf1517/auto-tzim --add-label "v0.1"
gh issue edit 5 --repo RUdolf1517/auto-tzim --add-label "v0.1"
gh issue edit 6 --repo RUdolf1517/auto-tzim --add-label "v0.1"
gh issue edit 7 --repo RUdolf1517/auto-tzim --add-label "v0.1"
gh issue edit 8 --repo RUdolf1517/auto-tzim --add-label "v0.1"
gh issue edit 9 --repo RUdolf1517/auto-tzim --add-label "v0.1"
```

```sh
gh issue edit 10 --repo RUdolf1517/auto-tzim --add-label "assignment" --add-label "manager"
```

```sh
gh issue edit 86 --repo RUdolf1517/tzim --add-label "enhancement"
```

Other shells (the same effect, chosen by what your terminal actually runs):

```fish
# fish
for n in 1 2 3 4 5 6 7 8 9
    gh issue edit $n --repo RUdolf1517/auto-tzim --add-label "v0.1"
end
```

```powershell
# PowerShell
1..9 | ForEach-Object { gh issue edit $_ --repo RUdolf1517/auto-tzim --add-label "v0.1" }
```

No `gh` at all — the REST endpoint directly (needs a token with `repo` scope in `GH_TOKEN`):

```sh
for n in 1 2 3 4 5 6 7 8 9; do
  curl -sS -X POST -H "Authorization: Bearer $GH_TOKEN" \
    -H "Accept: application/vnd.github+json" \
    https://api.github.com/repos/RUdolf1517/auto-tzim/issues/$n/labels \
    -d '{"labels":["v0.1"]}' >/dev/null && echo "issue $n: ok" || echo "issue $n: failed"
done
```

Or by hand in the browser, which needs no tooling at all: open each issue, **Labels → v0.1**. Nine clicks, and for #10
click `assignment` and `manager`.

Verify:

```sh
gh issue list --repo RUdolf1517/auto-tzim --label v0.1 --state all
```

Expected: #1 … #9. (`gh issue list --json number,labels --jq …` works too, but the plain `--label` filter is shorter and
does not depend on `jq`.)

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

## 6a. Decide what `main` protection should do (now that PR #11 needed a bypass)

**What happened:** `main` is protected with `required_pull_request_reviews`, the pull request was opened by the same
account that owns the repository, GitHub does not allow approving your own pull request, and the only way to land a
documentation-only change was `gh pr merge 11 --merge --admin`. That bypasses the review gate you configured, which makes
it a policy decision rather than a technical detail.

Three options, in order of preference for this project:

```sh
# A. Keep one required review, but let a documentation-only change skip it:
#    add a bypass actor for admins and require the rule only on code paths.
#    (Requires a ruleset rather than classic protection; the review requirement then
#     applies to what matters — src/, tests/, CI — and not to markdown.)
gh api repos/RUdolf1517/auto-tzim/rulesets --method POST \
  -f name="main: code needs review" -f target=branch -f enforcement=active \
  -f 'conditions[ref_name][include][]=refs/heads/main' \
  -F 'rules[][type]=pull_request' \
  -f 'rules[][parameters][required_approving_review_count]=1' \
  -f 'rules[][parameters][dismiss_stale_reviews_on_push]=true'

# B. Drop the review requirement while the project has one human and one agent:
gh api -X DELETE repos/RUdolf1517/auto-tzim/branches/main/protection/required_pull_request_reviews

# C. Keep it exactly as it is, and treat `--admin` as the documented path for
#    documentation-only pull requests opened by the owner's own account.
```

Whichever option you pick, record it in [DECISIONS](DECISIONS.md) — the next merge should not have to rediscover this.

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
| 1. Close `tzim#86` as a duplicate of `tzim#85` | not leaving noise in another project's tracker | no |
| 2. Attach labels | grouping the issue list | no |
| 3. Repository description | consistency with `tzim` | no |
| 4. Merge the docs commit in `tzim` | canonical spec home (Q9) | no — a checksummed copy is here |
| 5. Clear B-01 | **the MVP start** | **yes** |
| 6. Branch protection | platform-enforced merge rules | no |

If any command fails with 403 as well, the token lacks issue/pull-request write scope: that is a GitHub connection
setting for this conversation, not something to work around in the repository.

---

## 7. If the command does not run at all

These are the failures that look like "the command is wrong" but are not, in the order they are worth checking.

| Symptom | Cause | Fix |
| --- | --- | --- |
| `gh: command not found` | GitHub CLI is not installed in that terminal (a different machine from this session) | install `gh`, or use the browser (see §2) or the `curl` variant |
| `syntax error near unexpected token 'do'`, or `end` expected | the terminal runs **fish**; `for … do … done` is bash/zsh syntax | use the `fish` block in §2 |
| `The token in ... is invalid` / `gh auth login` prompt | `gh` is installed but not authenticated | `gh auth login` once; `gh auth status` to confirm |
| `unknown flag: --add-label` | `gh` older than 2.0 | `gh --version`; upgrade, or use `curl` |
| `accepts 1 arg(s), received 0` / `received 9` | the loop lost its variable (bad paste) or someone passed several numbers, which `gh issue edit` does not accept | run the one-line-per-issue block |
| `GraphQL: Resource not accessible by integration (addLabelsToLabelable)` | authenticated, but the token is an App/GitHub-Actions token with no issue write access | use a personal token with `repo` scope, or the browser |
| the whole block is pasted as one line and fails | the paste joined lines without `;` | paste one command at a time |
| labels attach, but nothing shows in `gh issue list` | `--state open` is the default and the filter needs `--label` | `gh issue list --repo RUdolf1517/auto-tzim --label v0.1 --state all` |

Diagnostics to paste into an issue if none of the above fits:

```sh
gh --version
gh auth status
echo "$SHELL"
gh issue edit 1 --repo RUdolf1517/auto-tzim --add-label "v0.1" 2>&1 | head -5
```

The last line is the whole test: in this session it returns
`Resource not accessible by integration` (a permission error, which means the syntax is fine); in a shell where it
prints nothing, the label was attached.
