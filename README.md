# auto-tzim

Companion orchestration tool for [`tzim`](https://github.com/RUdolf1517/tzim). It decides **when** and **which**
files `tzim` is allowed to touch, and records what happened. It contains no codecs, no verifiers and no file
replacer: every eligibility, refusal, verification and replacement decision belongs to `tzim`, and `auto-tzim`
never routes a refused file anywhere else.

> **Status: AT-01 skeleton.** Only `--help` and `--version` exist; the product below is not implemented.
> Build with the pinned Rust toolchain **1.82.0**: `cargo build --workspace`.

## What it will do (MVP / v0.1)

- You add **one narrow folder at a time**, explicitly; nothing is scanned before that.
- `scan` produces a **deterministic, read-only candidate plan**: which files our policy would hand over, with the exact
  argv. It does not invoke `tzim` on any of them and writes nothing in the folder.
- The first apply of a folder requires a matching plan (id + hash); without it the command exits `6` and does nothing.
- `run --once` then invokes the installed `tzim` as a subprocess, one file per process, argv array, no shell, no PTY.
- Every result is normalized into a documented status (`optimized`, `already_optimal`, `skipped_unsupported`,
  `skipped_by_tzim`, `skipped_policy`, `skipped_unstable`, `hardlink_skip`, `failed`, `interrupted`) and kept in local
  JSONL history. A `tzim`-decided skip keeps `tzim`'s own reason code — its note is stored verbatim, never interpreted.
- **v0.1 makes no network request at all** (D-28): there is no network code in the binary. `update-check` returns in
  v0.2 as a notification that never downloads and never replaces a binary.
- Local-only logs and local-only state; no telemetry.

Scheduler (launchd/systemd user timers), TUI, controlled Homebrew update and a Qt GUI are later stages and are
deliberately not mixed into the MVP.

## Safety rules that are not negotiable

The operative list is charter v0.2 §6 (nineteen invariants). The ones that shape everything else:

1. No folder is processed until the user adds it by name.
2. Before a folder's first apply there must be a **read-only candidate plan**: `auto-tzim` scan never runs
   file-processing `tzim`, never passes `--dry-run`, and writes nothing inside the folder. Only `tzim --version` is
   called. (`tzim`'s own dry run writes a temporary candidate into the destination directory — which is exactly why we
   do not use it.)
3. No root/sudo, no hidden background activity, no daemon.
4. No shell: filenames are argv data.
5. No symlink following; no special files.
6. Hard-linked and unstable files are skipped by default.
7. Nothing outside the canonical approved root is ever touched.
8. `auto-tzim` never implements codecs and never installs its own candidate over an original.
9. `tzim` owns eligibility, refusal, verification and installation.
10. Unknown/unsupported/refused/failed files are never re-routed to another handler.
11. `transforms` must be empty in the MVP: confirmation-required transformations do not run.
12. Scheduler/non-TTY/JSON modes never wait for a prompt and never allocate a PTY.
13. `failed` is never retried in an automatic loop.
14. v0.1 opens no socket; the v0.2 `update-check` will never download or replace a binary.
15. No telemetry and no undocumented network requests.
16. No promise of undo: a successful optimization rewrites the file.
17. The UI never duplicates core discovery/runner/updater logic.
18. Default-enabled formats and refusal reasons are never reconstructed from human output, extensions or the free-text
    `note`; an unknown contract is a fail-closed `incompatible_tzim`.
19. Exit codes are ours — `6` for a refusal before processing, `7` internal, `8` lock, `4` never produced — with a fixed
    preflight order and precedence, and no passthrough of a `tzim` exit code.

The full list, with the review gate behind each item, is in [MANAGER-SPEC.md §6](MANAGER-SPEC.md),
[docs/safety.md](docs/safety.md) and [docs/manager/GATES.md](docs/manager/GATES.md).

## Documentation

Every document below is a **specification**: it states what the program must do, and each one opens with a status
line saying so. None of them describes behaviour you can observe today.

### The contract

| Document | Contents |
| --- | --- |
| [SPEC.md](SPEC.md) | Technical specification, version 0.1 — the product contract |
| [MANAGER-SPEC.md](MANAGER-SPEC.md) | Project-manager charter: scope, gates, evidence standard, severity rules |
| [docs/spec/PROVENANCE.md](docs/spec/PROVENANCE.md) | Where the two copies came from, versions, checksums |
| [docs/spec/CONFLICTS.md](docs/spec/CONFLICTS.md) | The four places where the two specifications disagree, and which text wins |

### Product reference

| Document | Contents |
| --- | --- |
| [architecture](docs/architecture.md) | Modules, the data flow of one run, the boundary with `tzim` |
| [cli](docs/cli.md) | Commands, options, JSON output, exit codes, human output |
| [config](docs/config.md) | Every field, every default, every refusal, one complete example |
| [safety](docs/safety.md) | The seventeen invariants, the layers that enforce them, the dangerous windows |
| [guarantees](docs/guarantees.md) | What this program promises, and the four things it refuses to promise |
| [formats](docs/formats.md) | Why it supports no format of its own, and how the allowlist works |
| [state](docs/state.md) | The on-disk layout (plain JSONL, no database), locks, retention, migrations, what is *not* stored |
| [privacy](docs/privacy.md) | The complete network surface and what is kept locally |
| [update](docs/update.md) | Notify-only in v0.1; the rules a controlled update must satisfy |
| [testing](docs/testing.md) | Test layers, fixtures, negative controls, planned traceability |
| [testing harness](docs/testing-harness.md) | The fake `tzim`, the invocation log that proves a scan is read-only, the case runner, the pinned binary |
| [limitations](docs/limitations.md) | No undo, untested platforms, and everything that will surprise a user |
| [FAQ](docs/faq.md) | Short answers with links: why no undo, why exit `6`, why no capability list, can the coder start |
| [recovery](docs/recovery.md) | What to do after an interruption, a stale lock, a corrupt file |
| [roadmap](docs/roadmap.md) | v0.1 → v0.4 and what is deliberately absent |
| [releases](docs/releases.md) | Versioning, artifacts, definition of done |
| [dependencies](docs/dependencies.md) | What is depended on, and what is refused |
| [CONTRIBUTING](CONTRIBUTING.md) | What will not be accepted, and how work is organised |
| [SECURITY](SECURITY.md) | How to report a vulnerability privately, and what counts as one for a tool that moves files |
| [OWNERS](OWNERS.md) | Who decides what, and which role may write which path |
| [THIRD-PARTY](THIRD-PARTY.md) | Every third-party licence in the shipped set, for packagers |
| [audits](docs/audits/README.md) | Frozen records of what was true at a commit; first entry: the `tzim` v0.16.1 contract audit |

### Project records

| Document | Contents |
| --- | --- |
| [docs/manager/README.md](docs/manager/README.md) | Manager index and current status board |
| [docs/manager/PREFLIGHT-2026-10-29.md](docs/manager/PREFLIGHT-2026-10-29.md) | Launch preconditions, with evidence |
| [docs/manager/TZIM-STUDY-2026-10-29.md](docs/manager/TZIM-STUDY-2026-10-29.md) | What was verified about `tzim` and which gaps remain |
| [docs/manager/problems/](docs/manager/problems/README.md) | Three problems described in full: machine interface, dry-run writes, exit codes |
| [docs/manager/DECISIONS.md](docs/manager/DECISIONS.md) | Decision log, including proposals awaiting owner approval |
| [docs/manager/QUESTIONS-TO-OWNER.md](docs/manager/QUESTIONS-TO-OWNER.md) | Questions asked, and the owner's answers |
| [docs/manager/RISK-REGISTER.md](docs/manager/RISK-REGISTER.md) | Risks and the control that closes each |
| [docs/manager/ROADMAP-ISSUES.md](docs/manager/ROADMAP-ISSUES.md) | Issue breakdown per stage, with acceptance criteria |
| [docs/manager/DOC-MODEL.md](docs/manager/DOC-MODEL.md) | How this repository's documents map onto `tzim`'s, and why each difference exists |
| [docs/manager/GATES.md](docs/manager/GATES.md) | Pre-registered acceptance matrix for the MVP |
| [docs/manager/ASSIGNMENT-AT-MVP-DRAFT.md](docs/manager/ASSIGNMENT-AT-MVP-DRAFT.md) | The MVP assignment — prepared, filed as #10, not started |
| [docs/manager/HANDOFF-TEMPLATE.md](docs/manager/HANDOFF-TEMPLATE.md) · [REVIEW-TEMPLATE.md](docs/manager/REVIEW-TEMPLATE.md) | What the coder hands over, and how it is reviewed |
| [docs/manager/UPSTREAM.md](docs/manager/UPSTREAM.md) · [STATUS-LOG.md](docs/manager/STATUS-LOG.md) · [OWNER-ACTIONS.md](docs/manager/OWNER-ACTIONS.md) | Upstream state, the dated log, the commands only the owner can run |
| [CHANGELOG.md](CHANGELOG.md) | What changed; nothing has been released |

## Related project

`tzim` — <https://github.com/RUdolf1517/tzim> — is the only program that touches file bytes. `auto-tzim` changes
nothing in `tzim`: missing machine interfaces are raised as issues there, never worked around by parsing human
output.

## License

[MIT](LICENSE). Written by AI; directed by [RUdolf1517](https://github.com/RUdolf1517).
