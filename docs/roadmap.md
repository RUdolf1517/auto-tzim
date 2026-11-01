# Roadmap

> **Status: specification, no implementation.** No version has shipped. v0.1 is defined in detail and blocked on nothing
> but the work itself; v0.2 and later are commitments of *direction*, not of scope — each one gets its own issue set and
> its own acceptance matrix before a line of it is written.
> Related: [releases](releases.md) · [cli](cli.md) · [update](update.md) · [manager roadmap](manager/ROADMAP-ISSUES.md)

The order is chosen by **risk**, not by user demand: first prove that nothing can escape the plan, then let it run without
a person watching, then let it touch the binary it depends on, and only then put a GUI in front of it. Every version adds
exactly one new way to do damage, and that way is the thing tested before the version ships.

## 1. The versions at a glance

| Version | One sentence | New risk it introduces | Gate |
| --- | --- | --- | --- |
| **v0.1** | A person points it at one folder, reads a plan, approves it, and gets smaller files | none yet — nothing runs without a person | 17 acceptance rows, 23 invariants at one final SHA, macOS + Linux |
| **v0.2** | It runs on a timer, a TUI shows what it did, and it tells you when `tzim` has a new version | it acts while nobody is watching | no overlapping runs, forced-schedule test on both platforms, no socket outside one documented request |
| **v0.3** | It can update `tzim` itself, with the lock held and the result verified | it replaces the binary its own safety depends on | a real in-place upgrade from the previous version, verified by version, hash and a smoke run |
| **v0.4** | A Qt window over the same core | a second front door that can drift from the first | the GUI contains no discovery, subprocess or updater code |
| **v1.0** | The contract stops moving | users' automation starts depending on names we can no longer change | two consecutive minors with no contract change, no open Medium finding |

A patch release (`0.x.y`) only fixes defects and never changes a documented status, exit code, default or field. Under
[releases §1](releases.md) a change to the contract is a *major* change even while the major number is `0` — the number is
not the promise, the document is.

## 2. v0.1 — MVP: prove the plan

**A user gets:** one folder added to a config, a read-only candidate plan, an approval step, and files that are smaller
afterwards — with every outcome recorded and every non-candidate file proven byte-identical.

**In scope.**

- Core, CLI, config v1, SQLite state and history, locks, recovery, the full exit-code contract.
- The read-only candidate plan and plan-bound apply (D-20): the plan is a hash over the ordered files; an apply without a
  matching one refuses with exit `6`.
- **Two modes and one install path** (D-29): `confirm` shows a proposal and installs only on approval; `auto` does not ask.
  Work happens on a staged copy, the swap is journaled, and an original is deleted only after its verified replacement is
  in place.
- The adapter reads `tzim --capabilities` and fails closed on a schema pair it was not tested against (D-26); refusals carry
  `tzim`'s stable `reason` code beside its verbatim note (D-27); accepted `tzim` versions are `>=0.16.0, <0.17` (D-30).
- **No network, at all** (D-28). Not on start-up, not on a timer, not in a dependency.

**Out of scope, deliberately:** the scheduler, the TUI, the GUI, any update mechanism, `--jobs`, `--force`,
`--strip-metadata`, `--fits-tile`, backup/undo, cloud sync, Windows, and any change inside the `tzim` repository.

**Gate.** Every row of [manager/GATES.md](manager/GATES.md) — 17 acceptance rows, 23 invariants — demonstrated at one
final head SHA on macOS and Linux, with raw evidence a reviewer can re-run: the invocation log proving the folder was never
written before install, a `SIGKILL` at every step of the install journal, the same corpus run in both modes, and every
documented exit code produced by a real command. No unresolved Critical or High finding.

**What would stop it:** a data-safety defect in the install path; a `tzim` contract change that invalidates the accepted
window; or evidence that the staged copy cannot be made crash-safe on a supported filesystem.

## 3. v0.2 — unattended: scheduler, TUI, and the one request

**A user gets:** a folder that is optimized on a schedule without being asked, a terminal interface that shows the plan,
the argv and the history, and a notification when `tzim` itself has a newer release.

**In scope.**

- **Scheduler**: launchd user agent on macOS, systemd user timer on Linux. No system daemon, no root, schedule off by
  default. `pause`/`resume`/disable/remove genuinely change OS state. A missed run does not cause a catch-up storm: the
  timer fires `run --once` and nothing else, so `run --once` keeps exactly one implementation and the scheduler has no
  eligibility rules of its own.
- **TUI** (Textual, optional extra): folder and profile editing, the plan with its exact argv, current file and counters,
  history, pause/resume, and the update notification. It reads and writes through the core API only. Dangerous changes —
  `transforms`, cloud sync, a scheduled update — need a separate confirmation screen whose default is **No**.
