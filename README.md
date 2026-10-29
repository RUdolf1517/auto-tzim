# auto-tzim

Companion orchestration tool for [`tzim`](https://github.com/RUdolf1517/tzim). It decides **when** and **which**
files `tzim` is allowed to touch, and records what happened. It contains no codecs, no verifiers and no file
replacer: every eligibility, refusal, verification and replacement decision belongs to `tzim`, and `auto-tzim`
never routes a refused file anywhere else.

> **Status: pre-MVP — documentation only.**
> There is no package, no CLI, no release and no CI in this repository yet. Nothing here is installable or
> usable, and no implementation has been assigned (see
> [pre-flight](docs/manager/PREFLIGHT-2026-10-29.md)). This README describes the *contracted* product, not an
> existing one.

## What it will do (MVP / v0.1)

- You add **one narrow folder at a time**, explicitly; nothing is scanned before that.
- The **first run of a new folder is dry-run only**.
- `scan` shows a deterministic plan; `run --once` invokes the installed `tzim` as a subprocess, one file per
  process, with an argv array and no shell.
- Every result is normalized into a documented status (`optimized`, `already_optimal`, `skipped_refused`,
  `skipped_unstable`, `hardlink_skip`, `failed`, `interrupted`, …) and kept in local SQLite history.
- `update-check` only **notifies** about a newer `tzim`; nothing downloads or replaces a binary in v0.1.
- Local-only logs, no telemetry, no network access except an explicitly invoked `update-check`.

Scheduler (launchd/systemd user timers), TUI, controlled Homebrew update and a Qt GUI are later stages and are
deliberately not mixed into the MVP.

## Safety rules that are not negotiable

1. No folder is processed until the user adds it by name.
2. First run of a new folder is dry-run only.
3. No root/sudo, no hidden background activity, no daemon.
4. No shell: filenames are argv data.
5. No symlink following; no special files.
6. Hard-linked and unstable files are skipped by default.
7. Nothing outside the canonical approved root is ever touched.
8. `auto-tzim` never implements codecs and never installs its own candidate over an original.
9. `tzim` owns eligibility, refusal, verification and installation.
10. Unknown/unsupported/refused/failed files are never re-routed to another handler.
11. Confirmation-required transformations need a separate, explicit opt-in.
12. Scheduler/non-TTY/JSON modes never wait for a prompt.
13. `failed` is never retried in an automatic loop.
14. `update-check` never downloads or replaces a binary in the MVP.
15. No telemetry and no undocumented network requests.
16. No promise of undo: a successful optimization rewrites the file.
17. The UI never duplicates core discovery/runner/updater logic.

The full list, with the manager's review gates behind each item, is in
[MANAGER-SPEC.md §6](MANAGER-SPEC.md) and [docs/manager/GATES.md](docs/manager/GATES.md).

## Documentation

| Document | Contents |
| --- | --- |
| [SPEC.md](SPEC.md) | Technical specification, version 0.1 — the product contract |
| [MANAGER-SPEC.md](MANAGER-SPEC.md) | Project-manager charter: scope, gates, evidence standard, severity rules |
| [docs/spec/PROVENANCE.md](docs/spec/PROVENANCE.md) | Where the two copies came from, commit SHAs, checksums |
| [docs/manager/README.md](docs/manager/README.md) | Manager index and current status board |
| [docs/manager/PREFLIGHT-2026-10-29.md](docs/manager/PREFLIGHT-2026-10-29.md) | Launch preconditions, with evidence |
| [docs/manager/TZIM-STUDY-2026-10-29.md](docs/manager/TZIM-STUDY-2026-10-29.md) | What was verified about `tzim` and which machine-interface gaps remain |
| [docs/manager/DECISIONS.md](docs/manager/DECISIONS.md) | Decision log, including proposals awaiting owner approval |
| [docs/manager/QUESTIONS-TO-OWNER.md](docs/manager/QUESTIONS-TO-OWNER.md) | Open questions, blocking and secondary |
| [docs/manager/RISK-REGISTER.md](docs/manager/RISK-REGISTER.md) | Risks and the control that closes each |
| [docs/manager/ROADMAP-ISSUES.md](docs/manager/ROADMAP-ISSUES.md) | Issue breakdown per stage, with acceptance criteria |
| [docs/manager/GATES.md](docs/manager/GATES.md) | Pre-registered acceptance matrix for the MVP |

## Related project

`tzim` — <https://github.com/RUdolf1517/tzim> — is the only program that touches file bytes. `auto-tzim` changes
nothing in `tzim`: missing machine interfaces are raised as issues there, never worked around by parsing human
output.

## License

[MIT](LICENSE). Written by AI; directed by [RUdolf1517](https://github.com/RUdolf1517).
