# Review protocol: how a handoff is verified

Charter §7.2 lists what the manager checks personally. This page turns that list into commands, so that a review is
reproducible by somebody else and cannot degrade into "the coder says it works".

Nothing here is a coder's task; it is what the reviewer does **after** a handoff, in order, with the artifacts in front of
them. Every step has a defined failure.

---

## 0. First: bind the evidence to a SHA

```sh
# 1. What did the coder claim?
#    "Final head SHA: <sha>" — write it down, it is the only SHA that counts.

# 2. Does that SHA exist in the PR, and is it the head?
gh pr view <PR> --repo RUdolf1517/auto-tzim --json headRefOid,headRefName,state,isDraft \
  --jq '{head:.headRefOid, branch:.headRefName, state, draft:.isDraft}'

# 3. Does the working tree match what CI tested? (an unpushed commit is a finding, not a detail)
gh api repos/RUdolf1517/auto-tzim/commits/<sha> --jq '{sha:.sha, parents:[.parents[].sha], date:.commit.author.date}'

# 4. Are the commits labelled by issue, in the order claimed?
gh api repos/RUdolf1517/auto-tzim/commits?sha=<sha>&per_page=100 \
  --jq '.[] | "\(.sha[0:8])  \(.commit.message | split("\n")[0])"'
```

**Fail if:** the head SHA differs from the claimed one, the sha is not reachable from the PR, or a commit in the range is
unlabelled (the issue map cannot be reconstructed).

## 1. CI: the jobs must exist, run, and not be skipped

```sh
gh pr checks <PR> --repo RUdolf1517/auto-tzim
gh run list --repo RUdolf1517/auto-tzim --commit <sha> \
  --json databaseId,status,conclusion,workflowName,event --jq '.[] | "\(.databaseId) \(.workflowName) \(.status)/\(.conclusion)"'
gh run view <run-id> --repo RUdolf1517/auto-tzim --log > /tmp/run-<id>.log   # keep the raw log
```

**Fail if:** a required job is `skipped`, `cancelled`, or `neutral`; a job reports success in under a few seconds for a
suite that must run subprocess tests; the job's `head_sha` is not the claimed SHA; or the log shows a `continue-on-error`
swallowing a red step.

Cross-check the platform matrix: at minimum `ubuntu-latest` **and** `macos-latest`, Python 3.12 and 3.13. A green job on
one platform is half the claim.

## 2. Artifacts: what was attached, and does it belong here

```sh
ls -la <artifacts-dir>
sha256sum -c SHA256SUMS           # inside the artifact bundle
sha256sum <each-file>             # and compare against the values in the handoff text
```

**Fail if:** an artifact has no checksum, a checksum does not match, an artifact's internal `head_sha` field is not the
claimed SHA, or the bundle contains a summary without the raw rows behind it.

## 3. Row reconciliation: the numbers must add up

The handoff states a corpus size and a per-status breakdown. Recompute both from the raw rows:

```sh
# how many rows are there, really?
wc -l < raw-rows.jsonl

# recompute the counts instead of trusting the summary
jq -r '.status' < raw-rows.jsonl | sort | uniq -c | sort -rn

# the summary object must equal the recomputation, exactly
jq -s 'map(select(.type=="run")) | .[-1].counts' < raw-rows.jsonl
```

**Fail if:** `wc -l` differs from the claimed cases minus explicitly-reported `not run` rows; the counts disagree with
`jq`; a status appears in the summary that has no row; or the corpus is smaller than
[FIXTURE-CORPUS.md](FIXTURE-CORPUS.md) requires (111 IDs as of 2026-10-29).

## 4. Byte evidence: the claims that matter most

```sh
# every non-candidate file must be byte-identical before and after
# (the bundle must contain before/after hashes per path; verify a random five by hand on a machine with the fixture)
for p in a.png "b c.png" "$(printf 'newline\nname.png')"; do
  sha256sum "$p"
done
diff <(jq -r '.path + " " + .sha256_before' rows.jsonl | sort) \
     <(jq -r '.path + " " + .sha256_after'  rows.jsonl | grep -f <(jq -r 'select(.status|test("^(skipped|failed|interrupted|hardlink)")) | .path' rows.jsonl) | sort)
```

