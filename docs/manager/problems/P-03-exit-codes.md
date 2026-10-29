# P-03 · Exit codes: one missing value and one collision

Status: **analysis only** — no decision, no implementation authorised.
Sources: `SPEC.md` §8.2, `tzim` `docs/cli.md` §5 and `docs/safety.md` §5 (`source-inspected`), charter §9.

---

## 1. The problem in two sentences

`SPEC.md` §8.2 defines six exit codes for `auto-tzim`, and there is at least one run-level outcome they cannot express:
a run that **refuses to start** — the first-run gate, `pause`, or `min_free_bytes` — because `0` would claim success,
`1` claims a file failed, and `2` claims the user typed something wrong. Separately, the two programs' vocabularies
collide on the value `4`, which means *verification failure* in `tzim` and *lock unavailable* in `auto-tzim`.

## 2. The two tables, side by side

| Code | `tzim` (a file optimizer) | `auto-tzim` (an orchestrator) |
| --- | --- | --- |
| `0` | no failures; skips, refusals and `unsupported` included | command performed; `optimized`/`already_optimal`/documented skips allowed |
| `1` | at least one file failed, others processed | one or more files `failed` |
| `2` | invalid usage; nothing processed | usage/config error |
| `3` | fatal error; run could not complete (unreadable root, OOM, closed stdout) | incompatible/unavailable `tzim` |
| `4` | **verification failure**; the affected file was not replaced, run aborted | **another run holds the lock / lock unobtainable** |
| `5` | — | update check or update failure |
| `130` / `143` / `137` / `134` | signal-derived (`SIGINT`, `SIGTERM`, `SIGKILL`, abort) | not defined; signal-derived by convention |

The collision is real but manageable: `auto-tzim` must never pass a `tzim` exit code through as its own. The risk is
human, not mechanical — an operator reading a log line "exit 4" in a run that also mentions `tzim` will guess the
wrong meaning. Mitigations are documentation and log structure (always name which process produced which code), not
a code change.

## 3. Coverage: every run-level outcome against the table

| Run-level outcome | Who decides | Expresses it today? |
| --- | --- | --- |
| Everything planned ran; some files skipped/refused | us | yes — `0` |
| At least one file `failed`, the rest ran | us | yes — `1` |
| Bad command line, bad config, unknown status filter | us | yes — `2` |
| `tzim` missing, unreadable, wrong version, probe failure | us | yes — `3` |
| Another run is active, or the lock cannot be taken | us | yes — `4` |
| `update-check` failed (network, 403/429, malformed) | us | yes — `5` |
| **Run refused before processing** (first-run gate, `pause`, `min_free_bytes`, folder disabled) | us | **no** |
| **Verification failure inside `tzim`** (`tzim` exit `4`) — abort with the rest of the plan `interrupted` | `tzim` | partially: `1` says "a file failed", which is true, but the run also stopped early and a *bug-class* event was recorded; nothing says "abort" |
| **Interrupted by the user** (`SIGINT`) — in-flight file `interrupted`, rest not started | OS + us | ambiguous: we can exit `130` (honest) or `1` (says a file failed, which it did not) |
| **Internal error**, e.g. our own state DB is corrupt or an unexpected exception occurs | us | only via `1`, which misdescribes it |
| **Low disk detected mid-run** | us | `1` (fine: files genuinely failed) vs `0` (wrong) |

Three of these are gaps: the pre-flight refusal (the one raised as Q4), the internal error, and the decision about
what to return after a user interrupt. The verification abort is expressible but only by accident.

## 4. Why "just reuse 2" is not obviously wrong, and not obviously right

Arguments for reuse (`2` for everything that stops a run before it starts):

- no change to `SPEC.md`, so no contract revision and no re-review of the document set;
- `tzim` treats usage errors as `2` and nothing else, so `2` is "the caller did something the program will not do";
- a shell script checking `$? -ne 0` behaves identically either way.

Arguments against:

- `2` currently means "the user typed something wrong"; a first-run gate is not a typo — it is the program
  protecting the user from their own success. A wrapper that retries automatically on `2` (a legitimate reaction to a
  typo fix) would retry a safety stop forever, which is the shape of a retry storm the charter forbids (invariant 13
  names `failed`, but the spirit is the same).
- The same value would cover both "fix your config" and "preview this folder first", which need different user
  actions and different log messages.

Arguments for a new value (`6`):

- one row in a table, no ambiguity, and a script can distinguish "nothing to do" (`0`) from "I refused" (`6`);
- symmetrical with the existing design where `3`/`4`/`5` each name one specific operational situation;
- costs: a `SPEC.md` change (owner), a documented precedence update, and the need to explain a sixth value to users.

Arguments for `0` plus a JSON object:

- friendliest to pipelines that only look at `--json`;
- dishonest for a human reading `echo $?`, and for any script that trusts exit codes — including `systemd` units in
  v0.2, where a "refused" run reported as success would silently never run again.

## 5. Related ambiguity: what does a *successful* dry run return?

§8.2 says a dry run returns `0` when the plan was built and does not write history as if processing happened. That is
consistent, but it makes `0` cover three different things: "optimized files", "nothing to do", and "here is what I
would do". The `--json` output is the only place they are separable. This is acceptable — `tzim` does the same with
`0` and its statuses — but it is worth stating in the CLI reference so nobody "fixes" it later.

## 6. Signal-derived exits: a decision that is currently implicit

If the user presses `Ctrl-C` during `run --once`:

- `auto-tzim` has an in-flight child; it should stop it (D-10 describes `TERM` then `KILL` for timeouts, and the same
  path serves cancellation), mark the in-flight file `interrupted`, and stop planning new files;
- the process should then exit `130` (the POSIX convention, and what a shell expects), **not** `1`, because no file
  failed;
- but if a file had already failed earlier in the same run, `130` hides that from a script that only reads the code.

The honest resolution is a documented precedence — for example `130` wins over `1` when the run was interrupted,
with the JSON output still carrying the `failed` rows — and it belongs in the same review as Q4, because it is the
same table. `tzim` itself documents a precedence order (`2 → 4 → 3 → 1 → 0`); `auto-tzim` has none.

## 7. What a settled table would need to cover

1. Every row of §3, including the three that are currently inexpressible.
2. A precedence order for combinations (interrupt during a run with a prior failure; verification abort after some
   files were optimized; lock lost while a run is in flight).
3. A statement that `auto-tzim` never surfaces a child's exit code as its own, and that logs always name the process.
4. The mapping table of D-02 (tzim result → our status → run effect → our exit code) as the single place the two
   vocabularies meet.
5. Tests: each exit code produced by a real invocation, plus the combinations in item 2 (AC-8 in
   [GATES.md](../GATES.md)).

## 8. Cost of leaving it undecided

AT-06 cannot be accepted, because "exit codes are checked with real commands" is one of the audit criteria and one
value in the table does not exist yet. The coder would have to invent it, which is exactly the kind of decision the
charter keeps away from the coder — and any invention becomes a de-facto contract the moment a script depends on it.

## 9. Deliberately not done

- No change to `SPEC.md` §8.2; no new page in a CLI reference that pretends the value exists.
- No code path, no test, no fixture.
- D-03 stays in state **owner** in [DECISIONS.md](../DECISIONS.md).
