# Third-party notices

What this project would ship, and under what licence. The reasoning — why each entry exists and what it replaces — is in
[dependencies](docs/dependencies.md); this page is the flat list a distribution packager needs.

**Status: nothing is bundled yet.** The table is the intended set, fixed before the code so that the first release does
not add a dependency quietly. Exact versions and hashes are pinned when the package is created (AT-01/AT-02).

## Runtime dependencies

| Package | Used for | Licence | Ships a `NOTICE`? |
| --- | --- | --- | --- |
| [`tomli-w`](https://pypi.org/project/tomli-w/) | writing the configuration file; `tomllib` in the standard library can only read | MIT | no |
| Python standard library (`tomllib`, `sqlite3`, `urllib.request`, `subprocess`, `signal`, `hashlib`, …) | everything else | [PSF-2.0](https://docs.python.org/3/license.html) | no |

That is the entire runtime set: one third-party package. No codec, no image library, no archive library, no HTTP client,
no shell wrapper — those are `tzim`'s job, and it is not linked.

## External program, not a dependency

`auto-tzim` **runs** [`tzim`](https://github.com/RUdolf1517/tzim); it does not import, vendor or link it. `tzim` is a
separate MIT-licensed executable installed by the user, with its own
[THIRD-PARTY.md](https://github.com/RUdolf1517/tzim/blob/main/THIRD-PARTY.md) covering its crates. A user who installs
`auto-tzim` acquires nothing from that tree; a user who already has `tzim` keeps it where it is.

This is deliberate and it is the licence boundary: because the two programs meet at a process boundary, neither
project's dependency tree becomes the other's.

## Development-only

| Package | Used for | Licence | Notes |
| --- | --- | --- | --- |
| `pytest` | test runner | MIT | pinned with hashes; not imported by the shipped package |
| `pytest-timeout` | fails a hanging test instead of hanging CI | MIT | optional |
| `ruff` | linting and formatting | MIT | configuration lives in the repository |
| `mypy` or `pyright` | typing | MIT / Apache-2.0 | one of them, not both |

## Distribution notes

- Where a package offers a choice, this project relies on the permissive branch of that choice (MIT, not Apache-2.0) and
  therefore ships no `NOTICE` file. Re-check the locked version's source for a `NOTICE` before packaging a distribution.
- The rule that keeps this page short is enforced, not aspirational: a CI job fails when an unpinned dependency appears,
  and a dependency that phones home, runs install-time code or drags in a build toolchain is refused in
  [dependencies](docs/dependencies.md) §5 regardless of convenience.

## This project

`auto-tzim` is [MIT](LICENSE).
