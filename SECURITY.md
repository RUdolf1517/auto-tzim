# Security Policy

This document explains how to report a security vulnerability in **auto-tzim**. It does not replace the guarantees and
safety documents linked below, and it is not a claim that the software is free of vulnerabilities.

**Status: nothing is implemented yet.** There is no package, no binary and no release, so nothing can currently be
exploited. The policy exists before the code for the same reason the test corpus does: so that the first release does not
have to invent it afterwards, under pressure, while a report is open.

## Supported versions

| | |
| --- | --- |
| Released versions | **none** — there is no v0.1 yet |
| Investigated now | the `main` branch |
| After the first release | the latest published release and `main`; fixes ship in a new release, and backports are not promised |

Include the commit you tested (or, later, the release version) so the affected code can be identified unambiguously.

## Report a vulnerability

**Do not disclose an unpatched vulnerability in a public issue, pull request or discussion.**

1. **Preferred:** open the [Security tab](https://github.com/RUdolf1517/auto-tzim/security) and choose
   **Report a vulnerability**. That opens a private report visible only to the maintainer.
2. **Fallback:** email [rudolfzinovev@protonmail.com](mailto:rudolfzinovev@protonmail.com).

One channel is enough — please do not send the same report twice. Do not attach personal files or data that is not
needed to reproduce the problem; a minimal, synthetic reproducer is better than a real library of photos.

## Scope

`auto-tzim` is a small orchestrator that sits next to **files a person cares about**. It scans a folder without writing to
it, hands one file at a time to an external `tzim` binary, checks the result, and installs the optimized file in a second
folder. That shape defines what counts as a security report.

Reports about any documented MVP behaviour are welcome. The classes that matter most:

- **A write to the source folder during a scan.** The read-only promise is the product's central invariant; any write,
  create, rename, permission change or `atime`-mangling side effect there is a security defect, not a bug.
- **Loss, corruption or misplacement of a file** — a half-copied file left behind, two files mapped to one destination,
  an original replaced rather than copied, a result installed for the wrong input, an interrupted run that cannot be
  told apart from a finished one.
- **Writes outside the configured destination root**, including path traversal through a filename, through a string the
  `tzim` output supplied, or through a symlink planted inside the destination folder.
- **Argument or command injection through a filename** — a file named `-rf`, `--json`, `$(…)`, or containing newlines,
  invalid UTF-8 or a leading dash; any code path that builds a shell command instead of an argument vector.
- **Time-of-check/time-of-use**: a file swapped between the moment it was planned and the moment it was copied, a
  symlink retargeted mid-run, a hard link that appears after the link-count check.
- **Trust in the external binary**: `PATH` hijacking, a binary replaced between hash verification and execution, a
  pinned-version check that can be satisfied by a different build, or any execution of a download that was not verified
  against a published checksum.
- **Unexpected network activity.** Exactly one outbound request is documented (update check). Any other connection, any
  downgrade or redirection that turns it into something else, and any attempt to execute downloaded content is a defect.
- **State and logs**: injection into a state file or a history row, log records that disclose file paths or content to someone who
  reads them, or a history that misrepresents what happened to a file.
- **Resource exhaustion** that is disproportionate to the documented limits, and hangs that the per-file timeout does not
  end — including a child process that is waiting for a prompt that will never come.
- **Privilege boundaries**: behaviour when run as root, ownership or permission changes on user files, and anything that
  writes outside the project's own directories.

### Not a vulnerability

- A documented limitation, including **no undo**: the mitigation is a warning before the first apply, and it is written
  in [limitations](docs/limitations.md), not discovered.
- A `tzim` format bug, a `tzim` refusal, or a surprising `tzim` result. Those belong upstream in
  [RUdolf1517/tzim](https://github.com/RUdolf1517/tzim/issues) — our contract is that we never hide what `tzim` reported.
- "It saved nothing", "it skipped my file", "the format is not supported" — expected outcomes with documented reasons.
- An unsupported platform, or a self-built, unverified copy of `tzim` that we deliberately refused to run.

A documented safeguard that can be *bypassed*, however, is a vulnerability. The distinction is the safeguard, not the
document.

## What to include

As much of the following as applies:

- commit (or release) of `auto-tzim`, operating system, architecture, and how it was installed;
- the exact command, configuration file and folder layout, with paths shortened;
- the version and SHA-256 of the `tzim` binary in use, if the report involves it;
- what you expected, what happened, and **what was lost or changed on disk** — that last part is the one that decides
  severity;
- verbatim output or logs, with personal values removed;
- a minimal reproducer: the smallest directory tree and the fewest commands that show the problem.

**Do not run a suspected destructive reproducer against data you care about.** Use a disposable directory. If you think a
bug eats files, copy a synthetic tree, not a real one.

## Coordinated disclosure

Good-faith research is welcome. Test only files and systems you own or are authorized to test, and do not use more
exploitation than is needed to demonstrate the issue.

Reports are handled through validation, a fix, and disclosure agreed with the reporter. This policy promises a
coordinated process, not a fixed response or fix time. Please keep the details and any proof-of-concept material private
until disclosure is agreed.

## How a fix is handled here

The project is run under a written charter ([MANAGER-SPEC.md](MANAGER-SPEC.md)): the coder writes the fix and the test,
the manager verifies it independently and records the evidence, and the owner authorizes the merge and the release. A
security fix does not get a shortcut through those gates — a panic merge is how a second defect gets shipped. If a
report requires relaxing a safety invariant to fix quickly, the owner is asked explicitly, in writing, and the answer is
recorded in [DECISIONS](docs/manager/DECISIONS.md).

## Related documentation

- [Safety](docs/safety.md) — the read-only scan, the copy and verify sequence, and every failure mode we can name.
- [Guarantees](docs/guarantees.md) — what is promised about each file and what is explicitly not.
- [Privacy](docs/privacy.md) — what leaves the machine — in v0.1, nothing.
- [Limitations](docs/limitations.md) — the things that look like bugs and are not.
- [Dependencies](docs/dependencies.md) · [Third-party notices](THIRD-PARTY.md) — what is linked and what is invoked.
- [Releases](docs/releases.md) — artifacts, checksums, and verification before execution.
