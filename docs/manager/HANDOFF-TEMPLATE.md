# Handoff template (charter §12)

Copy this file, fill every line, and paste it into the MVP pull request. A line that is not filled is a finding: a
handoff with "CI: green" and no run ids cannot be verified, and an unverifiable handoff is `NOT READY`.

> The template is intentionally rigid. The reviewer's commands in
> [EVIDENCE-PROTOCOL](EVIDENCE-PROTOCOL.md) read specific fields; prose here becomes work there.

---

```text
Final head SHA:            <40-hex, the only SHA that counts>
Branch:                    <name>
Pull request:              <#number>
Issues and commits:
  AT-01  #1   <sha>  <one-line subject>
  AT-02  #2   <sha>  <one-line subject>
  …
Implemented behavior:      <3–8 lines: what a user can now do, in the order of the issues>
Non-goals preserved:       <the charter §6 items that could plausibly have been touched, each with the reason it was not>
CI runs and artifacts:
  <run-id>  <workflow>  <platform>  <python>  <status>  <head_sha>
  …
Raw evidence locations:    <paths inside evidence/<head-sha>/, or URLs to the uploaded bundle>
Independent commands/results:
  <command>  ->  <exit code>  <one-line result>      (at least: doctor, scan, run --once, the exit-code matrix)
Tests added and negative controls:
  <test file>  <what it asserts>  <the control that proves the assertion can fail>
Docs updated:              <file> — <what changed>   (one line per file)
Known limitations:         <what is still not true, in the words the README will use>
Remaining blocker:         none
                           | Blocker ID / Affected issue / Exact operation / Tool+version /
                             Observed error+evidence / What was tried / Why not fixable here /
                             Single owner action / Work continuing meanwhile
Merge/Release performed by coder: no
```

---

## Artifact bundle the handoff must point at

Produced by the harness ([testing-harness](../testing-harness.md) §6), not assembled by hand:

```text
evidence/<head-sha>/
  manifest.json     # head sha, platform, python, both binary identities
  SHA256SUMS        # one line per file, generated last
  summary.json      # counts per status — the reviewer recomputes these from the case files
  <case-id>/        # one directory per corpus case, 111 of them at freeze time
    stdout.txt  stderr.txt  exit.txt
    invocations.jsonl
    root-before.txt  root-after.txt
```

A bundle without `SHA256SUMS`, or one whose sums do not match, is not evidence.

## The claims table (fill the mark yourself, honestly)

| Claim | Mark | Where the proof is |
| --- | --- | --- |
| scan invokes no file-processing `tzim` | coder-reported / tool-executed | `X-…/invocations.jsonl` |
| target root unchanged by a scan | | `root-before.txt` vs `root-after.txt` |
| every non-candidate file byte-identical | | per-case before/after hashes |
| exit `4` never produced | | the matrix run |
| no PTY, no stdin read | | `R-02`, `R-01` rows |
| second run makes no invocation | | `R-14` row |
| docs match the schema | | the reflection test output |

The reviewer will overwrite every mark with **independently verified**, **source-inspected**, **tool-executed** or leave
it **coder-reported**. An unchanged mark at the end of the review is, by definition, an unverified claim.

## Three things that get a handoff returned unread

1. A summary number that cannot be recomputed from the case files.
2. A CI run whose `head_sha` is not the final head (an earlier green run does not transfer).
3. A "known limitations" section that omits something the repository already documents — for example that an apply lets
   `tzim` write a temporary candidate into the folder, or that `transforms` is empty.