**Fail if:** any `skipped_*`, `hardlink_skip`, `failed` or `interrupted` row has a differing before/after hash; a
`/tmp/ats_pwned` file from `N-05` exists; a `.tzim-*.tmp` created by us (not by `tzim` during an apply) exists in a target
root.

## 5. The read-only-plan proof (the invariant that changed in charter v0.2)

This is the row a plausible-looking implementation most often gets wrong, so it is verified from raw evidence, not from
the summary:

```sh
# a) the invocation log must show one --version and zero path invocations for scan
jq -r 'select(.event=="spawn") | [.argv[1:], .cwd] | @json' < invocation-log.jsonl

# b) the target root must be identical before and after a scan
( cd <root> && find . -print0 | sort -z | xargs -0 shasum -a 256 > /tmp/before.txt )
auto-tzim scan --folder <id>
( cd <root> && find . -print0 | sort -z | xargs -0 shasum -a 256 > /tmp/after.txt )
diff /tmp/before.txt /tmp/after.txt && echo "root unchanged"

# c) and it must work without write permission
chmod -R a-w <root> && auto-tzim scan --folder <id>; echo "exit=$?"; chmod -R u+w <root>
```

**Fail if:** any spawn during a scan carries a path argument or `--dry-run`; the listing differs; `scan` exits non-zero on a
read-only root; or the evidence for this row is only a claim in prose.

## 6. Exit codes: run the matrix, do not read it

```sh
# a script over the X-* cases; the matrix must be executed, not transcribed
for c in X-04 X-05 X-06 X-08 X-09 X-11 X-12; do
  ./harness/run-case "$c"; printf '%s -> %s\n' "$c" "$?"
done
grep -q ' -> 4$' /tmp/matrix.txt && echo "FAIL: exit 4 produced" || echo "no exit 4"
```

**Fail if:** any documented code is missing from the executed matrix; `4` appears; a preflight refusal returns `0`, `1` or
`2`; or the precedence pairs of D-19 are asserted only by documentation.

## 7. Negative controls the reviewer runs personally

These are cheap and they are what distinguishes evidence from a demo:

1. **Break one expectation** in a copy of the corpus and confirm the suite fails (`sed` one expected status, run, revert).
2. **Block the resolver** (or set an unreachable proxy) and confirm every command except `update-check` still works.
3. **Hold the lock** with a sleeping process and confirm `run --once` exits `8`, not `1`.
4. **Touch one planned file** between `scan` and `run` and confirm exit `6`.
5. **Rename a fixture** to `--json` and confirm it is treated as a path.
6. Re-run **five random corpus cases** from [FIXTURE-CORPUS.md](FIXTURE-CORPUS.md) and compare raw output.

**Fail if:** any control cannot be performed (the harness does not support it — that is itself a finding against AT-08),
or a control passes for the wrong reason.

## 8. Documentation truth check

```sh
# every documented default equals the schema default (the reflection test must exist; run it)
auto-tzim config validate --json | jq '.'

# no document claims behaviour that contradicts the code
grep -rn "would optimize\|predicted saving\|tzim --dry-run" docs/ README.md
```

**Fail if:** a documented default disagrees with the schema, an exit code in the docs is absent from the executed matrix,
or a document promises a saving or an undo.

## 9. What the reviewer writes down

For each claim in the handoff, one of four marks, and no other:

| Mark | Meaning |
| --- | --- |
| **independently verified** | re-derived from raw material: recomputed aggregates, matching checksums, re-run cases |
| **source-inspected** | read in the code at the claimed SHA, not executed |
| **tool-executed** | produced by a command the reviewer ran, recorded in the review document |
| **coder-reported** | asserted, not yet checked — a finding until it becomes one of the above |

The review ends with exactly one verdict:

```text
PASS — ready for owner merge/release decision
```
or
```text
NOT READY
F-01 <severity> <issue> <what is wrong> <evidence> <what must change>
F-02 …
```

A finding without an issue id, a severity per charter §9, and a raw evidence pointer is not a finding — it is an opinion,
and it does not block anything.

## 10. Red flags that end the review early

Any one of these stops the review and the merge recommendation immediately, regardless of the remaining rows:

- a spawn during scan/dry-run carrying a path argument, or any write into a target root during a scan;
- a `tzim` exit code appearing as our exit status;
- a preflight refusal reported as `0`;
- a summary number that does not equal the recomputation from raw rows;
- an artifact whose checksum or internal SHA does not match the claimed head;
- a required job that did not actually run.