- **`update-check` returns** (D-28 deferred it here): one documented `GET`, notify only, 24 h cache, exit `5` on failure,
  and the privacy page updated *before* the code merges. It never downloads, verifies or replaces anything.
- `--jobs` may leave the config and become a real flag **only** if per-file isolation is proven under concurrency; until
  then it stays rejected.

**Out of scope:** anything that modifies `tzim`'s binary; Qt; Windows; battery or network-aware policies (allowed later,
never changing safety semantics).

**Gate.** No overlapping runs under a forced schedule, proven by a test that compresses time rather than by a manual run;
documented and tested behaviour at logout and with no user session on both platforms; the TUI cannot bypass the approval
gate or the plan binding; a socket opened anywhere except the single documented `update-check` fails the suite; every
scheduler state change verified against the real OS (`launchctl print`, `systemctl --user show`).

**What would stop it:** a platform where a user timer cannot be paused reliably — then that platform ships without the
scheduler rather than with a scheduler that lies about being paused.

## 4. v0.3 — controlled update: let it touch its own dependency

**A user gets:** an upgrade of `tzim` performed by the tool, with the lock held, the old identity recorded and the new one
verified.

**In scope.**

- Update only through an explicitly configured official Homebrew tap/formula; the name is never guessed.
- Before: no active run, the global lock held, the old version and binary identity recorded.
- After: version and capabilities re-checked, the binary hash recorded, and a read-only smoke run before anything is
  processed. A failed update leaves a clear log and runs no compression.
- `update_policy = "scheduled"` becomes available only after one successful manual update, as a separate opt-in.
- `auto-tzim` **never** updates itself through this mechanism.

**Out of scope:** automatic rollback. It is not claimed until it exists and has been tested; until then an update that
fails leaves a working old binary and a log.

**Gate.** An isolated environment with the previous release installed, upgraded the same way a user would be, verified by
version, hash, adapter probe and a smoke run (charter §11.3). Checking the formula on GitHub is not evidence.

**What would stop it:** no reliable way to detect that a package manager changed the binary under a running process —
then the feature ships as "detect and tell the user" instead of "perform".

## 5. v0.4 — Qt GUI: a second front door

**A user gets:** native folder pickers, a preview of the plan, status and history tables, and pause/scheduler controls.

**In scope.** PySide6, a thin wrapper over the same core. Packaging, signing, notarisation and sandbox entitlements are
documented before the release, not after. The GUI does not start itself after installation and needs no administrator
rights.

**Out of scope.** Any file discovery, subprocess handling, adapter logic or updater code outside `core/`. A GUI that
duplicates core logic is the failure mode this version is gated against.

**Gate.** Every user-visible action in the GUI traced to a core call; the same corpus outcomes through the GUI and the CLI;
no core behaviour reachable only from the GUI.

## 6. v1.0 — the contract stops moving

Not a feature release. It is the point where the documented statuses, exit codes, config fields and JSON keys become a
promise to strangers, and where a change to any of them needs a major version.

**What would earn it:** two consecutive minor releases with no contract change, a release with no open Medium finding, and
a real installed base whose reports we have answered. Until those are true, `1.0` is a number, not a promise — and the
first thing a premature `1.0` buys is the right to break someone else's automation.

## 7. What has no version number

Not "not yet" — **not planned**, and each of these is refused at the design level, not deferred:

Own codecs or repackers; a `--force` flag; an undo or backup store (see [limitations](limitations.md) §1 for why the
honest version of that promise is not the one people expect); cloud sync; self-update of `auto-tzim`; a background agent
with a menu-bar icon; a container image; any change inside the `tzim` repository; Windows before the safety invariants are
proven on it.

## 8. What could move the order

- **A reported data-safety defect** stops everything else until it is closed. Nothing outranks it.
- **A `tzim` contract change** — a new capabilities schema, a removed flag — can pull a compatibility release forward, even
  as a patch, if it would otherwise make a shipped version refuse to work.
- **`transforms`**: `--strip-metadata` and `--fits-tile` stay out until a real-binary test proves preservation for each.
  A FITS-enabled `tzim` build makes that testable, and it could arrive before v0.2 on its own merits.
- ~~A `tzim` capability API~~ — **it shipped** in v0.16.0 and v0.1 consumes it (D-26). The question it left open is
  whether to *display* the default set, which is refused for now ([audit DR-05](audits/2026-11-01-tzim-v0.16.1.md)).
- **Windows** is not a version; it is a rewrite of the install path's assumptions and lands only when someone can test it.
