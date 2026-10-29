# Roadmap

> **Status: specification, no implementation.** v0.1 has not started; stages after it are sketches and may change.
> Related: [releases](releases.md) · [manager roadmap and issues](manager/ROADMAP-ISSUES.md) · [update](update.md)

The order is chosen by risk, not by user demand: first prove that nothing can escape the plan, then make it run on a
timer, then let it touch the binary it depends on, and only then put a GUI in front of it.

## v0.1 — MVP

Core, CLI, `run --once`, the **read-only candidate plan** and plan-bound apply, config/state/history, `tzim` version
check and an update **notification**. `tzim --dry-run` is used nowhere: a scan is our own read-only planner, precisely
because `tzim`'s dry run writes a temporary candidate into the destination directory. Acceptance
is defined row by row in [manager/GATES.md](manager/GATES.md): deterministic plans, first-run gate, no shell, no
prompts, no retry loop, crash recovery, documented exit codes, notify-only updates, and a limitations list that a user
can act on.

Gate: all of it on macOS and Linux at one final head SHA, with raw evidence, and no unresolved Critical/High finding.

## v0.2 — scheduler and TUI

- launchd user agent (macOS) and systemd user timer (Linux); no system service, no root.
- One implementation of `run --once`; the scheduler has no eligibility rules of its own.
- `pause`/`resume`/disable actually change OS state; a missed run does not cause a catch-up storm.
- Both platforms document what happens at logout and with no user session.
- A TUI that shows the plan, the exact argv, the run, and the history — reading and writing only through the core API.

Gate: no overlapping runs under a forced schedule; the TUI cannot bypass a confirmation; scheduler tests on both
platforms.

## v0.3 — controlled update

Update through an explicitly configured official Homebrew tap/formula, with the lock held for the whole operation,
binary identity recorded before and after, and a real isolated upgrade test from the previous installed version. No
automatic rollback claim until rollback exists. See [update](update.md) §2.

Gate: a real in-place upgrade, verified by version, hash, adapter probe and a dry-run smoke test.

## v0.4 — Qt GUI

PySide6, a thin wrapper over the same core: folder pickers, dry-run preview, status/history tables, pause/resume and
scheduler controls, update notifications. No discovery, subprocess or updater code outside `core/`. Packaging, signing,
notarisation and sandbox entitlements are documented before a release, not after.

## Deliberately not on the roadmap

Own codecs or repackers; a `--force` flag; an undo/backup store; cloud sync; self-update of `auto-tzim`; a background
agent with a menu-bar icon; Windows support in the MVP; any change inside the `tzim` repository.

## What could move the order

- A `tzim` capability API would simplify v0.1's adapter and might pull the "exact numbers" question ([P-02](manager/problems/P-02-dry-run-is-not-write-free.md)) earlier.
- A FITS-enabled release build would make the `transforms` allowlist testable and could arrive before v0.2.
- A reported data-safety defect stops everything else until it is closed.
