# Questions to the owner

Date: **2026-10-29**. Charter §16 asks the manager to study the charter, study `tzim`, and ask. Everything below
is a question only the owner can answer; each carries the manager's recommendation so that "yes to the
recommendation" is a valid answer. Decisions referenced by ID live in [DECISIONS.md](DECISIONS.md).

All four questions were answered on 2026-10-29, and the charter **v0.2** turned Q2–Q4 into binding decisions
([§0](#0-answers-received-2026-10-29)). Q5–Q10 are secondary and still open; each has a manager default in force, so none
of them blocks the MVP.

---

## 0. Answers received (2026-10-29)

Verbatim, per question:

| Q | Owner's answer | What the manager did |
| --- | --- | --- |
| Q1 | "жди сделай только всю документацию как в оригинальном репо tzim" — wait; produce the complete documentation, in the shape of the original `tzim` repository | No issues created, no assignment issued, no code. The full documentation set was written: product reference under `docs/`, manager records under `docs/manager/` |
| Q2 | "ничего не делай распиши проблему" — do nothing, describe the problem | [P-01](problems/P-01-machine-interface.md): the missing machine interface, described in full. No upstream issue filed, no adapter adopted |
| Q3 | "опиши подробно проблему" — describe the problem in detail | [P-02](problems/P-02-dry-run-is-not-write-free.md): a `tzim` dry run writes a temporary candidate into the destination directory |
| Q4 | "опиши подробно проблему" — describe the problem in detail | [P-03](problems/P-03-exit-codes.md): no exit code for a run refused before processing, and the collision on `4` |

**Charter v0.2 (same day)** resolved Q2–Q4 in writing, which supersedes "describe only":

| Q | Resolution |
| --- | --- |
| Q2 (machine interface) | accepted as a limitation, not a blocker: restricted adapter, `tzim --version` only, default set delegated without `--formats`, `transforms` empty, generic `skipped_by_tzim`, unknown contract ⇒ `incompatible_tzim`. One upstream issue filed (→ `tzim#86`). Decisions D-01, D-02, D-05 |
| Q3 (what `run --dry-run` does) | scan and dry-run are one thing: a **local read-only candidate planner**. `tzim --dry-run` is forbidden in the MVP, no target path is passed, nothing is written in the target root; plan reports candidates, not savings. Apply needs a plan id/hash. Decisions D-04, D-20, D-23 |
| Q4 (exit codes) | `6` preflight/policy refusal, `7` internal, `8` lock, `4` unused, `130/143` signals, precedence `130/143 > 7 > 2 > 8 > 3 > 6 > 5 > 1 > 0`, fixed preflight order, no raw `tzim` passthrough. Decisions D-18, D-19 |

Consequences of Q1 = wait, recorded so they are not re-litigated later:

- **No GitHub issues exist** for AT-01…AT-09; the internal IDs stay internal
  ([ROADMAP-ISSUES.md](ROADMAP-ISSUES.md) is the source).
- **No implementation is authorised**; pre-flight P1 now reads "answered: wait", not "unverified", and P2 (the
  unfinished `tzim` set) stays open.
- **Q2–Q4 are decided** by charter v0.2; the three problem notes stay as the analysis behind D-02, D-04 and D-18/D-19.
- **Documentation is now complete** for the MVP contract: a reader can review what the program must do, what it
  refuses to do, and how it will be checked, without reading code — because there is none.

---

## Q1 — Authorisation and the state of the previous `tzim` work · **met: charter v0.2 authorises the management work**

Charter §3.1 requires explicit permission to start, and §3.2 requires that the coder has finished the previously
assigned `tzim` set. The second condition is factually unmet: **PR #84 is open with two failing measurement checks**
and seven issues remain open (#57, #59, #78, #79, #80, #82, #83) — see
[pre-flight P2](PREFLIGHT-2026-10-29.md).

Options:

- **A (recommended if `tzim` work is not urgent): wait.** `auto-tzim` stays documentation-only until #84 is merged
  or explicitly parked. Nothing is lost; the pre-flight, study, gates and assignment draft are already in place.
- **B: start the MVP now, in parallel, and accept that one coder may be split across two projects.** Requires the
  owner to say that #84/#78 are parked or finished enough. The manager will then create AT-01…AT-09 and issue the
  assignment.
- **C: start with a narrower first slice** (AT-01 + AT-02 only: package bootstrap, config, atomic storage) so that
  the `tzim` side finishes first.

What it blocks: creating issues, issuing the assignment. What it does not block: everything already in this
repository.

## Q2 — How to close the missing machine interface · **decided in charter v0.2 (§16 P-01)**

**Outcome:** option **A**, with the adapter restricted further than this question proposed — the upstream issues were
filed as one (`tzim#86`), no machine-readable capability is assumed, and the default set is delegated rather than
displayed. The options below are kept as the record of what was weighed.

`tzim` has no machine-readable capability list and no structured skip reason ([study G-1, G-2](TZIM-STUDY-2026-10-29.md)).
The specification already answers what to do technically — a version-specific adapter with fail-closed behaviour,
plus an issue in `tzim` for the missing machine interface — but the owner decides whether we file those issues now
and whether the adapter may rely on a runtime probe.

Options:

- **A (recommended):** adapter-only for the MVP (D-01, D-02), *plus* two upstream issues in `tzim` —
  `--capabilities --json` and a stable skip/refusal reason code. Neither is a blocker; both are improvements the
  adapter stops needing later.
- **B:** file the upstream issues and **wait** for them before AT-04, so the MVP never runs on an adapter.
  This delays the MVP by at least one `tzim` release cycle.
- **C:** adapter-only, no upstream issues. Cheapest now, but the gaps stay invisible to whoever maintains the
  pair of projects later.

## Q3 — What exactly does `run --once --dry-run` do? · **decided in charter v0.2 (§16 P-02)**

**Outcome:** none of the three options as written. The owner removed the choice by forbidding `tzim --dry-run` in the
MVP: scan and dry-run became one **read-only candidate planner** that reports candidates and no numbers, and an apply
requires a matching plan. Options A/B/C are kept below as the analysis that led there.

`tzim`'s own dry run is not free: it builds and verifies a real candidate in the destination directory and then
deletes it ([study B-1](TZIM-STUDY-2026-10-29.md)). So there are two honest meanings:

- **A (recommended, D-04):** `scan` = our plan only, no `tzim` process; `run --once --dry-run` = plan **plus**
  `tzim -n --json` per file, giving exact predicted savings. Documentation states the temporary-file fact.
- **B:** both commands are plan-only, and `tzim` is never invoked in dry-run mode. Cheaper and completely
  write-free, but "how much would I save?" becomes an estimate, and the first real run is the first real
  measurement.
- **C:** `run --once --dry-run` is a plan, and a separate `--probe` flag asks `tzim` for numbers. More surface than
  the MVP needs.

Sub-question: does **either** variant satisfy the §8.1 first-run gate, or does the owner want the gate to require
`scan` specifically? The manager recommends "either", recorded in history as a dry run.

## Q4 — The missing exit code for a refused run · **decided in charter v0.2 (§16 P-03)**

**Outcome:** option **A** was adopted and extended — `6` for preflight refusals, plus `7`, `8`, `130`/`143`, `4` unused,
a fixed preflight order and the precedence `130/143 > 7 > 2 > 8 > 3 > 6 > 5 > 1 > 0`. See D-03, D-18, D-19.

The first-run gate, `pause`, and `min_free_bytes` all refuse a run before anything is touched, and §8.2 has no code
for that: `0` would claim success, `1` claims a file failed, `2` claims a usage/config error, `3`/`4`/`5` mean other
things ([D-03](DECISIONS.md#d-03--pre-flight-refusals-need-an-exit-code--owner)).

Options:

- **A (recommended):** add **exit code `6` — refused before processing; nothing was read for processing and
  nothing changed**, and update §8.2. One extra row, and scripts can distinguish "nothing to do" from "I refused".
- **B:** reuse `2` for all pre-flight refusals, and document that `2` covers both usage errors and refusals.
  No spec change, but a caller cannot tell a typo from a deliberate safety stop.
- **C:** return `0` with a documented `refused` object in `--json`. Kind to pipelines, dishonest for humans and
  for scripts that only look at the code.

---

## Secondary questions

### Q5 — May v0.1 ship with an empty `transforms` allowlist? (D-05)

Recommended: **yes.** Invariant 11 needs the mechanism; the only candidate transformation (`fits-tile`) is absent
from the released binaries, so enabling it would be untestable against what users actually install. The field
exists, validates, and rejects every value in v0.1.

### Q6 — Is the GitHub releases API an acceptable update source? (D-07)

Recommended: **yes**, one request, one command, documented in privacy as the product's only network call. If the
owner prefers a different documented source (for example the Homebrew tap's formula), say which; the manager will
pre-register that instead.

### Q7 — Hashing: cap at 64 MiB, or everywhere, or never? (D-09)

Recommended: **rewritten files only, up to 64 MiB, `null` recorded above the cap.** "Everywhere" doubles I/O;
"never" loses the strongest post-hoc evidence we could keep.

### Q8 — Test matrix: which macOS, which Linux?

Recommended: `ubuntu-latest` (x86_64) + `macos-latest` (arm64), Python 3.12 and 3.13, plus one job per stage on
3.12 only. Intel macOS and ARM Linux are documented as untested until someone asks for them. Windows is out of the
MVP (spec §3 is macOS/Linux).

### Q9 — Where is the canonical copy of the specification?

Both specification files exist only on the branch `arena/b3d4085f-tzim` in `tzim` and are not on `main`
([provenance](../spec/PROVENANCE.md)). Recommended: merge that docs-only commit into `tzim` `main` before the first
`auto-tzim` implementation PR, so "specification version 0.1" resolves identically in both repositories. This is
the owner's action, not the manager's.

### Q10 — Who is the coder, and who merges?

Assumption to confirm: a single coder agent works in this repository on its own branch and opens **one** MVP pull
request with issue-numbered commits; the manager reviews and recommends; **the owner merges**. The manager does not
merge, close issues or publish releases without explicit delegation (charter §2.2, §14).

---

## Assumptions the manager will act on unless corrected

1. **A1** — MVP supports macOS and Linux only; CPython 3.12+; no Windows claim in any document.
2. **A2** — one implementation PR for the MVP, commits labelled by issue ID; later stages get their own PRs.
3. **A3** — the manager writes documentation, issues, gates and reviews; never product code, tests or CI.
4. **A4** — CI runs on GitHub-hosted runners; no self-hosted infrastructure is introduced.
5. **A5** — the repository description, topics and any branch protection are set by the owner when asked once.
6. **A6** — `tzim` stays untouched; its gaps become issues there, not patches here.

## What happens after the answers (state on 2026-10-29)

| Item | State |
| --- | --- |
| Q1 | charter v0.2 authorises the management deliverable; **implementation still waits** on precondition §3.2 (B-01) |
| Q2–Q4 | decided; recorded as D-01…D-05, D-18, D-19, D-20, D-23 |
| Q5 (`transforms` empty) | decided for the MVP by charter §6-11 (D-05); the field returns when a machine-readable contract exists |
| Q6 (update source) | manager default in force (D-07): one GitHub releases API request; owner may override at no cost |
| Q7 (hash cap) | manager default in force (D-09); `null` above 64 MiB, recorded explicitly |
| Q8 (test matrix) | manager default in force: `ubuntu-latest` + `macos-latest`, Python 3.12 and 3.13 |
| Q9 (canonical spec home) | open, owner action: the docs-only commit `efc31756` is still only on a branch of `tzim` — command in [OWNER-ACTIONS.md](OWNER-ACTIONS.md) §4 |
| Q10 (who merges, who is the coder) | default in force: one coder, one MVP pull request, owner merges |

The next decision point is not a question but a precondition: B-01 must be cleared before #10 may be started.
